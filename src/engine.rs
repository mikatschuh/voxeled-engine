use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    thread,
    time::Instant,
};

use glam::{IVec3, Vec3};
use parking_lot::RwLock;
use rtrb::RingBuffer;
use tokio::io;

use crate::{
    Chunk, ComposableGenerator, MeshReceiver,
    cam_controller::CamController,
    engine_config::{Config, ConfigUpdate},
    flood_fill::{SphereGeneratorAllocations, chunk_neighbors},
    mesh::MeshUpload,
    meshing::{BitMap2D, BitMap3D},
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

const MAX_LOD: usize = 16;

pub enum Update {
    ConfigUpdate { update: ConfigUpdate },
    ShutDown,
}

pub struct RenderThreadChannels {
    pub updates: rtrb::Producer<Update>,
    pub player: Arc<RwLock<CamController>>,
    pub voxel_collider: Arc<RwLock<HashMap<ChunkID, BitMap3D>>>,
    pub mesh_updates: MeshReceiver,
}

pub fn engine_thread(
    mut config: Config,
    player: CamController,
    world_generator: ComposableGenerator,
) -> Result<RenderThreadChannels, io::Error> {
    // render thread interface
    let (updates, mut updates_recv) = RingBuffer::new(16);

    let player = Arc::new(RwLock::new(player));
    let player_render = player.clone();

    let collider = Arc::new(RwLock::new(HashMap::<ChunkID, BitMap3D>::new()));
    let collider_render = collider.clone();

    let (mesh_updates_tx, mesh_updates_rx) =
        mpsc::new::<(ChunkID, MeshUpload)>(config.mesh_queue_cap);

    thread::Builder::new()
        .name("engine thread".to_owned())
        .spawn(move || -> Result<(), io::Error> {
            let worker_count = (num_cpus::get() - 2).min(config.worker_count).max(1); // minus main + engine thread

            let mut working_class_people = TaskSubmitter::new();

            let (chunk_tx, chunk_submission_queue) =
                mpsc::new::<(ChunkID, Chunk)>(config.chunk_queue_cap);

            let mut submitted_chunks: HashSet<ChunkID> = HashSet::with_capacity(10_000);
            let (discarded_tasks_tx, discarded_tasks_queue) =
                mpsc::new(config.discarded_tasks_queue_cap);

            let (collider_tx, collider_submission_queue) = mpsc::new(config.collider_queue_cap);

            let (solid_maps_tx, solid_map_queue) =
                mpsc::new::<(ChunkID, Box<[BitMap2D; 6]>)>(config.solid_map_queue_cap);

            let threadpool = Threadpool::new(worker_count, |_| task::Context {
                task_queues: working_class_people.add_worker(config.task_queue_cap, MAX_LOD),
                player_pos: player.clone(),
                full_detail_distance: config.full_detail_distance,

                world_generator: world_generator.clone(),

                discarded_tasks: discarded_tasks_tx.clone(),

                chunk_tx: chunk_tx.clone(),
                collider_tx: collider_tx.clone(),
                solid_map_tx: solid_maps_tx.clone(),

                meshes: mesh_updates_tx.clone(),
            })?;

            let mut sphere_generator_allocations =
                SphereGeneratorAllocations::default(config.max_chunks);
            let mut players_last_pos = None;

            let mut chunks: HashMap<ChunkID, Chunk> = HashMap::with_capacity(10_000);

            let mut solid_maps: [HashMap<ChunkID, BitMap2D>; 6] = [
                HashMap::with_capacity(10_000),
                HashMap::with_capacity(10_000),
                HashMap::with_capacity(10_000),
                HashMap::with_capacity(10_000),
                HashMap::with_capacity(10_000),
                HashMap::with_capacity(10_000),
            ];
            print_info!("setup complete");

            let mut time_window = Instant::now();
            let mut tick_count = 0_usize;
            'tick_loop: loop {
                // update configs
                while let Ok(update) = updates_recv.pop() {
                    use Update::*;
                    match update {
                        ConfigUpdate { update } => config.update(update),
                        ShutDown => break 'tick_loop,
                    }
                }

                // submit chunk generation tasks
                let player_pos = { player.read().pos() / 32. }.round();
                if Some(player_pos) != players_last_pos {
                    players_last_pos = Some(player_pos);

                    sphere_generator_allocations.flood_fill(
                        player_pos,
                        config.full_detail_distance,
                        config.total_generation_distance,
                        config.max_chunks,
                        |chunk| {
                            if submitted_chunks.insert(chunk) {
                                let mut axis = 0;
                                working_class_people.submit_task(
                                    chunk,
                                    Task::GenerateChunkAndMesh {
                                        chunk,
                                        neighbors: Box::new(chunk_neighbors(chunk).map(
                                            |neighbor| {
                                                let solid_map = solid_maps[axis]
                                                    .get(&neighbor)
                                                    .unwrap_or(&[0_u32; 32])
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
                    solid_maps[3].insert(chunk, solid_map[3]);
                    solid_maps[4].insert(chunk, solid_map[4]);
                    solid_maps[5].insert(chunk, solid_map[5]);
                }

                {
                    let mut collider = collider.write();
                    while let Ok((chunk, submission)) = collider_submission_queue.pop() {
                        collider.insert(chunk, *submission);
                    }
                }

                while let Ok(chunk) = discarded_tasks_queue.pop() {
                    submitted_chunks.remove(&chunk);
                }

                // tick measurement
                tick_count += 1;
                let time_elapsed = time_window.elapsed().as_secs_f64();
                if time_elapsed >= 1. {
                    if config.print_tps {
                        print_info!(
                            "tps  {}\tqueued-tasks  {}",
                            (tick_count as f64 / time_elapsed).round() as usize,
                            working_class_people.len()
                        );
                    }
                    tick_count = 0;
                    time_window = Instant::now();
                }
            }
            print_info!("SHUTDOWN");

            drop(threadpool);
            Ok(())
        })?;

    Ok(RenderThreadChannels {
        updates,
        player: player_render,
        voxel_collider: collider_render,
        mesh_updates: mesh_updates_rx,
    })
}
