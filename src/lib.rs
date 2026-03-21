use glam::{IVec3, Vec3};

pub mod cam_controller;
pub mod engine_config;
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
mod config_loader;
mod engine;
mod flood_fill;
mod mesh;
mod meshing;
mod random;
mod task_submission;
mod worker;
mod worker_pool;
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

pub use chunk::{Chunk, VoxelType};
pub use config_loader::{ConfigFile, Live, config_thread};
pub use engine::{ChunkID, LodLevel, RenderThreadChannels, Update, engine_thread};
pub use engine_config::{Config, ConfigUpdate};
pub use flood_fill::SphereGeneratorAllocations;
pub use frustum::{Frustum, FrustumAllocations};
pub use mesh::{Instance, MeshUpload, TextureID};
pub use mpsc::{Receiver as MpscReceiver, Sender as MpscSender, new as mpsc_channel};
pub use random::Noise;
pub use time::{DeltaTime, DeltaTimeMeter};
pub use voxel::VoxelTypes;
pub use world_gen::{ComposableGenerator, Gen2D, Gen3D, GenBox, Generator, Seed};
