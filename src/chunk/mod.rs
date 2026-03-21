mod chunk_format;

pub type VoxelType = u16;

pub use chunk_format::Chunk;
use glam::{IVec3, UVec3, Vec3};

/// Starting at `0`. `LOD0` is the highest resolution, where each individual chunk represents the smallest region of space.
pub type Lod = u16;

/// A struct whiches values map to every possible chunk, including LOD.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkID {
    /// In Chunk & `LOD` space. That means that for a chunk with higher `LOD` one unit does represent `2^LOD` of chunks.
    pub pos: IVec3,
    pub lod: Lod,
}

impl ChunkID {
    pub fn new(lod: Lod, pos: IVec3) -> Self {
        Self { lod, pos }
    }

    pub fn total_pos(&self) -> IVec3 {
        self.pos << self.lod
    }

    pub fn center(&self) -> Vec3 {
        self.total_pos().as_vec3() + Vec3::splat(0.5) * (1 << self.lod) as f32
    }

    pub fn parent(&self) -> Self {
        Self {
            lod: self.lod + 1,
            pos: self.pos >> 1,
        }
    }

    pub fn from_pos(v: Vec3, lod: Lod) -> Self {
        Self {
            lod,
            pos: v.floor().as_ivec3(),
        }
    }

    pub fn bytes(&self) -> [u32; 4] {
        bytemuck::cast([
            self.pos.x.cast_unsigned(),
            self.pos.y.cast_unsigned(),
            self.pos.z.cast_unsigned(),
            self.lod as u32,
        ])
    }
}

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

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
