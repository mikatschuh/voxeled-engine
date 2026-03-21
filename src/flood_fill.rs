use std::collections::{HashSet, VecDeque};

use glam::{IVec3, Vec3};

use crate::{ChunkID, chunk::lod_at_dst};

pub struct SphereGeneratorAllocations {
    pub touched: HashSet<ChunkID>,
    pub candidates: VecDeque<ChunkID>,
    pub next_lod_candidates: VecDeque<ChunkID>,
}

impl SphereGeneratorAllocations {
    pub fn default(max_chunks: usize) -> Self {
        Self {
            touched: HashSet::with_capacity(max_chunks * 2),
            candidates: VecDeque::with_capacity(max_chunks * 2),
            next_lod_candidates: VecDeque::with_capacity(max_chunks * 2),
        }
    }
}

impl SphereGeneratorAllocations {
    pub fn flood_fill(
        &mut self,
        center: Vec3,
        lowest_lod_dst: f32,
        radius: f32,
        max_chunks: usize,
        mut out: impl FnMut(ChunkID),
    ) {
        let mut chunk_count: usize = 0;
        if max_chunks == 0 {
            return;
        }

        self.touched.clear();
        self.candidates.clear();

        let base_chunk = ChunkID::from_pos(center, 0);
        self.candidates.push_back(base_chunk);

        while let Some(chunk) = self.candidates.pop_front() {
            if (chunk.total_pos().as_vec3() + 0.5 * (1 << chunk.lod) as f32).distance(center)
                < radius
            {
                out(chunk);
                chunk_count += 1;
                if chunk_count >= max_chunks {
                    break;
                }

                for neighbor in chunk_neighbors(chunk) {
                    if self.touched.insert(neighbor) {
                        let parent = neighbor.parent();
                        let lod = lod_at_dst(lowest_lod_dst, center, parent.center());
                        if lod == chunk.lod {
                            self.candidates.push_back(neighbor);
                        } else if lod > chunk.lod && self.touched.insert(parent) {
                            self.next_lod_candidates.push_back(parent);
                        }
                    }
                }
            }

            if self.candidates.is_empty() {
                std::mem::swap(&mut self.candidates, &mut self.next_lod_candidates);
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
