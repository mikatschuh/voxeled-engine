use glam::IVec3;

use crate::{
    ChunkID, VoxelType,
    chunk::{CHUNK_VOLUME, DenseChunk, idx_to_coord},
    random::Noise,
    voxel::{self, VoxelTypes},
};

pub mod generators;

pub type Seed = u64;
pub trait Generator: Clone + Send + Sync + 'static {
    fn generate(&self, chunk_id: ChunkID) -> DenseChunk;
}

#[derive(Debug, Clone)]
pub enum ShapeGenerator {
    Gen2D(Gen2D),
    Gen3D(Gen3D),
    Box(GenBox),
    Full,
}

#[derive(Debug, Clone)]
pub struct Gen2D {
    pub invert: bool,

    pub noise: Noise,
    pub octaves: usize,
    pub base_height: f64,
    pub x_scale: f64,
    pub y_scale: f64,
    pub z_scale: f64,
}

#[derive(Debug, Clone)]
pub struct Gen3D {
    pub noise: Noise,
    pub octaves: usize,
    pub x_scale: f64,
    pub y_scale: f64,
    pub z_scale: f64,
    pub exponent: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone)]
pub struct GenBox {
    pub invert: bool,
    pub min: IVec3,
    pub max: IVec3,
}

#[derive(Debug, Clone)]
struct Layer {
    generator: ShapeGenerator,
    material: VoxelTypes,
}

#[derive(Debug, Clone)]
pub struct ComposableGenerator {
    gen_stack: Vec<Layer>,
}

impl Generator for ComposableGenerator {
    fn generate(&self, chunk: ChunkID) -> DenseChunk {
        let mut voxel = voxel::fill(VoxelTypes::Air as u16);
        for layer in self.gen_stack.iter() {
            let material = layer.material as u16;
            match &layer.generator {
                ShapeGenerator::Gen2D(generator) => generator.generate(chunk, &mut voxel, material),
                ShapeGenerator::Gen3D(generator) => generator.generate(chunk, &mut voxel, material),
                ShapeGenerator::Box(generator) => generator.generate(chunk, &mut voxel, material),
                ShapeGenerator::Full => (0..CHUNK_VOLUME).for_each(|i| voxel[i] = material as u16),
            }
        }

        voxel
    }
}

impl Gen2D {
    fn generate(&self, chunk: ChunkID, voxel: &mut DenseChunk, material: VoxelType) {
        for (x, plane) in voxel.chunks_mut(32 * 32).enumerate() {
            for z in 0..32 {
                let pos_x = (x as i32 + chunk.pos.x * 32) << chunk.lod;
                let pos_z = (z as i32 + chunk.pos.z * 32) << chunk.lod;

                let height = self.noise.get_octaves(
                    pos_x as f64 / self.x_scale,
                    0.0,
                    pos_z as f64 / self.z_scale,
                    1.,
                    self.octaves,
                );
                for y in 0..32 {
                    let pos_y = (y as i32 + chunk.pos.y * 32) << chunk.lod;

                    plane[y * 32 + z] = if pos_y
                        < ((2.0_f64.powf(height as f64 * self.y_scale)) - self.base_height) as i32
                    {
                        if self.invert {
                            continue;
                        }
                        material
                    } else if self.invert {
                        material
                    } else {
                        continue;
                    }
                }
            }
        }
    }
}

impl Gen3D {
    fn generate(&self, chunk: ChunkID, voxel: &mut DenseChunk, material: VoxelType) {
        for (i, voxel) in voxel.iter_mut().enumerate() {
            let coord = idx_to_coord(i);

            let pos = IVec3::new(
                (coord.x as i32 + chunk.pos.x * 32) << chunk.lod,
                (coord.y as i32 + chunk.pos.y * 32) << chunk.lod,
                (coord.z as i32 + chunk.pos.z * 32) << chunk.lod,
            );

            let val = self.noise.get_octaves(
                pos.x as f64 / self.x_scale,
                pos.y as f64 / self.y_scale,
                pos.z as f64 / self.z_scale,
                1.,
                self.octaves,
            );

            *voxel = if val.powf(self.exponent as f64) < self.threshold {
                continue;
            } else {
                material
            }
        }
    }
}

impl GenBox {
    fn generate(&self, chunk: ChunkID, voxel: &mut DenseChunk, material: VoxelType) {
        for (i, voxel) in voxel.iter_mut().enumerate() {
            let coord = idx_to_coord(i);

            let pos = IVec3::new(
                (coord.x as i32 + chunk.pos.x * 32) << chunk.lod,
                (coord.y as i32 + chunk.pos.y * 32) << chunk.lod,
                (coord.z as i32 + chunk.pos.z * 32) << chunk.lod,
            );

            *voxel = if pos.cmpge(self.min).all() && pos.cmplt(self.max).all() {
                if self.invert {
                    continue;
                }
                material
            } else if self.invert {
                material
            } else {
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::IVec3;

    use crate::{ChunkID, voxel::VoxelTypes};

    use super::Gen3D;

    #[test]
    fn gen3d_uses_z_coordinate_for_world_z() {
        let chunk = ChunkID::new(0, IVec3::new(0, 0, 0));
        let mut out = [0_u16; 32 * 32 * 32];
        let gen3d = Gen3D {
            noise: crate::Noise::new(1),
            octaves: 1,
            x_scale: 1.0,
            y_scale: 1.0,
            z_scale: 1.0,
            exponent: 1.0,
            threshold: -1.0,
        };

        gen3d.generate(chunk, &mut out, VoxelTypes::Air as u16);

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let idx = x * 32 * 32 + y * 32 + z;
                    assert_eq!(out[idx], z as u16);
                }
            }
        }
    }
}
