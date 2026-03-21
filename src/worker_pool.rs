use std::{
    io,
    marker::PhantomData,
    thread,
    time::{Duration, Instant},
};

pub trait Runable {
    fn execute_tasks(&mut self) -> bool; // returns true if it was able to do something
}

pub type WorkerID = usize;

#[derive(Debug)]
pub struct Threadpool<C: Runable> {
    _phantom: PhantomData<C>,
    _workers: Vec<thread::JoinHandle<()>>,
}

impl<C: Runable + Send + 'static> Threadpool<C> {
    pub fn new(
        num_of_workers: usize,
        mut context: impl FnMut(usize) -> C,
    ) -> Result<Self, io::Error> {
        let mut workers: Vec<thread::JoinHandle<()>> = Vec::with_capacity(num_of_workers);

        for i in 0..num_of_workers {
            let mut context = context(i);

            let worker = thread::Builder::new()
                .name(format!("worker {i}"))
                .spawn(move || {
                    let mut time_since_task = Instant::now();
                    loop {
                        if context.execute_tasks() {
                            time_since_task = Instant::now();
                        }

                        let time_since_task = time_since_task.elapsed().as_micros();
                        if time_since_task > 10 {
                            thread::sleep(Duration::from_micros(time_since_task.min(1000) as u64));
                        }
                    }
                })?;

            workers.push(worker);
        }

        Ok(Self {
            _phantom: PhantomData::default(),
            _workers: workers,
        })
    }
}
