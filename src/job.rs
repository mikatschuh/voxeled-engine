use std::{sync::Arc, thread};

use crossbeam::sync::ShardedLock;

use crate::{
    chunk::{Chunk, ChunkID, DataState, Level},
    mesh::Mesh,
    meshing::{generate_mesh, map_visible},
    world_gen::Generator,
};

pub enum Job<G: Generator> {
    GenerateChunk {
        voxel_grid: Arc<Level>,
        chunk_id: ChunkID,
        generator: Arc<ShardedLock<G>>,
    },

    GenerateMesh {
        voxel_grid: Arc<Level>,
        chunk_id: ChunkID,
    },

    GenerateChunkAndMesh {
        voxel_grid: Arc<Level>,
        chunk_id: ChunkID,
        generator: Arc<ShardedLock<G>>,
    },
}

impl<G: Generator> Job<G> {
    pub fn run(self, debug_log: &mut Vec<String>) {
        match self {
            Self::GenerateChunk {
                voxel_grid,
                chunk_id,
                generator,
            } => _ = Self::generate_chunk(voxel_grid, chunk_id, generator, debug_log),

            Self::GenerateMesh {
                voxel_grid,
                chunk_id,
            } => _ = Self::generate_mesh(voxel_grid, chunk_id, debug_log),

            Self::GenerateChunkAndMesh {
                voxel_grid,
                chunk_id,
                generator,
            } => _ = Self::generate_chunk_and_mesh(voxel_grid, chunk_id, generator, debug_log),
        }
    }

    fn generate_chunk(
        level: Arc<Level>,
        chunk_id: ChunkID,
        generator: Arc<ShardedLock<G>>,
        _debug_log: &mut Vec<String>,
    ) -> Option<()> {
        if level
            .insert(chunk_id, Chunk::new(DataState::Generating))
            .is_err()
        {
            return Some(());
        }

        println!(
            "generating chunk   thread {}, chunk_id: {chunk_id:?}",
            thread::current().name().unwrap(),
        );

        let voxel = generator.read().unwrap().generate(chunk_id);

        level.chunk_op(chunk_id, |chunk| chunk.write_voxel(voxel))
    }

    fn generate_mesh(
        level: Arc<Level>,
        chunk_id: ChunkID,
        _debug_log: &mut Vec<String>,
    ) -> Option<()> {
        if level
            .chunk_op(chunk_id, |chunk| chunk.occl_state.try_start_generating())?
            .is_err()
        {
            return Some(());
        };

        let occl_maps = map_visible(&level, chunk_id);

        level.chunk_op(chunk_id, |chunk| chunk.write_occl(occl_maps));

        if level
            .chunk_op(chunk_id, |chunk| chunk.mesh_state.try_start_generating())?
            .is_err()
        {
            return Some(());
        }

        println!(
            "generating mesh   thread {}, chunk_id: {chunk_id:?}",
            thread::current().name().unwrap(),
        );

        let voxel = level.chunk_op(chunk_id, |chunk| *chunk.voxel.read())?;
        let mesh = voxel.map_or_else(Mesh::new, |voxel| generate_mesh(chunk_id, voxel, occl_maps));

        level.chunk_op(chunk_id, |chunk| chunk.write_mesh(mesh))
    }

    fn generate_chunk_and_mesh(
        level: Arc<Level>,
        chunk_id: ChunkID,
        generator: Arc<ShardedLock<G>>,
        debug_log: &mut Vec<String>,
    ) -> Option<()> {
        if level
            .insert(chunk_id, Chunk::new(DataState::Generating))
            .is_err()
        {
            return Some(());
        }

        debug_log.push(format!("L{} {}", chunk_id.lod, chunk_id.pos));

        let voxel = generator.read().unwrap().generate(chunk_id);

        level.chunk_op(chunk_id, |chunk| chunk.write_voxel(voxel))?;

        if level
            .chunk_op(chunk_id, |chunk| chunk.occl_state.try_start_generating())?
            .is_err()
        {
            return Some(());
        };

        let occl_maps = map_visible(&level, chunk_id);

        level.chunk_op(chunk_id, |chunk| chunk.write_occl(occl_maps));

        if level
            .chunk_op(chunk_id, |chunk| chunk.mesh_state.try_start_generating())?
            .is_err()
        {
            return Some(());
        }

        let mesh = generate_mesh(chunk_id, voxel, occl_maps);

        level.chunk_op(chunk_id, |chunk| chunk.write_mesh(mesh))
    }
}
