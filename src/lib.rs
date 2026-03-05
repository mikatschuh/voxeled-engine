use glam::{IVec3, Vec3};

pub mod physics;

mod bitvec;
pub mod cam_controller;
mod chunk;
#[allow(dead_code)]
// mod sampling;
#[allow(dead_code)]
mod data_structures;
mod engine;
mod flood_fill;
pub mod frustum;
mod mesh;
mod meshing;
pub mod mpsc;
mod random;
mod task;
mod task_submission;
mod worker;
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

pub type MeshReceiver = MpscReceiver<(ChunkID, Mesh)>;

pub use chunk::{Chunk, VoxelType};
pub use engine::{ChunkID, ConfigUpdates, LodLevel, RenderThreadChannels, create_engine_thread};
pub use flood_fill::SphereConfig;
pub use frustum::{Frustum, FrustumAllocations};
pub use mesh::{Instance, Mesh, TextureID};
pub use mpsc::{Receiver as MpscReceiver, Sender as MpscSender, new as mpsc_channel};
pub use random::Noise;
pub use time::{DeltaTime, DeltaTimeMeter};
pub use world_gen::{
    ComposableGenerator, Gen2D, Gen3D, GenBox, Generator, MaterialGenerator, Seed,
};
