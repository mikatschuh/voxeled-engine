use crossbeam::deque::Injector;
use parking_lot::RwLock;
use std::{mem, sync::Arc, thread};

use crate::{job::Job, world_gen::Generator};

#[derive(Debug)]
pub struct Threadpool<G: Generator> {
    debug_book: Vec<Arc<RwLock<Vec<String>>>>, // for every worker

    task_queue: Arc<Injector<Job<G>>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<G: Generator> Threadpool<G> {
    pub fn new(num_threads: usize) -> Self {
        let task_queue = Arc::new(Injector::<Job<G>>::new());
        let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();

        let mut debug_book = Vec::new();

        for i in 0..num_threads {
            let debug_log = Arc::new(RwLock::new(Vec::new()));
            let task_queue = task_queue.clone();

            let cloned_debug_log = debug_log.clone();
            let Ok(join_handle) = thread::Builder::new()
                .name(format!("{}", i))
                .spawn(move || {
                    loop {
                        // Always handle ALL priority tasks first
                        while let Some(task) = task_queue.steal().success() {
                            let mut lock = cloned_debug_log.write();
                            task.run(&mut lock);
                        }
                    }
                })
            else {
                println!("thread couldnt been spawned");
                continue;
            };

            debug_book.push(debug_log);
            workers.push(join_handle);
        }

        println!("\n\ntotal number of threads: {} threads", workers.len());

        Self {
            debug_book,
            workers,
            task_queue,
        }
    }

    pub fn debug_log(&mut self) -> String {
        let mut out = String::new();
        for (worker, debug_log) in self.debug_book.iter().enumerate() {
            out += &format!(
                "\nthread {worker}: {}",
                mem::take(&mut *debug_log.write())
                    .into_iter()
                    .reduce(|a, b| a.to_owned() + ", " + &b)
                    .unwrap_or(String::new())
            );
            for task in debug_log.read().iter() {
                out += &task.to_string()
            }
        }
        out
    }

    /// A function to add priority tasks. Returns the task if the queue was full.
    pub fn push(&mut self, task: Job<G>) {
        self.task_queue.push(task);
    }

    pub fn drop(self) {
        for worker in self.workers {
            let _ = worker.join();
        }
    }
}
