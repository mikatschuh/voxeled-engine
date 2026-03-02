use std::{
    collections::{HashMap, VecDeque},
    mem,
};

use glam::UVec3;

use crate::{bitvec::PackedVec32, chunk::CHUNK_VOLUME};

pub type VoxelType = u16;
pub type PaletteID = u16;
const UNCOMPRESSED_RECHECK_INTERVAL: usize = CHUNK_VOLUME;

pub struct Chunk {
    count_of_change: usize,
    palette_data: Option<PaletteData>,
    dense_data: Option<Box<[VoxelType; CHUNK_VOLUME]>>,

    voxel: PackedVec32, // bit-packed
}

pub struct PaletteData {
    palette_index_size: u8, // number of bits needed to represent every palette entry
    type_to_id: HashMap<VoxelType, PaletteID>,
    free_list: VecDeque<PaletteID>,

    max_palette_size: usize,
    palette: Vec<VoxelType>, // contains palette and blocks or just palette
    palette_rc: Vec<u16>,
}

impl Chunk {
    pub fn from_buffer(buffer: &[VoxelType; CHUNK_VOLUME]) -> Self {
        let every_input_voxel = buffer.iter().copied();
        Self::from_iterator(every_input_voxel)
    }

    fn from_iterator(every_input_voxel: impl Iterator<Item = u16> + Clone) -> Self {
        let mut voxel_type_uses = [0_u16; 65_536];
        every_input_voxel
            .clone()
            .for_each(|v| voxel_type_uses[v as usize] += 1);

        let mut type_to_id: HashMap<VoxelType, u16> = HashMap::new();
        let mut palette: Vec<VoxelType> = Vec::new();
        let mut palette_rc: Vec<u16> = Vec::new();
        for (voxel_type, num_of_uses) in voxel_type_uses.into_iter().enumerate() {
            let voxel_type = voxel_type as VoxelType;

            if num_of_uses > 0 {
                let palette_id = palette.len() as PaletteID;
                type_to_id.insert(voxel_type, palette_id);

                palette.push(voxel_type);
                palette_rc.push(num_of_uses);
            }
        }

        let palette_index_size = log2_round_down(palette.len());

        let memory_usage = type_to_id.capacity() * 4
            + palette.capacity() * 2
            + palette_rc.capacity() * 2
            + (CHUNK_VOLUME * palette_index_size as usize).div_ceil(32) * 4;

        if memory_usage > CHUNK_VOLUME * 2 {
            let mut dense = Box::new([0_u16; CHUNK_VOLUME]);
            every_input_voxel
                .enumerate()
                .for_each(|(i, v)| dense[i] = v);

            return Self {
                count_of_change: 0,
                palette_data: None,
                dense_data: Some(dense),
                voxel: PackedVec32::new(0, 1),
            };
        }

        let mut voxel = PackedVec32::new(CHUNK_VOLUME, palette_index_size);
        if palette_index_size != 0 {
            every_input_voxel
                .enumerate()
                .for_each(|(i, v)| voxel.set(i, *type_to_id.get(&v).unwrap() as u32));
        }

        Self {
            count_of_change: 0,
            palette_data: Some(PaletteData {
                palette_index_size,
                type_to_id,
                free_list: VecDeque::new(),
                max_palette_size: 1 << palette_index_size,
                palette,
                palette_rc,
            }),
            dense_data: None,
            voxel,
        }
    }

    pub fn get(&self, coord: UVec3) -> VoxelType {
        if let Some(p_data) = &self.palette_data {
            if p_data.palette_index_size == 0 {
                p_data.palette[0]
            } else {
                p_data.palette[self.voxel.get(coords_to_1d_index(coord)) as usize]
            }
        } else {
            self.dense_data.as_ref().unwrap()[coords_to_1d_index(coord)]
        }
    }

    pub fn set(&mut self, coord: UVec3, voxel_type: VoxelType) {
        let index = coords_to_1d_index(coord);

        let Some(p_data) = &mut self.palette_data else {
            let dense = self.dense_data.as_mut().unwrap();
            let old = dense[index];
            if old == voxel_type {
                return;
            }

            dense[index] = voxel_type;

            if self.count_of_change >= UNCOMPRESSED_RECHECK_INTERVAL {
                let every_input_voxel = dense.iter().copied();

                *self = Self::from_iterator(every_input_voxel);
                return;
            }

            self.count_of_change += 1;
            return;
        };

        // Singleton palette is represented with 0 packed bits.
        // Avoid touching `self.voxel` in that state until we promote to 1-bit storage.
        if p_data.palette_index_size == 0 {
            if p_data.palette[0] == voxel_type {
                return;
            }
            self.voxel = PackedVec32::new(CHUNK_VOLUME, 1); // allocate packed storage
            (0..CHUNK_VOLUME).for_each(|i| self.voxel.set(i, 0));
            p_data.palette_index_size = 1;
            p_data.max_palette_size = 2;
        } else {
            let current_palette_index = self.voxel.get(index) as usize;
            if p_data.palette[current_palette_index] == voxel_type {
                return;
            }
        }

        let old_palette_index = self.voxel.get(index) as usize;

        // decrement reference count of old palette entry and potentially remove old palette entry
        let prev_voxel_type_rc = &mut p_data.palette_rc[old_palette_index];
        *prev_voxel_type_rc -= 1;
        if *prev_voxel_type_rc == 0 {
            p_data.type_to_id.remove(&p_data.palette[old_palette_index]); // remove from table

            if let Some(existing_palette_index) = p_data.type_to_id.get(&voxel_type) {
                self.voxel.set(index, *existing_palette_index as u32);
                p_data.palette_rc[*existing_palette_index as usize] += 1;

                p_data.free_list.push_back(old_palette_index as PaletteID); // create tombstone
                return;
            }

            p_data
                .type_to_id
                .insert(voxel_type, old_palette_index as PaletteID); // make new palette entry by reusing old slot
            p_data.palette[old_palette_index] = voxel_type;
            p_data.palette_rc[old_palette_index] = 1;
            self.voxel.set(index, old_palette_index as u32);

            // rebuild palettes as the half of them is used up by tombstones
            if p_data.max_palette_size - p_data.palette.len() >= 8
                && p_data.free_list.len() >= p_data.palette.len() / 2
            {
                p_data.free_list.clear();

                let mut new_palette_rc: Vec<u16> = vec![];
                let mut new_palette: Vec<VoxelType> = vec![];
                p_data.type_to_id = p_data
                    .type_to_id
                    .iter()
                    .enumerate()
                    .map(|(i, (voxel_type, old_i))| {
                        new_palette_rc.push(p_data.palette_rc[*old_i as usize]);
                        new_palette.push(*voxel_type);
                        (*voxel_type, i as u16)
                    })
                    .collect::<HashMap<VoxelType, PaletteID>>();
                p_data.palette_rc = new_palette_rc;

                p_data.palette_index_size = log2_round_down(new_palette.len());
                p_data.max_palette_size = 1 << p_data.palette_index_size;

                let mut new_voxels = PackedVec32::new(32_768, p_data.palette_index_size);
                if p_data.palette_index_size != 0 {
                    (0..CHUNK_VOLUME).for_each(|i| {
                        new_voxels.set(
                            i,
                            *p_data
                                .type_to_id
                                .get(&p_data.palette[self.voxel.get(i) as usize])
                                .unwrap() as u32,
                        );
                    })
                };
                self.voxel = new_voxels;
                p_data.palette = new_palette;
            }
            return;
        }

        if let Some(existing_palette_index) = p_data.type_to_id.get(&voxel_type) {
            self.voxel.set(index, *existing_palette_index as u32);
            p_data.palette_rc[*existing_palette_index as usize] += 1;
            return;
        } else if let Some(free_index) = p_data.free_list.pop_front() {
            let free_index_usize = free_index as usize;
            p_data.palette[free_index_usize] = voxel_type; // make new palette entry by reusing old slot
            p_data.palette_rc[free_index_usize] = 1;
            p_data.type_to_id.insert(voxel_type, free_index);

            self.voxel.set(index, free_index as u32);

            return;
        }

        if p_data.palette.len() >= p_data.max_palette_size {
            p_data.palette_index_size += 1;
            self.voxel.repack_in_place(p_data.palette_index_size);
            p_data.max_palette_size = 1_usize << p_data.palette_index_size;

            self.count_of_change += 1;
        }
        p_data
            .type_to_id
            .insert(voxel_type, p_data.palette.len() as PaletteID);
        self.voxel.set(index, p_data.palette.len() as u32);

        p_data.palette.push(voxel_type); // make new palette entry
        p_data.palette_rc.push(1);

        if self.count_of_change >= 2 {
            self.count_of_change = 0;
            let memory_usage = p_data.type_to_id.capacity() * 4
                + p_data.palette.capacity() * 2
                + p_data.palette_rc.capacity() * 2
                + (CHUNK_VOLUME * p_data.palette_index_size as usize).div_ceil(32) * 4;

            if memory_usage > CHUNK_VOLUME * 2 {
                let mut dense = Box::new([0_u16; CHUNK_VOLUME]);
                (0..CHUNK_VOLUME)
                    .for_each(|i| dense[i] = p_data.palette[self.voxel.get(i) as usize]);

                self.palette_data = None;
                self.dense_data = Some(dense);
                self.voxel = PackedVec32::new(0, 1);
            }
        }
    }

    pub fn to_buffer(&self) -> [VoxelType; CHUNK_VOLUME] {
        if let Some(p_data) = &self.palette_data {
            let mut buffer = [p_data.palette[0]; CHUNK_VOLUME];

            if p_data.palette_index_size != 0 {
                (0..CHUNK_VOLUME)
                    .for_each(|i| buffer[i] = p_data.palette[self.voxel.get(i) as usize]);
            }
            buffer
        } else {
            **self.dense_data.as_ref().unwrap()
        }
    }

    pub fn calculate_memory_usage(&self) -> usize {
        // constants:
        let mut total = mem::size_of::<Self>();

        if let Some(p_data) = &self.palette_data {
            total += p_data.free_list.capacity() * 2;
            total += p_data.palette.capacity() * 2;
            total += p_data.palette_rc.capacity() * 2;
            total += p_data.type_to_id.capacity() * 4;
            total += self.voxel.len() * self.voxel.bits_per_elem() as usize;
        } else {
            total += CHUNK_VOLUME * 2
        }
        total
    }
}

#[inline(always)]
fn coords_to_1d_index(coord: UVec3) -> usize {
    (coord.x * 32 * 32 + coord.y * 32 + coord.z) as usize
}

#[inline(always)]
fn log2_round_down(x: usize) -> u8 {
    (x as f32).log2().ceil() as u8
}

#[cfg(test)]
mod tests {
    use rand::{Rng, random, thread_rng};

    use crate::chunk::CHUNK_VOLUME;

    use super::*;

    fn idx_to_coord(i: usize) -> UVec3 {
        let x = i / (32 * 32);
        let yz = i % (32 * 32);
        let y = yz / 32;
        let z = yz % 32;
        UVec3::new(x as u32, y as u32, z as u32)
    }

    fn exp_u16<R: Rng + ?Sized>(rng: &mut R, lambda: f64) -> u16 {
        let max = u16::MAX as f64;
        let u: f64 = rng.r#gen(); // [0,1)
        // Trunkierte Exp auf [0,1], dann auf [0, u16::MAX] skalieren
        let x01 = -((1.0 - u * (1.0 - (-lambda).exp())).ln()) / lambda;
        (x01 * max).round() as u16
    }

    #[test]
    fn set_does_not_skip_when_palette_zero_matches_target() {
        let mut buffer = [0_u16; CHUNK_VOLUME];
        for (i, v) in buffer.iter_mut().enumerate() {
            *v = if i % 2 == 0 { 1 } else { 2 };
        }

        let mut chunk = Chunk::from_buffer(&buffer);
        let idx = 1; // currently 2
        let coord = idx_to_coord(idx);
        chunk.set(coord, 1);
        buffer[idx] = 1;

        assert_eq!(chunk.to_buffer(), buffer);
    }

    #[test]
    fn randomized_mutations_match_dense_buffer() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let replaces = (0..50_000)
                .map(|_| {
                    (
                        UVec3::new(
                            random::<u32>() & 31,
                            random::<u32>() & 31,
                            random::<u32>() & 31,
                        ),
                        (exp_u16(&mut rng, 32.0) as f32).clamp(0., u16::MAX as f32) as u16,
                    )
                })
                .collect::<Vec<(UVec3, VoxelType)>>();

            let mut dense = [0_u16; CHUNK_VOLUME];
            let mut chunk = Chunk::from_buffer(&dense);

            replaces
                .iter()
                .for_each(|replace| dense[coords_to_1d_index(replace.0)] = replace.1);
            replaces
                .iter()
                .for_each(|replace| chunk.set(replace.0, replace.1));

            assert_eq!(chunk.to_buffer(), dense);
        }
    }
}
