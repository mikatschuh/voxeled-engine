use glam::{IVec3, Vec3};

pub mod physics;

mod chunk;
#[allow(dead_code)]
// mod sampling;
#[allow(dead_code)]
mod data_structures;
pub mod frustum;
mod job;
mod mesh;
mod meshing;
mod random;
mod server;
mod world_gen;

#[cfg(test)]
mod test;

mod threadpool;
mod time;
mod voxel;

pub fn block(v: Vec3) -> IVec3 {
    v.floor().as_ivec3()
}

pub fn block_coord(n: f32) -> i32 {
    n.floor() as i32
}

pub use chunk::ChunkID;
pub use frustum::Frustum;
pub use mesh::{Instance, Mesh, TextureID};
pub use server::Server;
pub use threadpool::Threadpool;
pub use time::{DeltaTime, DeltaTimeMeter};
pub use voxel::VoxelType;
pub use world_gen::{
    Box, ComposeableGenerator, Gen2D, Gen3D, Generator, Layer, Seed, ShapeGenerator, generators,
};
