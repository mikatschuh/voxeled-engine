mod chunk_format;

pub type VoxelType = u16;

pub use chunk_format::Chunk;
use glam::UVec3;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub type BitMap3D = [[u32; 32]; 32];

pub type DenseChunk = [VoxelType; CHUNK_VOLUME];

#[inline(always)]
pub fn coords_to_1d_index(coord: UVec3) -> usize {
    (coord.x * 32 * 32 + coord.y * 32 + coord.z) as usize
}

pub fn idx_to_coord(i: usize) -> UVec3 {
    let x = i / (32 * 32);
    let yz = i % (32 * 32);
    let y = yz / 32;
    let z = yz % 32;
    UVec3::new(x as u32, y as u32, z as u32)
}
