mod collision;
#[cfg(test)]
mod test;
mod verlet;

use glam::IVec3;
use glam::Vec3;

pub use verlet::Body;
pub use verlet::TCBody;

pub use collision::Aabb;
pub use collision::Voxel;

pub fn block(v: Vec3) -> IVec3 {
    v.floor().as_ivec3()
}

pub fn block_coord(n: f32) -> i32 {
    n.floor() as i32
}
