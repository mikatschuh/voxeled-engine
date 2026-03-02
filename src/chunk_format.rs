use std::{
    collections::{HashMap, VecDeque},
    mem,
};

use glam::UVec3;

use crate::bitvec::PackedVec32;

pub type VoxelType = u16;
pub type PaletteID = u16;

pub struct Chunk {
    palette_index_size: u8, // number of bits needed to represent every palette entry
    type_to_id: HashMap<VoxelType, PaletteID>,
    free_list: VecDeque<PaletteID>,

    max_palette_size: usize,
    palette: Vec<VoxelType>, // contains palette and blocks or just palette
    palette_rc: Vec<u16>,

    voxel: PackedVec32, // bit-packed
}

impl Chunk {
    pub fn from_buffer(buffer: &[[[VoxelType; 32]; 32]; 32]) -> Self {
        let every_input_voxel = buffer
            .iter()
            .flat_map(|plane| plane.iter())
            .flat_map(|collum| collum.iter());

        let mut voxel_type_uses = [0_u16; 65_536];
        every_input_voxel
            .clone()
            .for_each(|v| voxel_type_uses[*v as usize] += 1);

        let mut type_to_id: HashMap<VoxelType, u16> = HashMap::new();
        let mut palette: Vec<VoxelType> = Vec::new();
        let mut palette_num_of_uses: Vec<u16> = Vec::new();
        for (voxel_type, num_of_uses) in voxel_type_uses.into_iter().enumerate() {
            let voxel_type = voxel_type as VoxelType;

            if num_of_uses > 0 {
                let palette_id = palette.len() as PaletteID;
                type_to_id.insert(voxel_type, palette_id);

                palette.push(voxel_type);
                palette_num_of_uses.push(num_of_uses);
            }
        }

        let palette_index_size = log2_round_down(palette.len());

        let mut voxel = PackedVec32::new(32 * 32 * 32, palette_index_size);
        if palette_index_size != 0 {
            every_input_voxel
                .enumerate()
                .for_each(|(i, v)| voxel.set(i, *type_to_id.get(v).unwrap() as u32));
        }

        Self {
            palette_index_size,
            type_to_id,
            free_list: VecDeque::new(),
            max_palette_size: 1_usize << palette_index_size,
            palette,
            palette_rc: palette_num_of_uses,
            voxel,
        }
    }

    pub fn get(&self, coord: UVec3) -> VoxelType {
        self.palette[self.voxel.get(coords_to_1d_index(coord)) as usize]
    }

    pub fn set(&mut self, coord: UVec3, voxel_type: VoxelType) {
        let index = coords_to_1d_index(coord);

        let old_palette_index = self.voxel.get(index) as usize; // HashMap Lookup

        // decrement reference count of old palette entry and potentially remove old palette entry
        let prev_voxel_type_rc = &mut self.palette_rc[old_palette_index];
        *prev_voxel_type_rc -= 1;
        if *prev_voxel_type_rc == 0 {
            self.type_to_id.remove(&self.palette[old_palette_index]);

            // rebuild palettes as the half of them is used up
            if self.max_palette_size - self.palette.len() >= 8
                && self.free_list.len() >= self.palette.len() / 2 + 1
            {
                self.free_list.clear();

                let mut new_palette_rc: Vec<u16> = vec![];
                let mut new_palette: Vec<VoxelType> = vec![];
                self.type_to_id = self
                    .type_to_id
                    .iter()
                    .enumerate()
                    .map(|(i, (voxel_type, _))| {
                        new_palette_rc.push(self.palette_rc[i]);
                        new_palette.push(*voxel_type);
                        (*voxel_type, i as u16)
                    })
                    .collect::<HashMap<VoxelType, PaletteID>>();

                self.palette_rc = new_palette_rc;

                self.palette_index_size = log2_round_down(new_palette.len());
                self.max_palette_size = 1_usize << self.palette_index_size;

                let mut new_voxels = PackedVec32::new(32_768, self.palette_index_size);
                for i in 0..32_768 {
                    new_voxels.set(
                        i,
                        self.type_to_id[&self.palette[self.voxel.get(i) as usize]] as u32,
                    );
                }
                self.voxel = new_voxels;
                self.palette = new_palette
            } else {
                self.free_list.push_back(old_palette_index as PaletteID); // if no rebuild threshold reached => use free list
            }
        }

        if let Some(existing_palette_index) = self.type_to_id.get(&voxel_type) {
            self.voxel.set(index, *existing_palette_index as u32);
            self.palette_rc[*existing_palette_index as usize] += 1;
            return;
        } else if let Some(free_index) = self.free_list.pop_front() {
            let free_index_usize = free_index as usize;
            self.palette[free_index_usize] = voxel_type; // make new palette entry by reusing old slot
            self.palette_rc[free_index_usize] = 1;
            self.type_to_id.insert(voxel_type, free_index);

            self.voxel.set(index, free_index as u32);
            return;
        }
        if self.palette.len() >= self.max_palette_size {
            self.palette_index_size += 1;
            self.voxel.repack_in_place(self.palette_index_size + 1);
        }
        self.type_to_id
            .insert(voxel_type, self.palette.len() as PaletteID);
        self.voxel.set(index, self.palette.len() as u32);

        self.palette.push(voxel_type); // make new palette entry
        self.palette_rc.push(1);
    }

    pub fn calculate_memory_usage(&self) -> usize {
        // constants:
        let mut total = mem::size_of::<Self>();

        total += self.free_list.capacity() * 2;
        total += self.palette.capacity() * 2;
        total += self.palette_rc.capacity() * 2;
        total += self.type_to_id.capacity() * 4;

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
