use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    thread,
};

use glam::{IVec3, Vec3};
use parking_lot::RwLock;
use rtrb::RingBuffer;
use tokio::io;

use crate::{
    Chunk, ComposableGenerator, Mesh,
    cam_controller::CamController,
    flood_fill::{SphereConfig, SphereGeneratorAllocations, chunk_neighbors},
    meshing::BitMap3D,
    mpsc,
    task::{self, Task},
    task_submission::TaskSubmitter,
    worker::Threadpool,
};

pub type LodLevel = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkID {
    pub pos: IVec3,
    pub lod: LodLevel,
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

    pub fn from_pos(v: Vec3, lod: LodLevel) -> Self {
        Self {
            lod,
            pos: v.floor().as_ivec3(),
        }
    }

    pub fn bytes(&self) -> Box<[u8]> {
        let bytes = [
            self.pos.x.cast_unsigned(),
            self.pos.y.cast_unsigned(),
            self.pos.z.cast_unsigned(),
            self.lod as u32,
        ];
        bytemuck::cast_slice(&bytes).into()
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

pub enum ConfigUpdates {
    WorldGeneration(SphereConfig),
    ShutDown,
}

pub struct RenderThreadChannels {
    pub config_updates: rtrb::Producer<ConfigUpdates>,
    pub player: Arc<RwLock<CamController>>,
    pub voxel_collider: Arc<RwLock<HashMap<ChunkID, BitMap3D>>>,
    pub mesh_updates: mpsc::Receiver<(ChunkID, Mesh)>,
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

    let collider = Arc::new(RwLock::new(HashMap::<ChunkID, BitMap3D>::new()));
    let collider_render = collider.clone();

    let (mesh_updates_tx, mesh_updates_rx) = mpsc::new::<(ChunkID, Mesh)>(10_000);

    thread::Builder::new()
        .name("engine thread".to_owned())
        .spawn(move || -> Result<(), io::Error> {
            let worker_count = (num_cpus::get() - 2).min(workers).max(1); // minus main + engine thread

            let mut working_class_people = TaskSubmitter::new();

            let (chunk_tx, chunk_submission_queue) =
                mpsc::new::<(ChunkID, Chunk)>(M_S_PER_TICK / 20 * worker_count);
            let mut submitted_chunks: HashSet<ChunkID> = HashSet::with_capacity(10_000);

            let (collider_tx, collider_submission_queue) =
                mpsc::new(M_S_PER_TICK / 20 * worker_count);

            let (solid_maps_tx, solid_map_queue) =
                mpsc::new::<(ChunkID, Box<[BitMap3D; 3]>)>(10_000);

            let threadpool = Threadpool::new(worker_count, |_| task::Context {
                task_queue: working_class_people.add_worker(1000),
                world_generator: world_generator.clone(),

                chunk_tx: chunk_tx.clone(),
                collider_tx: collider_tx.clone(),
                solid_map_tx: solid_maps_tx.clone(),

                meshes: mesh_updates_tx.clone(),
            })?;

            let mut sphere_generator_allocations = SphereGeneratorAllocations::default(5000);
            let mut players_last_pos = None;

            let mut chunks: HashMap<ChunkID, Chunk> = HashMap::with_capacity(10_000);

            let mut solid_maps: [HashMap<ChunkID, BitMap3D>; 3] = [
                HashMap::with_capacity(10_000),
                HashMap::with_capacity(10_000),
                HashMap::with_capacity(10_000),
            ];

            'tick_loop: loop {
                // update configs
                while let Ok(config_update) = config_updates_recv.pop() {
                    use ConfigUpdates::*;
                    match config_update {
                        WorldGeneration(config) => sphere_config = config,
                        ShutDown => break 'tick_loop,
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
                            if submitted_chunks.insert(chunk) && !chunks.contains_key(&chunk) {
                                let mut axis = 0;
                                working_class_people.submit_task(
                                    chunk,
                                    Task::GenerateChunkAndMesh {
                                        chunk,
                                        neighbors: Box::new(chunk_neighbors(chunk).map(
                                            |neighbor| {
                                                let solid_map = solid_maps[axis >> 1]
                                                    .get(&neighbor)
                                                    .unwrap_or(&[[0_u32; 32]; 32])
                                                    .clone();
                                                axis += 1;
                                                solid_map
                                            },
                                        )),
                                    },
                                );
                            }
                        },
                    );
                }

                // process thread pool output
                while let Ok(submission) = chunk_submission_queue.pop() {
                    chunks.insert(submission.0, submission.1);
                }

                while let Ok((chunk, solid_map)) = solid_map_queue.pop() {
                    solid_maps[0].insert(chunk, solid_map[0]);
                    solid_maps[1].insert(chunk, solid_map[1]);
                    solid_maps[2].insert(chunk, solid_map[2]);
                }

                {
                    let mut collider = collider.write();
                    while let Ok((chunk, submission)) = collider_submission_queue.pop() {
                        collider.insert(chunk, *submission);
                    }
                }
            }
            drop(threadpool);
            Ok(())
        })?;

    Ok(RenderThreadChannels {
        config_updates,
        player: player_render,
        voxel_collider: collider_render,
        mesh_updates: mesh_updates_rx,
    })
}
