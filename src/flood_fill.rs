use std::collections::{HashSet, VecDeque};

use glam::{IVec3, Vec3};

use crate::{ChunkID, engine::LodLevel};

pub struct SphereConfig {
    pub full_detail_range: f32,
    pub radius: f32, // in chunks (32*32*32)
    pub max_chunks: usize,
}

pub struct SphereGeneratorAllocations {
    pub already_queued: HashSet<ChunkID>,
    pub candidates: VecDeque<ChunkID>,
    pub next_lod_candidates: VecDeque<ChunkID>,
}

impl SphereGeneratorAllocations {
    pub fn new(max_chunks: usize) -> Self {
        Self {
            already_queued: HashSet::with_capacity(max_chunks * 2),
            candidates: VecDeque::with_capacity(max_chunks * 2),
            next_lod_candidates: VecDeque::with_capacity(max_chunks * 2),
        }
    }
}

impl SphereConfig {
    pub fn flood_fill(
        &self,
        center: Vec3,
        buffers: &mut SphereGeneratorAllocations,
        mut out: impl FnMut(ChunkID),
    ) {
        let mut chunk_count: usize = 0;
        if self.max_chunks == 0 {
            return;
        }

        buffers.already_queued.clear();
        buffers.candidates.clear();

        let base_chunk = ChunkID::from_pos(center, 0);
        buffers.candidates.push_back(base_chunk);

        while let Some(chunk) = buffers.candidates.pop_front() {
            if (chunk.total_pos().as_vec3() + 0.5 * (1 << chunk.lod) as f32).distance(center)
                < self.radius
            {
                out(chunk);
                chunk_count += 1;
                if chunk_count >= self.max_chunks {
                    break;
                }

                for neighbor in chunk_neighbors(chunk) {
                    if buffers.already_queued.insert(neighbor) {
                        let lod = lod_at_dst(
                            self.full_detail_range,
                            center,
                            (neighbor.total_pos() & !1).as_vec3(),
                        );
                        let parent = neighbor.parent();
                        if lod > chunk.lod && buffers.already_queued.insert(parent) {
                            buffers.next_lod_candidates.push_back(parent);
                        } else if lod == chunk.lod {
                            buffers.candidates.push_back(neighbor);
                        }
                    }
                }
            }

            if buffers.candidates.is_empty() {
                std::mem::swap(&mut buffers.candidates, &mut buffers.next_lod_candidates);
            }
        }
    }
}

pub fn chunk_neighbors(c: ChunkID) -> [ChunkID; 6] {
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

pub fn lod_at_dst(full_detail_range: f32, cam_chunk_pos: Vec3, chunk_coord: Vec3) -> LodLevel {
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
