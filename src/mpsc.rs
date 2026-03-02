use std::sync::Arc;

use crossbeam::queue::ArrayQueue;
use rtrb::{PopError, PushError};

pub fn new<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "capacity must be > 0");

    let inner = Arc::new(Inner::new(capacity));

    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner },
    )
}

#[derive(Debug, Clone)]
pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

#[derive(Debug)]
pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrySendError<T> {
    Full(T),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TryRecvError {
    Empty,
}

#[derive(Debug)]
struct Inner<T> {
    queue: ArrayQueue<T>,
}

impl<T> Inner<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: ArrayQueue::new(capacity),
        }
    }
}

impl<T> Sender<T> {
    pub fn push(&self, value: T) -> Result<(), PushError<T>> {
        match self.inner.queue.push(value) {
            Ok(()) => Ok(()),
            Err(value) => Err(PushError::Full(value)),
        }
    }
}

impl<T> Receiver<T> {
    pub fn pop(&self) -> Result<T, PopError> {
        self.inner.queue.pop().ok_or(PopError::Empty)
    }

    pub fn drain(&self) -> Vec<T> {
        let mut values = Vec::new();
        while let Ok(value) = self.pop() {
            values.push(value);
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::{PopError::Empty, PushError::Full, new as channel};

    #[test]
    fn basic_send_recv() {
        let (tx, rx) = channel(4);
        tx.push(10).unwrap();
        tx.push(20).unwrap();
        assert_eq!(rx.pop().unwrap(), 10);
        assert_eq!(rx.pop().unwrap(), 20);
        assert_eq!(rx.pop().unwrap_err(), Empty);
    }

    #[test]
    fn full_when_queue_is_full() {
        let (tx, rx) = channel(2);

        tx.push(1).unwrap();
        tx.push(2).unwrap();
        assert_eq!(tx.push(3), Err(Full(3)));

        assert_eq!(rx.pop().unwrap(), 1);
        tx.push(3).unwrap();
    }

    #[test]
    fn capacity_is_shared_across_all_producers() {
        let (tx0, rx) = channel(1);
        let tx1 = tx0.clone();

        tx0.push(1).unwrap();
        assert_eq!(tx1.push(10), Err(Full(10)));
        assert_eq!(rx.pop().unwrap(), 1);
    }

    #[test]
    fn threaded_multi_producer_receives_everything_in_producer_order() {
        let producers = 4usize;
        let messages_per_producer = 20_000usize;
        let (tx, rx) = channel::<u64>(1024);

        let mut handles = Vec::with_capacity(producers);
        for producer_id in 0..producers {
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                for seq in 0..messages_per_producer {
                    let mut value = ((producer_id as u64) << 32) | (seq as u64);
                    loop {
                        match tx.push(value) {
                            Ok(()) => break,
                            Err(Full(v)) => {
                                value = v;
                                std::hint::spin_loop();
                            }
                        }
                    }
                }
            }));
        }
        drop(tx);

        let total = producers * messages_per_producer;
        let mut received = 0usize;
        let mut next_expected = vec![0usize; producers];
        while received < total {
            match rx.pop() {
                Ok(value) => {
                    let producer_id = (value >> 32) as usize;
                    let seq = (value & 0xFFFF_FFFF) as usize;
                    assert_eq!(seq, next_expected[producer_id]);
                    next_expected[producer_id] += 1;
                    received += 1;
                }
                Err(_) => std::hint::spin_loop(),
            }
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn drain_collects_all_current_items() {
        let (tx, rx) = channel(8);
        tx.push(1).unwrap();
        tx.push(2).unwrap();
        tx.push(3).unwrap();
        assert_eq!(rx.drain(), vec![1, 2, 3]);
        assert_eq!(rx.pop().unwrap_err(), Empty);
    }
}
