use crossbeam::atomic::AtomicCell;
use std::{sync::Arc, time::Instant};

/// A structure, which is able to store the exact time of the last update.
pub struct DeltaTimeMeter {
    last_update: Instant,
    current: DeltaTime,
}

impl DeltaTimeMeter {
    /// Creates a new delta-time instance.
    /// The instance will
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            current: DeltaTime(Arc::new(AtomicCell::new(0))),
        }
    }

    /// Logs a new delta-time.
    pub fn update(&mut self) {
        self.current
            .0
            .store(self.last_update.elapsed().as_nanos() as u64);
        self.last_update = Instant::now()
    }

    /// Creates a new delta-time-reader which is able to read out and obtain the current delta-time.
    pub fn reader(&self) -> DeltaTime {
        self.current.clone()
    }
}

#[derive(Clone)]
pub struct DeltaTime(Arc<AtomicCell<u64>>);

impl DeltaTime {
    /// returns the delta-time in seconds
    pub fn get_f32(&self) -> f32 {
        self.0.load() as f32 / 1_000_000_000.0
    }

    pub fn get_f64(&self) -> f64 {
        self.0.load() as f64 / 1_000_000_000.0
    }
}

use std::fmt;
impl fmt::Debug for DeltaTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_f32())
    }
}
