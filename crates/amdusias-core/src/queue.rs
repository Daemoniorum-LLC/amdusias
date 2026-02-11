//! Lock-free single-producer single-consumer queue for audio thread communication.

use crate::{Error, Result};
use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Lock-free single-producer single-consumer (SPSC) queue.
///
/// This queue is designed for real-time audio applications where the audio thread
/// must never block. It uses atomic operations to ensure thread safety without locks.
///
/// # Usage
///
/// - **Producer (non-audio thread)**: Pushes events, parameter changes, etc.
/// - **Consumer (audio thread)**: Pops and processes events without blocking.
///
/// # Memory Ordering
///
/// Uses `Acquire`/`Release` ordering for correctness without the overhead of
/// `SeqCst` ordering.
pub struct SpscQueue<T> {
    /// Ring buffer storage.
    buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    /// Capacity (power of 2 for fast modulo).
    capacity: usize,
    /// Write position (only modified by producer).
    head: AtomicUsize,
    /// Read position (only modified by consumer).
    tail: AtomicUsize,
}

// SAFETY: SpscQueue is Send + Sync because:
// - Only one thread writes to head (producer)
// - Only one thread writes to tail (consumer)
// - Atomic operations ensure visibility
unsafe impl<T: Send> Send for SpscQueue<T> {}
unsafe impl<T: Send> Sync for SpscQueue<T> {}

impl<T> SpscQueue<T> {
    /// Creates a new SPSC queue with the given capacity.
    ///
    /// The capacity is rounded up to the next power of 2 for efficient modulo operations.
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be > 0");

        // Round up to next power of 2
        let capacity = capacity.next_power_of_two();

        let buffer: Box<[UnsafeCell<MaybeUninit<T>>]> = (0..capacity)
            .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
            .collect();

        Self {
            buffer,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Returns the capacity of the queue.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of items currently in the queue.
    ///
    /// Note: This is an approximation in a concurrent context.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    /// Returns true if the queue is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if the queue is full.
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    /// Pushes an item onto the queue.
    ///
    /// # Errors
    ///
    /// Returns `Error::QueueFull` if the queue is at capacity.
    ///
    /// # Thread Safety
    ///
    /// Only one thread should call this method (the producer).
    pub fn push(&self, value: T) -> Result<()> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= self.capacity {
            return Err(Error::QueueFull);
        }

        let index = head & (self.capacity - 1);

        // SAFETY: We have exclusive access to this slot (head position).
        unsafe {
            (*self.buffer[index].get()).write(value);
        }

        // Release ensures the write above is visible before we update head.
        self.head.store(head.wrapping_add(1), Ordering::Release);

        Ok(())
    }

    /// Pops an item from the queue.
    ///
    /// # Errors
    ///
    /// Returns `Error::QueueEmpty` if the queue is empty.
    ///
    /// # Thread Safety
    ///
    /// Only one thread should call this method (the consumer).
    pub fn pop(&self) -> Result<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return Err(Error::QueueEmpty);
        }

        let index = tail & (self.capacity - 1);

        // SAFETY: We have exclusive access to this slot (tail position).
        let value = unsafe { (*self.buffer[index].get()).assume_init_read() };

        // Release ensures the read above completes before we update tail.
        self.tail.store(tail.wrapping_add(1), Ordering::Release);

        Ok(value)
    }

    /// Tries to peek at the front item without removing it.
    ///
    /// # Safety
    ///
    /// The returned reference is only valid until the next `pop` call.
    /// Only the consumer thread should call this.
    pub fn peek(&self) -> Option<&T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let index = tail & (self.capacity - 1);

        // SAFETY: Item exists and won't be modified until pop().
        Some(unsafe { (*self.buffer[index].get()).assume_init_ref() })
    }
}

impl<T> Drop for SpscQueue<T> {
    fn drop(&mut self) {
        // Drop any remaining items
        while self.pop().is_ok() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let queue = SpscQueue::new(4);

        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();

        assert_eq!(queue.pop().unwrap(), 1);
        assert_eq!(queue.pop().unwrap(), 2);
        assert_eq!(queue.pop().unwrap(), 3);
        assert!(queue.pop().is_err());
    }

    #[test]
    fn test_full_queue() {
        let queue = SpscQueue::new(2);

        queue.push(1).unwrap();
        queue.push(2).unwrap();
        assert!(queue.push(3).is_err()); // Queue is full
    }

    #[test]
    fn test_wrap_around() {
        let queue = SpscQueue::new(2);

        for i in 0..10 {
            queue.push(i).unwrap();
            assert_eq!(queue.pop().unwrap(), i);
        }
    }

    #[test]
    fn test_capacity_power_of_two() {
        // Capacity should be rounded up to power of 2
        let queue = SpscQueue::<i32>::new(3);
        assert_eq!(queue.capacity(), 4);

        let queue = SpscQueue::<i32>::new(5);
        assert_eq!(queue.capacity(), 8);

        let queue = SpscQueue::<i32>::new(16);
        assert_eq!(queue.capacity(), 16);
    }

    #[test]
    fn test_peek() {
        let queue = SpscQueue::new(4);

        queue.push(42).unwrap();

        // Peek should see the value without removing it
        assert_eq!(*queue.peek().unwrap(), 42);
        assert_eq!(*queue.peek().unwrap(), 42);

        // Pop should remove it
        assert_eq!(queue.pop().unwrap(), 42);
        assert!(queue.peek().is_none());
    }

    #[test]
    fn test_len_and_empty() {
        let queue = SpscQueue::new(4);

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.push(1).unwrap();
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);

        queue.push(2).unwrap();
        assert_eq!(queue.len(), 2);

        queue.pop().unwrap();
        assert_eq!(queue.len(), 1);

        queue.pop().unwrap();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_is_full() {
        let queue = SpscQueue::new(2);

        assert!(!queue.is_full());
        queue.push(1).unwrap();
        assert!(!queue.is_full());
        queue.push(2).unwrap();
        assert!(queue.is_full());
    }
}

#[cfg(test)]
mod concurrent_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_spsc_queue_concurrent() {
        const NUM_ITEMS: usize = 10_000;
        let queue = Arc::new(SpscQueue::new(1024));

        let producer_queue = Arc::clone(&queue);
        let consumer_queue = Arc::clone(&queue);

        // Producer thread
        let producer = thread::spawn(move || {
            for i in 0..NUM_ITEMS {
                // Spin until we can push
                while producer_queue.push(i).is_err() {
                    thread::yield_now();
                }
            }
        });

        // Consumer thread
        let consumer = thread::spawn(move || {
            let mut received = Vec::with_capacity(NUM_ITEMS);
            while received.len() < NUM_ITEMS {
                if let Ok(value) = consumer_queue.pop() {
                    received.push(value);
                } else {
                    thread::yield_now();
                }
            }
            received
        });

        producer.join().expect("producer panicked");
        let received = consumer.join().expect("consumer panicked");

        // Verify all items received in order
        assert_eq!(received.len(), NUM_ITEMS);
        for (i, &value) in received.iter().enumerate() {
            assert_eq!(
                value, i,
                "Item {} was received out of order: expected {}, got {}",
                i, i, value
            );
        }
    }

    #[test]
    fn test_spsc_queue_stress() {
        // Stress test with rapid push/pop cycles
        const ITERATIONS: usize = 100_000;
        let queue = Arc::new(SpscQueue::new(64));

        let producer_queue = Arc::clone(&queue);
        let consumer_queue = Arc::clone(&queue);

        let producer = thread::spawn(move || {
            for i in 0..ITERATIONS {
                while producer_queue.push(i).is_err() {
                    // Queue full, yield
                    thread::yield_now();
                }
            }
        });

        let consumer = thread::spawn(move || {
            let mut count = 0;
            let mut last_value = None;
            while count < ITERATIONS {
                if let Ok(value) = consumer_queue.pop() {
                    // Verify monotonic ordering
                    if let Some(last) = last_value {
                        assert!(
                            value > last,
                            "Values not monotonically increasing: {} after {}",
                            value,
                            last
                        );
                    }
                    last_value = Some(value);
                    count += 1;
                }
            }
            count
        });

        producer.join().expect("producer panicked");
        let count = consumer.join().expect("consumer panicked");
        assert_eq!(count, ITERATIONS);
    }

    #[test]
    fn test_spsc_queue_no_data_loss() {
        // Verify no data is lost or duplicated
        const NUM_ITEMS: usize = 50_000;
        let queue = Arc::new(SpscQueue::new(256));

        let producer_queue = Arc::clone(&queue);
        let consumer_queue = Arc::clone(&queue);

        let producer = thread::spawn(move || {
            let mut sum: u64 = 0;
            for i in 0..NUM_ITEMS {
                sum += i as u64;
                while producer_queue.push(i).is_err() {
                    thread::yield_now();
                }
            }
            sum
        });

        let consumer = thread::spawn(move || {
            let mut sum: u64 = 0;
            let mut count = 0;
            while count < NUM_ITEMS {
                if let Ok(value) = consumer_queue.pop() {
                    sum += value as u64;
                    count += 1;
                }
            }
            sum
        });

        let producer_sum = producer.join().expect("producer panicked");
        let consumer_sum = consumer.join().expect("consumer panicked");

        assert_eq!(
            producer_sum, consumer_sum,
            "Data loss or corruption: producer sum {} != consumer sum {}",
            producer_sum, consumer_sum
        );
    }
}
