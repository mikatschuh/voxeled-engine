use std::{collections::HashMap, sync::Arc, thread};

use glam::Vec3;
use parking_lot::RwLock;
use tokio::io;

use crate::{
    ChunkID, ComposeableGenerator, DeltaTime,
    cam_controller::CamController,
    data_structures::LockHashMap,
    flood_fill::{SphereConfig, SphereGeneratorAllocations},
    meshing::BitMap3D,
    mpsc_channel,
    physics::TCBody,
    spsc_channel,
    task::{self, Task},
    voxel::VoxelData3D,
    worker::Threadpool,
};

pub struct Player {
    body: TCBody,
    dir: Vec3,
    delta_time: DeltaTime,
}

pub enum ConfigUpdates {
    WorldGeneration(SphereConfig),
}

pub struct RenderThreadChannels {
    pub config_updates: spsc_channel::Sender<ConfigUpdates>,
    pub player: Arc<RwLock<CamController>>,
    pub voxel_collider: Arc<LockHashMap<ChunkID, BitMap3D>>,
    pub mesh_updates: mpsc_channel::Receiver<()>,
}

pub fn create_engine_thread(
    workers: usize,
    mut sphere_config: SphereConfig,
    player: CamController,
    world_generator: ComposeableGenerator,
) -> Result<RenderThreadChannels, io::Error> {
    let (config_updates, config_updates_recv) = spsc_channel(10);

    let player = Arc::new(RwLock::new(player));
    let player_render = player.clone();

    let collider = Arc::new(LockHashMap::<ChunkID, BitMap3D>::new());
    let collider_render = collider.clone();

    let (mesh_updates_producer, mesh_updates_recv) = mpsc_channel(10_000);

    thread::Builder::new()
        .name("engine thread".to_owned())
        .spawn(move || -> Result<(), io::Error> {
            let worker_count = (num_cpus::get() - 2).min(workers).max(1); // minus main + engine thread

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
                // update configss
                while let Ok(config_update) = config_updates_recv.try_recv() {
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
