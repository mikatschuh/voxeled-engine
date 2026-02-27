use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "capacity must be > 0");

    let inner = Arc::new(Inner::new(capacity));
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner },
    )
}

#[derive(Debug)]
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
    Disconnected(T),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

#[derive(Debug)]
struct Inner<T> {
    buffer: Box<[Slot<T>]>,
    capacity: usize,
    producer_pos: AtomicUsize,
    consumer_pos: AtomicUsize,
    sender_alive: AtomicBool,
    receiver_alive: AtomicBool,
}

#[derive(Debug)]
struct Slot<T> {
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Inner<T> {
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(Slot {
                value: UnsafeCell::new(MaybeUninit::uninit()),
            });
        }

        Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            producer_pos: AtomicUsize::new(0),
            consumer_pos: AtomicUsize::new(0),
            sender_alive: AtomicBool::new(true),
            receiver_alive: AtomicBool::new(true),
        }
    }
}

impl<T> Sender<T> {
    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        if !self.inner.receiver_alive.load(Ordering::Acquire) {
            return Err(TrySendError::Disconnected(value));
        }

        let producer = self.inner.producer_pos.load(Ordering::Relaxed);
        let consumer = self.inner.consumer_pos.load(Ordering::Acquire);

        if producer.wrapping_sub(consumer) == self.inner.capacity {
            return Err(TrySendError::Full(value));
        }

        let idx = producer % self.inner.capacity;
        // SAFETY: In SPSC, only the producer writes this slot before producer_pos advances.
        unsafe {
            (*self.inner.buffer[idx].value.get()).write(value);
        }
        self.inner
            .producer_pos
            .store(producer.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    pub fn is_disconnected(&self) -> bool {
        !self.inner.receiver_alive.load(Ordering::Acquire)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.inner.sender_alive.store(false, Ordering::Release);
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let consumer = self.inner.consumer_pos.load(Ordering::Relaxed);
        let producer = self.inner.producer_pos.load(Ordering::Acquire);

        if consumer == producer {
            if !self.inner.sender_alive.load(Ordering::Acquire) {
                return Err(TryRecvError::Disconnected);
            }
            return Err(TryRecvError::Empty);
        }

        let idx = consumer % self.inner.capacity;
        // SAFETY: In SPSC, only the consumer reads this slot after observing producer_pos.
        let value = unsafe { (*self.inner.buffer[idx].value.get()).assume_init_read() };
        self.inner
            .consumer_pos
            .store(consumer.wrapping_add(1), Ordering::Release);
        Ok(value)
    }

    /// Drains all currently available elements in FIFO order.
    pub fn drain(&self) -> Vec<T> {
        let mut out = Vec::new();
        while let Ok(value) = self.try_recv() {
            out.push(value);
        }
        out
    }

    pub fn len(&self) -> usize {
        let producer = self.inner.producer_pos.load(Ordering::Acquire);
        let consumer = self.inner.consumer_pos.load(Ordering::Acquire);
        producer.wrapping_sub(consumer)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_disconnected(&self) -> bool {
        !self.inner.sender_alive.load(Ordering::Acquire)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.receiver_alive.store(false, Ordering::Release);
    }
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        let consumer = self.consumer_pos.load(Ordering::Relaxed);
        let producer = self.producer_pos.load(Ordering::Relaxed);

        for pos in consumer..producer {
            let idx = pos % self.capacity;
            // SAFETY: Queue is no longer shared; every slot in [consumer, producer) is initialized.
            unsafe {
                (*self.buffer[idx].value.get()).assume_init_drop();
            }
        }
    }
}

unsafe impl<T: Send> Send for Slot<T> {}
unsafe impl<T: Send> Sync for Slot<T> {}

#[cfg(test)]
mod tests {
    use super::{TryRecvError, TrySendError, channel};

    #[test]
    fn basic_send_recv() {
        let (tx, rx) = channel(4);
        tx.try_send(10).unwrap();
        tx.try_send(20).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 10);
        assert_eq!(rx.try_recv().unwrap(), 20);
        assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
    }

    #[test]
    fn fixed_capacity() {
        let (tx, rx) = channel(2);
        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();
        assert!(matches!(tx.try_send(3), Err(TrySendError::Full(3))));
        assert_eq!(rx.try_recv().unwrap(), 1);
        tx.try_send(3).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 2);
        assert_eq!(rx.try_recv().unwrap(), 3);
    }

    #[test]
    fn drain_all_items() {
        let (tx, rx) = channel(8);
        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();
        tx.try_send(3).unwrap();
        assert_eq!(rx.drain(), vec![1, 2, 3]);
        assert!(rx.is_empty());
    }

    #[test]
    fn disconnected_behavior() {
        let (tx, rx) = channel::<u32>(2);
        drop(rx);
        assert!(matches!(tx.try_send(1), Err(TrySendError::Disconnected(1))));

        let (tx, rx) = channel(2);
        tx.try_send(7).unwrap();
        drop(tx);
        assert_eq!(rx.try_recv().unwrap(), 7);
        assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Disconnected);
    }
}
