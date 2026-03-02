use std::{collections::HashMap, sync::Arc, thread};

use glam::{IVec3, Vec3};
use parking_lot::RwLock;
use rtrb::RingBuffer;
use tokio::io;

use crate::{
    ComposableGenerator, DeltaTime,
    cam_controller::CamController,
    chunk::BitMap3D,
    data_structures::LockHashMap,
    flood_fill::{SphereConfig, SphereGeneratorAllocations},
    mpsc,
    physics::TCBody,
    task::{self, Task},
    voxel::VoxelData3D,
    worker::Threadpool,
};

pub type LodLevel = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkID {
    pub lod: LodLevel,
    pub pos: IVec3,
}

impl ChunkID {
    pub fn new(lod: LodLevel, pos: IVec3) -> Self {
        Self { lod, pos }
    }

    pub fn total_pos(self) -> IVec3 {
        self.pos << self.lod
    }

    pub fn parent(self) -> Self {
        Self {
            lod: self.lod + 1,
            pos: self.pos >> 1,
        }
    }

    pub fn size(self) -> f32 {
        (1 << self.lod) as f32
    }

    pub fn from_pos(v: Vec3, lod: LodLevel) -> Self {
        Self {
            lod,
            pos: v.floor().as_ivec3(),
        }
    }
}

impl From<Vec3> for ChunkID {
    fn from(value: Vec3) -> Self {
        Self {
            lod: 0,
            pos: value.floor().as_ivec3(),
        }
    }
}

pub struct Player {
    body: TCBody,
    dir: Vec3,
    delta_time: DeltaTime,
}

pub enum ConfigUpdates {
    WorldGeneration(SphereConfig),
}

pub struct RenderThreadChannels {
    pub config_updates: rtrb::Producer<ConfigUpdates>,
    pub player: Arc<RwLock<CamController>>,
    pub voxel_collider: Arc<LockHashMap<ChunkID, BitMap3D>>,
    pub mesh_updates: mpsc::Receiver<()>,
}

const M_S_PER_TICK: usize = 1_000_000 / 60;

pub fn create_engine_thread(
    workers: usize,
    mut sphere_config: SphereConfig,
    player: CamController,
    world_generator: ComposableGenerator,
) -> Result<RenderThreadChannels, io::Error> {
    // render thread interface
    let (config_updates, mut config_updates_recv) = RingBuffer::new(10);

    let player = Arc::new(RwLock::new(player));
    let player_render = player.clone();

    let collider = Arc::new(LockHashMap::<ChunkID, BitMap3D>::new());
    let collider_render = collider.clone();

    let (mesh_updates_producer, mesh_updates_recv) = mpsc::new(10_000);

    thread::Builder::new()
        .name("engine thread".to_owned())
        .spawn(move || -> Result<(), io::Error> {
            let worker_count = (num_cpus::get() - 2).min(workers).max(1); // minus main + engine thread

            let (chunk_submitter, chunk_submission_queue) =
                mpsc::new::<()>(M_S_PER_TICK / 20 * worker_count);
            let mut working_class = Threadpool::new(
                worker_count,
                task::Context {
                    world_generator,
                    meshes: mesh_updates_producer,
                },
            )?;

            let mut sphere_generator_allocations = SphereGeneratorAllocations::new(5000);
            let mut players_last_pos = None;

            let mut chunks: HashMap<ChunkID, VoxelData3D> = HashMap::with_capacity(10_000);

            loop {
                // update configs
                while let Ok(config_update) = config_updates_recv.pop() {
                    use ConfigUpdates::*;
                    match config_update {
                        WorldGeneration(config) => sphere_config = config,
                    }
                }

                // submit chunk generation tasks
                let player_pos = (player.read().pos() / 32.).round();
                if Some(player_pos) != players_last_pos {
                    players_last_pos = Some(player_pos);

                    sphere_config.flood_fill(
                        player_pos,
                        &mut sphere_generator_allocations,
                        |chunk| {
                            if !chunks.contains_key(&chunk) {
                                working_class
                                    .submit_with_chunk(chunk, Task::GenerateChunk { chunk })
                            }
                        },
                    );
                }
            }
        })?;

    Ok(RenderThreadChannels {
        config_updates,
        player: player_render,
        voxel_collider: collider_render,
        mesh_updates: mesh_updates_recv,
    })
}
