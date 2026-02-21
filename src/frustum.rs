use glam::{IVec3, Vec3};

use std::collections::{HashSet, VecDeque};

use crate::chunk::ChunkID;

pub type LodLevel = u16;

/// Chunks are 32^3
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub cam_pos: Vec3,
    pub direction: Vec3,

    pub fov: f32,
    pub aspect_ratio: f32,

    pub max_chunks: usize,
    pub max_distance: f32,
    pub full_detail_range: f32,
}

impl Frustum {
    pub fn flood_fill(self) -> Vec<ChunkID> {
        if self.max_chunks == 0 {
            return Vec::new();
        }

        let mut chunks: Vec<ChunkID> = Vec::with_capacity(self.max_chunks);

        let cam_pos = self.cam_pos / 32.0;

        let forward = if self.direction.length_squared() > 0.0 {
            self.direction.normalize()
        } else {
            Vec3::Z
        };
        let right = if forward.y.abs() > 0.999 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::Y.cross(forward).normalize()
        };
        let up = forward.cross(right).normalize();

        let tan_half_fov = (self.fov * 0.5).tan();
        let tan_half_fov_x = tan_half_fov * self.aspect_ratio;
        let max_distance = self.max_distance.max(0.0);

        let in_frustum = |c: ChunkID| -> bool {
            let size = c.size();
            let center = c.total_pos().as_vec3() + Vec3::splat(size * 0.5);
            let delta = center - cam_pos;
            let half_extent = size * 0.5;

            let outside_plane = |normal: Vec3, offset: f32| {
                let signed_center = delta.dot(normal) + offset;
                let projected_radius =
                    half_extent * (normal.x.abs() + normal.y.abs() + normal.z.abs());
                (signed_center - projected_radius) > 0.0
            };

            let near_normal = -forward;
            let far_normal = forward;
            let left_normal = -right - forward * tan_half_fov_x;
            let right_normal = right - forward * tan_half_fov_x;
            let bottom_normal = -up - forward * tan_half_fov;
            let top_normal = up - forward * tan_half_fov;

            !outside_plane(near_normal, 0.0)
                && !outside_plane(far_normal, -max_distance)
                && !outside_plane(left_normal, 0.0)
                && !outside_plane(right_normal, 0.0)
                && !outside_plane(bottom_normal, 0.0)
                && !outside_plane(top_normal, 0.0)
        };

        let mut already_queued: HashSet<ChunkID> = HashSet::with_capacity(self.max_chunks * 2);
        let mut candidates: VecDeque<ChunkID> = VecDeque::with_capacity(self.max_chunks * 2);

        let base_chunk = ChunkID::from_pos(cam_pos, 0);
        candidates.push_back(base_chunk);
        already_queued.insert(base_chunk);

        let mut next_lods_candidates: VecDeque<ChunkID> =
            VecDeque::with_capacity(self.max_chunks * 2);

        while let Some(chunk) = candidates.pop_front() {
            if in_frustum(chunk) {
                chunks.push(chunk);
                if chunks.len() >= self.max_chunks {
                    break;
                }

                for neighbor in chunk_neighbors(chunk) {
                    if already_queued.insert(neighbor) {
                        let lod = lod_at_dst(
                            self.full_detail_range,
                            cam_pos,
                            (neighbor.total_pos() & !1).as_vec3(),
                        );
                        let parent = neighbor.parent();
                        if lod > chunk.lod && already_queued.insert(parent) {
                            next_lods_candidates.push_back(parent);
                        } else if lod == chunk.lod {
                            candidates.push_back(neighbor);
                        }
                    }
                }
            }

            if candidates.is_empty() {
                std::mem::swap(&mut candidates, &mut next_lods_candidates);
            }
        }

        chunks
    }
}

fn chunk_neighbors(c: ChunkID) -> [ChunkID; 6] {
    let pos = c.pos;
    [
        pos + IVec3::NEG_X,
        pos + IVec3::X,
        pos + IVec3::NEG_Y,
        pos + IVec3::Y,
        pos + IVec3::NEG_Z,
        pos + IVec3::Z,
    ]
    .map(|p| ChunkID::new(c.lod, p))
}

fn lod_at_dst(full_detail_range: f32, cam_chunk_pos: Vec3, chunk_coord: Vec3) -> LodLevel {
    let dst = cam_chunk_pos.distance(chunk_coord);
    (dst / full_detail_range).ceil().log2().ceil().min(65535.) as u16
}

pub fn chunk_overlaps(a: &ChunkID, b: ChunkID) -> bool {
    if a.lod == b.lod {
        return a.pos == b.pos;
    }

    if a.lod > b.lod {
        let shift = (a.lod - b.lod) as i32;
        return (b.pos >> shift) == a.pos;
    }

    let shift = (b.lod - a.lod) as i32;
    (a.pos >> shift) == b.pos
}
