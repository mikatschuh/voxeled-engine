use std::mem::MaybeUninit;

use crate::VoxelType;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub type BitMap3D = [[u32; 32]; 32];
