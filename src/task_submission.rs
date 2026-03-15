use rtrb::PushError;

use crate::{ChunkID, flood_fill::MAX_LOD, task::Task, worker::WorkerID};

pub struct TaskSubmitter {
    queues: Vec<Vec<rtrb::Producer<Task>>>,
}

impl TaskSubmitter {
    pub fn new() -> Self {
        Self { queues: vec![] }
    }

    pub fn add_worker(&mut self, cap: usize) -> Vec<rtrb::Consumer<Task>> {
        let (txs, rxs) = (0..MAX_LOD).map(|_| rtrb::RingBuffer::new(cap)).unzip();
        self.queues.push(txs);
        rxs
    }

    pub fn submit_task(&mut self, chunk: ChunkID, mut task: Task) {
        let bucket = bucket(chunk, self.queues.len());
        let queue: &mut rtrb::Producer<Task> = &mut self.queues[bucket][chunk.lod as usize];

        loop {
            match queue.push(task) {
                Ok(()) => return,
                Err(PushError::Full(t)) => task = t,
            }
            std::hint::spin_loop();
        }
    }

    pub fn len(&self) -> usize {
        self.queues
            .iter()
            .flat_map(|queue| queue.iter())
            .map(|b| b.buffer().capacity() - b.slots())
            .sum()
    }
}

/// Maps integer 3D coords to `0..values-1`.
/// - Cheap integer math only
/// - For any `values x values x values` region aligned to that grid,
///   each value appears exactly `values * values` times.
#[inline]
pub fn bucket(chunk: ChunkID, values: usize) -> WorkerID {
    let p = chunk.pos;

    assert!(values > 0, "values must be > 0");
    if values == 1 {
        return 0;
    }

    let n = values as i32;

    // Balanced base pattern: x + y + z mod n
    let base = (p.x + p.y + p.z).rem_euclid(n);

    // Coarse cell id (cell size = n in each axis)
    let cx = p.x.div_euclid(n);
    let cy = p.y.div_euclid(n);
    let cz = p.z.div_euclid(n);

    // Cheap per-cell hash -> offset in 0..n-1
    let mut h = (cx as usize).wrapping_mul(0x9E37_79B1)
        ^ (cy as usize).wrapping_mul(0x85EB_CA77)
        ^ (cz as usize).wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;

    let shift = (h % values) as i32;
    ((base + shift).rem_euclid(n)) as usize
}
