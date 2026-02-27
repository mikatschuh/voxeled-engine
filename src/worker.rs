use std::{
    fmt, io,
    marker::PhantomData,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{
    ChunkID,
    spsc_channel::{self, TrySendError},
};

use parking_lot::RwLock;

pub trait Runable<C> {
    fn run(self, debug_log: &mut Vec<String>, context: &mut C);
}

pub type WorkerID = usize;

#[derive(Debug)]
pub struct Threadpool<T: fmt::Debug + Runable<C>, C> {
    _phantom: PhantomData<C>,

    debug_log: Vec<Arc<RwLock<Vec<String>>>>, // for every worker
    queues: Vec<spsc_channel::Sender<T>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<T: fmt::Debug + Runable<C> + Send + 'static, C: Clone + Send + 'static> Threadpool<T, C> {
    pub fn new(num_of_workers: usize, context: C) -> Result<Self, io::Error> {
        let mut workers: Vec<thread::JoinHandle<()>> = Vec::with_capacity(num_of_workers);
        let mut debug_logs: Vec<Arc<RwLock<Vec<String>>>> = Vec::with_capacity(num_of_workers);
        let mut queues: Vec<spsc_channel::Sender<T>> = Vec::with_capacity(num_of_workers);

        for i in 0..num_of_workers {
            let (task_sender, task_recv) = crate::spsc_channel::<T>(1000);
            let debug_log = Arc::new(RwLock::new(vec![]));
            let debug_log_cloned = debug_log.clone();

            let mut context = context.clone();

            let worker = thread::Builder::new()
                .name(format!("worker {i}"))
                .spawn(move || {
                    let mut time_since_task = Instant::now();
                    loop {
                        if task_recv.is_disconnected() {
                            return;
                        }

                        while let Ok(task) = task_recv.try_recv() {
                            task.run(&mut debug_log.write(), &mut context);
                            time_since_task = Instant::now();
                        }

                        let time_since_task = time_since_task.elapsed().as_micros();
                        if time_since_task > 10 {
                            thread::sleep(Duration::from_micros(time_since_task.min(1000) as u64));
                        }
                    }
                })?;

            debug_logs.push(debug_log_cloned);
            queues.push(task_sender);
            workers.push(worker);
        }

        Ok(Self {
            _phantom: PhantomData::default(),

            debug_log: debug_logs,
            queues,
            workers,
        })
    }

    pub fn submit(&mut self, worker: WorkerID, mut task: T) {
        loop {
            match self.queues[worker].try_send(task) {
                Ok(_) => return,
                Err(e) => match e {
                    TrySendError::Full(t) => task = t,
                    TrySendError::Disconnected(_) => panic!("worker died!"),
                },
            }
        }
    }

    pub fn submit_with_chunk(&mut self, chunk: ChunkID, mut task: T) {
        loop {
            match self.queues[bucket(chunk, self.workers.len())].try_send(task) {
                Ok(_) => return,
                Err(e) => match e {
                    TrySendError::Full(t) => task = t,
                    TrySendError::Disconnected(_) => panic!("worker died!"),
                },
            }
        }
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
