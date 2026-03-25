use glam::{IVec3, Vec3};

pub mod cam_controller;
pub mod config;
pub mod config_loader;
pub mod error;
pub mod frustum;
pub mod mpsc;
pub mod physics;

mod bitvec;
mod chunk;
#[allow(dead_code)]
// mod sampling;
#[allow(dead_code)]
mod data_structures;
#[macro_use]
#[allow(dead_code)]
mod debug;
mod engine;
mod flood_fill;
mod mesh;
mod meshing;
mod random;
mod worker;
mod worker_pool;
mod worker_spsc;
mod world_gen;

#[cfg(test)]
mod test;

mod time;
mod voxel;

pub fn block(v: Vec3) -> IVec3 {
    v.floor().as_ivec3()
}

pub fn block_coord(n: f32) -> i32 {
    n.floor() as i32
}

pub type MeshReceiver = MpscReceiver<(ChunkID, MeshUpload)>;

pub use chunk::{Chunk, ChunkID, Lod, VoxelType};
pub use engine::{RenderThreadChannels, Update, engine_thread};
pub use flood_fill::SphereGeneratorAllocations;
pub use frustum::{Frustum, FrustumAllocations};
pub use mesh::{Instance, MeshUpload, TextureID};
pub use mpsc::{Receiver as MpscReceiver, Sender as MpscSender, new as mpsc_channel};
pub use random::Noise;
pub use time::{DeltaTime, DeltaTimeMeter};
pub use voxel::VoxelTypes;
pub use world_gen::{ComposableGenerator, Gen2D, Gen3D, GenBox, Generator, Seed};
pub mod spsc {
    pub use rtrb::Consumer;
    pub use rtrb::Producer;
}
