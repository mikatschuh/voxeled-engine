use glam::Vec3;

use std::{
    any::Any,
    collections::{HashMap, HashSet, VecDeque},
};

use crate::{
    engine::ChunkID,
    flood_fill::{chunk_neighbors, lod_at_dst},
};

/// Chunks are 32^3
#[derive(Debug, Clone)]
pub struct Frustum {
    pub cam_pos: Vec3,
    pub direction: Vec3,

    pub fov: f32,
    pub aspect_ratio: f32,

    pub max_chunks: usize,
    pub max_distance: f32,
    pub full_detail_range: f32,
}

#[derive(Debug, Clone)]
pub struct FrustumAllocations {
    pub chunks: Vec<ChunkID>,
    pub ready_chunks: Vec<ChunkID>,
    pub already_queued: HashSet<ChunkID>,
    pub candidates: VecDeque<ChunkID>,
    pub next_lod_candidates: VecDeque<ChunkID>,
}

impl FrustumAllocations {
    pub fn default(max_chunks: usize) -> Self {
        Self {
            chunks: Vec::with_capacity(max_chunks),
            ready_chunks: Vec::with_capacity(max_chunks),
            already_queued: HashSet::with_capacity(max_chunks * 2),
            candidates: VecDeque::with_capacity(max_chunks * 2),
            next_lod_candidates: VecDeque::with_capacity(max_chunks * 2),
        }
    }
}

impl Frustum {
    pub fn flood_fill<'a>(
        self,
        buffers: &'a mut FrustumAllocations,
        ready_meshes: &HashMap<ChunkID, impl Any>,
    ) -> &'a [ChunkID] {
        if self.max_chunks == 0 {
            return &[];
        }

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
            let size = (1 << c.lod) as f32;
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

        buffers.already_queued.clear();
        buffers.candidates.clear();
        buffers.next_lod_candidates.clear();
        buffers.chunks.clear();

        let base_chunk = ChunkID::from_pos(cam_pos, 0);
        buffers.candidates.push_back(base_chunk);
        buffers.already_queued.insert(base_chunk);

        while let Some(chunk) = buffers.candidates.pop_front() {
            if in_frustum(chunk) {
                buffers.chunks.push(chunk);
                if buffers.chunks.len() >= self.max_chunks {
                    break;
                }

                for neighbor in chunk_neighbors(chunk) {
                    if buffers.already_queued.insert(neighbor) {
                        let lod = lod_at_dst(
                            self.full_detail_range,
                            cam_pos,
                            (neighbor.total_pos() & !1).as_vec3(),
                        );
                        let parent = neighbor.parent();
                        if lod == chunk.lod {
                            buffers.candidates.push_back(neighbor);
                        } else if lod > chunk.lod && buffers.already_queued.insert(parent) {
                            buffers.next_lod_candidates.push_back(parent);
                        }
                    }
                }
            }

            if buffers.candidates.is_empty() {
                std::mem::swap(&mut buffers.candidates, &mut buffers.next_lod_candidates);
            }
        }
        buffers.ready_chunks.clear();
        select_render_chunks(&buffers.chunks, ready_meshes, &mut buffers.ready_chunks);

        &buffers.ready_chunks
    }
}

fn select_render_chunks(
    chunks: &[ChunkID],
    ready_meshes: &HashMap<ChunkID, impl Any>,
    ready_chunks: &mut Vec<ChunkID>,
) {
    for desired in chunks.iter().copied() {
        let mut candidate = desired;
        if !ready_meshes.contains_key(&candidate) {
            let mut next = candidate;
            let mut found = false;

            let mut iterations = 0_usize;
            while iterations <= 5 {
                next = next.parent();
                if ready_meshes.contains_key(&next) {
                    candidate = next;
                    found = true;
                    break;
                }
                iterations += 1;
            }
            if !found && !ready_meshes.contains_key(&candidate) {
                continue;
            }
        }

        ready_chunks.retain(|existing| {
            !(chunk_overlaps(existing, candidate) && existing.lod < candidate.lod)
        });
        if ready_chunks
            .iter()
            .any(|existing| chunk_overlaps(existing, candidate) && existing.lod <= candidate.lod)
        {
            continue;
        }
        ready_chunks.push(candidate);
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
