use std::sync::Arc;

use glam::UVec3;
use parking_lot::RwLock;

use crate::{
    Chunk, ChunkID, ComposableGenerator, Generator,
    cam_controller::CamController,
    chunk::{DenseChunk, idx_to_coord},
    flood_fill::lod_at_dst,
    mesh::MeshUpload,
    meshing::{
        BitMap2D, BitMap3D, generate_mesh, get_axis_aligned_solid_maps, get_edges, map_visible,
    },
    mpsc, voxel,
    worker_pool::Runable,
};

#[derive(Debug)]
pub struct Context {
    pub task_queues: Vec<rtrb::Consumer<Task>>,
    pub player_pos: Arc<RwLock<CamController>>,
    pub full_detail_distance: f32,

    pub world_generator: ComposableGenerator,

    pub discarded_tasks: mpsc::Sender<ChunkID>,

    pub chunk_tx: mpsc::Sender<(ChunkID, Chunk)>,
    pub collider_tx: mpsc::Sender<(ChunkID, Box<BitMap3D>)>,
    pub solid_map_tx: mpsc::Sender<(ChunkID, Box<[BitMap2D; 6]>)>,

    pub meshes: mpsc::Sender<(ChunkID, MeshUpload)>,
}

#[derive(Debug)]
pub enum Task {
    GenerateChunkAndMesh {
        chunk: ChunkID,
        neighbors: Box<[BitMap2D; 6]>,
    },
}

impl Runable for Context {
    fn run_task(&mut self) -> bool {
        let mut task_queues = self.task_queues.iter_mut();

        let task = loop {
            let Some(task_queue) = task_queues.next() else {
                return false;
            };

            if let Ok(task) = task_queue.pop() {
                break task;
            }
        };

        use Task::*;
        match task {
            GenerateChunkAndMesh { chunk, neighbors } => self.generate_chunk(chunk, neighbors),
        }
        true
    }
}

impl Context {
    pub fn generate_chunk(&mut self, chunk: ChunkID, neighbors: Box<[BitMap2D; 6]>) {
        if lod_at_dst(
            self.full_detail_distance,
            self.player_pos.read().pos() / 32.,
            chunk.total_pos().as_vec3(),
        ) > chunk.lod + 1
        {
            self.discarded_tasks
                .push(chunk)
                .expect("the discarded task submission queue is full (shouldn't)");
            return;
        }

        let data = self.world_generator.generate(chunk);

        let collider = Box::new(get_z_aligned_collider(&data));
        self.collider_tx
            .push((chunk, collider))
            .expect("the collider submission queue is full (shouldn't)");

        let solid_maps = Box::new(get_axis_aligned_solid_maps(&data));
        let mesh = generate_mesh(&data, map_visible(&solid_maps, &neighbors));

        self.meshes
            .push((chunk, mesh.bytes()))
            .expect("the mesh submission queue is full (shouldn't)");

        self.solid_map_tx
            .push((chunk, Box::new(get_edges(*solid_maps))))
            .expect("the solid map submission queue is full (shouldn't)");

        if chunk.lod == 0 {
            self.chunk_tx
                .push((chunk, Chunk::from_buffer(&data)))
                .expect("the chunk submission queue is full (shouldn't)");
        }
    }
}

fn get_z_aligned_collider(data: &DenseChunk) -> BitMap3D {
    let mut z_aligned = [[0; 32]; 32];

    // data setup
    for (i, voxel) in data.iter().enumerate() {
        let UVec3 { x, y, z } = idx_to_coord(i);

        let voxel_is_solid_u32 = voxel::is_physically_solid_u32(*voxel);

        if voxel_is_solid_u32 > 0 {
            z_aligned[x as usize][y as usize] |= voxel_is_solid_u32 >> z;
        }
    }
    z_aligned
}
