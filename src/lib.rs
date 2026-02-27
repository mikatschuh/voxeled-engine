use glam::{IVec3, Vec3};

pub mod physics;

pub mod cam_controller;
mod chunk;
#[allow(dead_code)]
// mod sampling;
#[allow(dead_code)]
mod data_structures;
mod engine;
mod flood_fill;
pub mod frustum;
mod job;
mod mesh;
mod meshing;
pub mod mpsc_channel;
mod random;
pub mod spsc_channel;
mod task;
mod worker;
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
pub use frustum::{Frustum, FrustumAllocations};
pub use mesh::{Instance, Mesh, TextureID};
pub use mpsc_channel::{Receiver as MpscReceiver, Sender as MpscSender, channel as mpsc_channel};
pub use random::Noise;
pub use spsc_channel::{Receiver as SpscReceiver, Sender as SpscSender, channel as spsc_channel};
pub use time::{DeltaTime, DeltaTimeMeter};
pub use voxel::VoxelType;
pub use world_gen::{
    ComposeableGenerator, Gen2D, Gen3D, GenBox, Generator, MaterialGenerator, Seed,
};
