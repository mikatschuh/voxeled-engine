use std::mem::MaybeUninit;

use glam::IVec3;

use crate::server::{
    chunks::{Chunk, ChunkID, Level},
    voxel::{VoxelChunk, VoxelData3D, VoxelType},
};

fn down_sample(chunk_id: ChunkID, level: &Level) {
    if chunk_id.lod == 0 || level.contains(chunk_id) {
        return;
    }

    let child_lod = chunk_id.lod - 1;
    let child_base = chunk_id.pos << 1;

    let air_chunk = [[[VoxelType::Air; 32]; 32]; 32];
    let mut sub_chunks = [[[air_chunk; 2]; 2]; 2];

    for chunk_x in 0..2 {
        for chunk_y in 0..2 {
            for chunk_z in 0..2 {
                let child_id = ChunkID::new(
                    child_lod,
                    child_base + IVec3::new(chunk_x as i32, chunk_y as i32, chunk_z as i32),
                );

                if !level.contains(child_id) {
                    if child_lod > 0 {
                        down_sample(child_id, level);
                    } else {
                        level.insert(child_id, Chunk::VoxelChunk::new(VoxelType::Air));
                    }
                }

                if let Some(chunk) = level.get(child_id) {
                    sub_chunks[chunk_x][chunk_y][chunk_z] = chunk.read().unwrap().data;
                }
            }
        }
    }

    let mut combined_data: MaybeUninit<VoxelData3D> = MaybeUninit::uninit();
    let combined_data_ref = unsafe { combined_data.assume_init_mut() };

    let mut is_empty = true;
    for x in 0..32 {
        let src_x_base = x << 1;
        for y in 0..32 {
            let src_y_base = y << 1;
            for z in 0..32 {
                let src_z_base = z << 1;
                let mut stone_count = 0u8;
                let mut dirt_count = 0u8;

                for ox in 0..2 {
                    let src_x = src_x_base + ox;
                    let child_x = src_x >> 5;
                    let local_x = src_x & 31;
                    for oy in 0..2 {
                        let src_y = src_y_base + oy;
                        let child_y = src_y >> 5;
                        let local_y = src_y & 31;
                        for oz in 0..2 {
                            let src_z = src_z_base + oz;
                            let child_z = src_z >> 5;
                            let local_z = src_z & 31;

                            match sub_chunks[child_x][child_y][child_z][local_x][local_y][local_z] {
                                VoxelType::Stone => stone_count += 1,
                                VoxelType::Dirt => dirt_count += 1,
                                VoxelType::Air => {}
                            }
                        }
                    }
                }

                let voxel = if stone_count == 0 && dirt_count == 0 {
                    VoxelType::Air
                } else if stone_count >= dirt_count {
                    VoxelType::Stone
                } else {
                    VoxelType::Dirt
                };

                if voxel != VoxelType::Air {
                    is_empty = false;
                }
                combined_data_ref[x][y][z] = voxel;
            }
        }
    }

    level.insert(
        chunk_id,
        VoxelChunk {
            is_empty,
            data: unsafe { combined_data.assume_init() },
        },
    );
}
