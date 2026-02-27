use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "capacity must be > 0");
    assert!(
        capacity.is_power_of_two(),
        "capacity must be a power of two"
    );

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
    mask: usize,
    enqueue_pos: AtomicUsize,
    dequeue_pos: AtomicUsize,
    sender_count: AtomicUsize,
    receiver_alive: AtomicBool,
}

#[derive(Debug)]
struct Slot<T> {
    sequence: AtomicUsize,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Inner<T> {
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for i in 0..capacity {
            buffer.push(Slot {
                sequence: AtomicUsize::new(i),
                value: UnsafeCell::new(MaybeUninit::uninit()),
            });
        }

        Self {
            buffer: buffer.into_boxed_slice(),
            mask: capacity - 1,
            enqueue_pos: AtomicUsize::new(0),
            dequeue_pos: AtomicUsize::new(0),
            sender_count: AtomicUsize::new(1),
            receiver_alive: AtomicBool::new(true),
        }
    }
}

impl<T> Sender<T> {
    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        if !self.inner.receiver_alive.load(Ordering::Acquire) {
            return Err(TrySendError::Disconnected(value));
        }

        loop {
            let pos = self.inner.enqueue_pos.load(Ordering::Relaxed);
            let slot = &self.inner.buffer[pos & self.inner.mask];
            let seq = slot.sequence.load(Ordering::Acquire);
            let diff = seq as isize - pos as isize;

            if diff == 0 {
                if self
                    .inner
                    .enqueue_pos
                    .compare_exchange_weak(
                        pos,
                        pos.wrapping_add(1),
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    // SAFETY: This producer has uniquely claimed this slot position.
                    unsafe {
                        (*slot.value.get()).write(value);
                    }
                    slot.sequence.store(pos.wrapping_add(1), Ordering::Release);
                    return Ok(());
                }
            } else if diff < 0 {
                return Err(TrySendError::Full(value));
            } else {
                std::hint::spin_loop();
            }

            if !self.inner.receiver_alive.load(Ordering::Acquire) {
                return Err(TrySendError::Disconnected(value));
            }
        }
    }

    pub fn is_disconnected(&self) -> bool {
        !self.inner.receiver_alive.load(Ordering::Acquire)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.sender_count.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.inner.sender_count.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        loop {
            let pos = self.inner.dequeue_pos.load(Ordering::Relaxed);
            let slot = &self.inner.buffer[pos & self.inner.mask];
            let seq = slot.sequence.load(Ordering::Acquire);
            let diff = seq as isize - pos.wrapping_add(1) as isize;

            if diff == 0 {
                if self
                    .inner
                    .dequeue_pos
                    .compare_exchange_weak(
                        pos,
                        pos.wrapping_add(1),
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    // SAFETY: Single consumer has uniquely claimed this slot for read.
                    let value = unsafe { (*slot.value.get()).assume_init_read() };
                    slot.sequence.store(
                        pos.wrapping_add(self.inner.mask).wrapping_add(1),
                        Ordering::Release,
                    );
                    return Ok(value);
                }
            } else if diff < 0 {
                if self.inner.sender_count.load(Ordering::Acquire) == 0 {
                    return Err(TryRecvError::Disconnected);
                }
                return Err(TryRecvError::Empty);
            } else {
                std::hint::spin_loop();
            }
        }
    }

    pub fn is_disconnected(&self) -> bool {
        self.inner.sender_count.load(Ordering::Acquire) == 0
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.receiver_alive.store(false, Ordering::Release);
    }
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        let start = self.dequeue_pos.load(Ordering::Relaxed);
        let end = self.enqueue_pos.load(Ordering::Relaxed);

        for pos in start..end {
            let slot = &self.buffer[pos & self.mask];
            let seq = slot.sequence.load(Ordering::Relaxed);
            if seq == pos.wrapping_add(1) {
                // SAFETY: Queue is no longer shared; any occupied slot must be dropped.
                unsafe {
                    (*slot.value.get()).assume_init_drop();
                }
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
        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 1);
        assert_eq!(rx.try_recv().unwrap(), 2);
        assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
    }

    #[test]
    fn full_queue() {
        let (tx, _rx) = channel(2);
        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();
        assert!(matches!(tx.try_send(3), Err(TrySendError::Full(3))));
    }

    #[test]
    fn disconnected_sender_side() {
        let (tx, rx) = channel::<u32>(2);
        drop(rx);
        assert!(matches!(tx.try_send(7), Err(TrySendError::Disconnected(7))));
    }

    #[test]
    fn disconnected_receiver_side() {
        let (tx, rx) = channel(2);
        tx.try_send(11).unwrap();
        drop(tx);
        assert_eq!(rx.try_recv().unwrap(), 11);
        assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Disconnected);
    }
}
