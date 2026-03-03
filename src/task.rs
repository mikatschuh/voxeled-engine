use glam::UVec3;

use crate::{
    Chunk, ChunkID, ComposableGenerator, Generator, Mesh,
    chunk::{DenseChunk, idx_to_coord},
    meshing::{BitMap3D, generate_mesh, get_axis_aligned_solid_maps, map_visible},
    mpsc, voxel,
    worker::{RecvTask, Runable},
};

#[derive(Debug)]
pub struct Context {
    pub task_queue: rtrb::Consumer<Task>,

    pub world_generator: ComposableGenerator,

    pub chunk_tx: mpsc::Sender<(ChunkID, Chunk)>,
    pub collider_tx: mpsc::Sender<(ChunkID, Box<BitMap3D>)>,
    pub solid_map_tx: mpsc::Sender<(ChunkID, Box<[BitMap3D; 3]>)>,

    pub meshes: mpsc::Sender<Mesh>,
}

impl RecvTask<Task> for Context {
    fn recv_task(&mut self) -> Option<Task> {
        self.task_queue.pop().ok()
    }
}

#[derive(Debug)]
pub enum Task {
    GenerateChunkAndMesh {
        chunk: ChunkID,
        neighbors: Box<[BitMap3D; 6]>,
    },
}

impl Runable<Context> for Task {
    fn run(self, context: &mut Context) {
        use Task::*;
        match self {
            GenerateChunkAndMesh { chunk, neighbors } => generate_chunk(context, chunk, neighbors),
        }
    }
}

pub fn generate_chunk(context: &mut Context, chunk: ChunkID, neighbors: Box<[BitMap3D; 6]>) {
    let data = context.world_generator.generate(chunk);

    let collider = Box::new(get_z_aligned_collider(&data));
    context
        .collider_tx
        .push((chunk, collider))
        .expect("the collider submission queue is full (shouldn't)");

    let solid_maps = Box::new(get_axis_aligned_solid_maps(&data));
    let mesh = generate_mesh(chunk, &data, map_visible(&solid_maps, &neighbors));

    context
        .meshes
        .push(mesh)
        .expect("the mesh submission queue is full (shouldn't)");

    context
        .solid_map_tx
        .push((chunk, solid_maps))
        .expect("the solid map submission queue is full (shouldn't)");

    if chunk.lod == 0 {
        context
            .chunk_tx
            .push((chunk, Chunk::from_buffer(&data)))
            .expect("the chunk submission queue is full (shouldn't)");
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
