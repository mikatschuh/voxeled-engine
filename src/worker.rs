use std::{
    fmt, io,
    marker::PhantomData,
    thread,
    time::{Duration, Instant},
};

use crate::ChunkID;

pub trait RecvTask<T: Runable<Self>>: Sized {
    fn recv_task(&mut self) -> Option<T>;
}

pub trait Runable<C> {
    fn run(self, context: &mut C);
}

pub type WorkerID = usize;

#[derive(Debug)]
pub struct Threadpool<T: fmt::Debug + Runable<C>, C: RecvTask<T>> {
    _phantom: PhantomData<(T, C)>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<T: Runable<C> + fmt::Debug + Send + 'static, C: RecvTask<T> + Send + 'static>
    Threadpool<T, C>
{
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
                        while let Some(task) = context.recv_task() {
                            task.run(&mut context);
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
            workers,
        })
    }
}
