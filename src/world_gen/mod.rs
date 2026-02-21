use glam::IVec3;

use crate::{
    ChunkID, VoxelType,
    random::Noise,
    voxel::{self, VoxelData3D},
};

pub mod generators;

pub type Seed = u64;
pub trait Generator: Clone + Send + Sync + 'static {
    fn generate(&self, chunk_id: ChunkID) -> VoxelData3D;
}

#[derive(Debug, Clone)]
pub enum ShapeGenerator {
    Gen2D(Gen2D),
    Gen3D(Gen3D),
    Box(Box),
}

#[derive(Debug, Clone)]
pub struct Gen2D {
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
pub struct Box {
    pub min: IVec3,
    pub max: IVec3,
}

#[derive(Debug, Clone)]
pub struct MaterialGenerator {
    pub noise: Noise,
    pub scale: f64,
    pub threshold: f64,
    pub octaves: usize,
}

impl MaterialGenerator {
    pub fn new(seed: Seed) -> Self {
        Self {
            noise: Noise::new(seed as u32 ^ 0b11010101010101010100011010101010),
            scale: 8.0,
            threshold: 0.6,
            octaves: 3,
        }
    }
}

impl MaterialGenerator {
    pub fn generate(&self, pos: IVec3) -> VoxelType {
        let mat = self.noise.get_octaves(
            pos.x as f64,
            pos.y as f64,
            pos.z as f64,
            self.scale,
            self.octaves,
        );

        match mat {
            _ if mat >= self.threshold => VoxelType::CrackedStone,
            _ => VoxelType::Stone,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    generator: ShapeGenerator,
    material: Option<MaterialGenerator>,
}

#[derive(Debug, Clone)]
pub struct ComposeableGenerator {
    gen_stack: Vec<Layer>,
}

impl Generator for ComposeableGenerator {
    fn generate(&self, chunk: ChunkID) -> VoxelData3D {
        let mut voxel = voxel::fill(VoxelType::Air);
        for layer in self.gen_stack.iter() {
            let material = |pos: IVec3| {
                layer
                    .material
                    .as_ref()
                    .map_or(VoxelType::Air, |mat| mat.generate(pos))
            };

            match &layer.generator {
                ShapeGenerator::Gen2D(generator) => generator.generate(chunk, &mut voxel, material),
                ShapeGenerator::Gen3D(generator) => generator.generate(chunk, &mut voxel, material),
                ShapeGenerator::Box(generator) => generator.generate(chunk, &mut voxel, material),
            }
        }

        voxel
    }
}

impl Gen2D {
    fn generate(
        &self,
        chunk: ChunkID,
        voxel: &mut VoxelData3D,
        material: impl Fn(IVec3) -> VoxelType,
    ) {
        for (x, plane) in voxel.iter_mut().enumerate() {
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
                    plane[y][z] = if pos_y
                        < ((2.0_f64.powf(height as f64) * self.y_scale) - self.base_height) as i32
                    {
                        material(IVec3::new(pos_x, pos_y, pos_z))
                    } else {
                        VoxelType::Air
                    }
                }
            }
        }
    }
}

impl Gen3D {
    fn generate(
        &self,
        chunk: ChunkID,
        voxel: &mut VoxelData3D,
        material: impl Fn(IVec3) -> VoxelType,
    ) {
        for (x, plane) in voxel.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let pos = IVec3::new(
                        (x as i32 + chunk.pos.x * 32) << chunk.lod,
                        (y as i32 + chunk.pos.y * 32) << chunk.lod,
                        (z as i32 + chunk.pos.z * 32) << chunk.lod,
                    );

                    let val = self.noise.get_octaves(
                        pos.x as f64 / self.x_scale,
                        pos.y as f64 / self.y_scale,
                        pos.z as f64 / self.z_scale,
                        1.,
                        self.octaves,
                    );

                    *voxel = if val.powf(self.exponent as f64) <= self.threshold {
                        VoxelType::Air
                    } else {
                        material(pos)
                    }
                }
            }
        }
    }
}

impl Box {
    fn generate(
        &self,
        chunk: ChunkID,
        voxel: &mut VoxelData3D,
        material: impl Fn(IVec3) -> VoxelType,
    ) {
        for (x, plane) in voxel.iter_mut().enumerate() {
            for (y, row) in plane.iter_mut().enumerate() {
                for (z, voxel) in row.iter_mut().enumerate() {
                    let pos = IVec3::new(
                        (x as i32 + chunk.pos.x * 32) << chunk.lod,
                        (y as i32 + chunk.pos.y * 32) << chunk.lod,
                        (z as i32 + chunk.pos.z * 32) << chunk.lod,
                    );

                    *voxel = if pos.cmpge(self.min).all() && pos.cmplt(self.max).all() {
                        VoxelType::Air
                    } else {
                        material(pos)
                    }
                }
            }
        }
    }
}
