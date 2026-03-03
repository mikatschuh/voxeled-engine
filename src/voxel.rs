use crate::{VoxelType, chunk::CHUNK_VOLUME, mesh::TextureID};

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelTypes {
    Air = 1,
    CrackedStone,
    Stone,
    Dirt,
}

pub fn is_physically_solid(voxel: VoxelType) -> bool {
    voxel != VoxelTypes::Air as u16
}

pub fn is_physically_solid_u32(voxel: VoxelType) -> u32 {
    if voxel != VoxelTypes::Air as u16 {
        0b1000_0000_0000_0000_0000_0000_0000_0000
    } else {
        0
    }
}

pub fn is_solid_u32(voxel: VoxelType) -> u32 {
    if voxel != VoxelTypes::Air as u16 {
        0b1000_0000_0000_0000_0000_0000_0000_0000
    } else {
        0
    }
}

#[allow(unused)]
/// orientations
/// 0 = -x
/// 1 = +x
/// 2 = -y
/// 3 = +y
/// 4 = -z
/// 5 = +z
pub fn texture_id(voxel: VoxelType, orientation: u8) -> TextureID {
    voxel as u16 - 2
}

pub fn fill(fill: VoxelType) -> [VoxelType; CHUNK_VOLUME] {
    [fill; _]
}
