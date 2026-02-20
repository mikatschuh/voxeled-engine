use glam::{IVec3, Vec3};

use std::{
    cmp::Reverse,
    collections::{HashSet, VecDeque},
    time::Instant,
    vec::IntoIter,
};

use crate::chunk::ChunkID;

pub type LodLevel = u16;

pub const MAX_LOD: LodLevel = 8;

#[allow(unused)]
pub fn cube(edges: i32, lod_level: LodLevel) -> IntoIter<ChunkID> {
    let mut chunk_ids = vec![];
    for x in 0..edges >> lod_level {
        for y in 0..edges >> lod_level {
            for z in 0..edges >> lod_level {
                chunk_ids.push(ChunkID::new(lod_level, IVec3::new(x, y, z)))
            }
        }
    }
    chunk_ids.into_iter()
}

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
        let now = Instant::now();

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
                        let lod = lod_level_at(
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
        println!("flood_fill: {}", now.elapsed().as_micros());

        chunks
    }

    pub fn chunk_ids(self) -> IntoIter<ChunkID> {
        let cam_chunk_pos = self.cam_pos / 32.0;

        let chunks = every_chunk_in_frustum(
            cam_chunk_pos,
            self.direction,
            self.fov,
            self.aspect_ratio,
            self.max_distance,
        );

        let mut candidates: Vec<ChunkID> = Vec::with_capacity(chunks.len());
        for chunk_pos in chunks {
            let lod = lod_level_at(self.full_detail_range, cam_chunk_pos, chunk_pos.as_vec3());
            let lod_shift = lod as i32;
            let lod_pos = IVec3::new(
                chunk_pos.x >> lod_shift,
                chunk_pos.y >> lod_shift,
                chunk_pos.z >> lod_shift,
            );

            candidates.push(ChunkID::new(lod, lod_pos));
        }
        candidates.sort_by_key(|candidate| Reverse(candidate.lod));

        let mut chunk_ids_set: HashSet<ChunkID> = HashSet::with_capacity(candidates.len());
        for candidate in candidates {
            if chunk_ids_set.contains(&candidate) {
                continue;
            }

            let mut ancestor = candidate;
            let mut has_coarser = false;
            while ancestor.lod < MAX_LOD {
                ancestor = ancestor.parent();
                if chunk_ids_set.contains(&ancestor) {
                    has_coarser = true;
                    break;
                }
            }

            if has_coarser {
                continue;
            }

            chunk_ids_set.insert(candidate);
        }

        let mut chunk_ids: Vec<ChunkID> = chunk_ids_set.into_iter().collect();
        chunk_ids.sort_by(|a, b| {
            (a.pos << a.lod)
                .as_vec3()
                .distance_squared(cam_chunk_pos)
                .total_cmp(&(b.pos << b.lod).as_vec3().distance_squared(cam_chunk_pos))
        });

        chunk_ids.into_iter()
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

fn lod_level_at(full_detail_range: f32, cam_chunk_pos: Vec3, chunk_coord: Vec3) -> LodLevel {
    let dst = cam_chunk_pos.distance(chunk_coord);
    match dst {
        dst if dst <= full_detail_range => 0,
        dst if dst <= full_detail_range * 2. => 1,
        dst if dst <= full_detail_range * 4. => 2,
        dst if dst <= full_detail_range * 8. => 3,
        dst if dst <= full_detail_range * 16. => 4,
        dst if dst <= full_detail_range * 32. => 5,
        dst if dst <= full_detail_range * 64. => 6,
        dst if dst <= full_detail_range * 128. => 7,
        _ => MAX_LOD,
    }
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

fn every_chunk_in_frustum(
    position: Vec3,
    direction: Vec3,
    fov: f32,
    aspect_ratio: f32,
    render_distance: f32,
) -> Vec<IVec3> {
    let mut points = Vec::new();

    let forward = if direction.length_squared() > 0.0 {
        direction.normalize()
    } else {
        Vec3::Z
    };
    let right = if forward.y.abs() > 0.999 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::Y.cross(forward).normalize()
    };
    let up = forward.cross(right).normalize();

    let tan_half_fov = (fov / 2.0).tan();
    let max_distance = render_distance.max(0.0);
    let bounds = max_distance.ceil() as i32 + 1;
    let chunk_pad = 0.5;

    let min = IVec3::new(
        (position.x.floor() as i32) - bounds,
        (position.y.floor() as i32) - bounds,
        (position.z.floor() as i32) - bounds,
    );
    let max = IVec3::new(
        (position.x.ceil() as i32) + bounds,
        (position.y.ceil() as i32) + bounds,
        (position.z.ceil() as i32) + bounds,
    );

    for z in min.z..=max.z {
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let delta = Vec3::new(x as f32, y as f32, z as f32) - position;

                let view_x = delta.dot(right);
                let view_y = delta.dot(up);
                let view_z = delta.dot(forward);

                if view_z < -chunk_pad || view_z > max_distance + chunk_pad {
                    continue;
                }

                let frustum_half_height = view_z * tan_half_fov + chunk_pad;
                let frustum_half_width = frustum_half_height * aspect_ratio + chunk_pad;

                if view_x.abs() > frustum_half_width || view_y.abs() > frustum_half_height {
                    continue;
                }

                points.push(IVec3::new(x, y, z));
            }
        }
    }

    points
}
