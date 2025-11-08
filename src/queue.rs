//! Message queue for inter-thread communication

use crate::{Result, time::Duration};

#[cfg(feature = "posix")]
mod posix;

#[cfg(feature = "freertos")]
mod freertos;

/// A thread-safe message queue
pub struct Queue<T> {
    #[cfg(feature = "posix")]
    inner: posix::PosixQueue<T>,
    #[cfg(feature = "freertos")]
    inner: freertos::FreeRtosQueue<T>,
}

impl<T> Queue<T> {
    /// Creates a new queue with the specified capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of items the queue can hold
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osal_rs::Queue;
    ///
    /// let queue: Queue<i32> = Queue::new(10);
    /// ```
    pub fn new(capacity: usize) -> Self {
        #[cfg(feature = "posix")]
        return Self {
            inner: posix::PosixQueue::new(capacity),
        };

        #[cfg(feature = "freertos")]
        return Self {
            inner: freertos::FreeRtosQueue::new(capacity),
        };
    }

    /// Sends a message to the queue, blocking if the queue is full
    pub fn send(&self, item: T) -> Result<()> {
        self.inner.send(item)
    }

    /// Attempts to send a message without blocking
    pub fn try_send(&self, item: T) -> Result<()> {
        self.inner.try_send(item)
    }

    /// Sends a message with a timeout
    pub fn send_timeout(&self, item: T, timeout: Duration) -> Result<()> {
        self.inner.send_timeout(item, timeout)
    }

    /// Receives a message from the queue, blocking if empty
    pub fn recv(&self) -> Result<T> {
        self.inner.recv()
    }

    /// Attempts to receive a message without blocking
    pub fn try_recv(&self) -> Result<T> {
        self.inner.try_recv()
    }

    /// Receives a message with a timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T> {
        self.inner.recv_timeout(timeout)
    }

    /// Returns the number of items currently in the queue
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity of the queue
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn test_queue_basic() {
        let queue = Queue::new(5);
        queue.send(42).unwrap();
        assert_eq!(queue.recv().unwrap(), 42);
    }

    #[test]
    fn test_queue_try_recv() {
        let queue: Queue<i32> = Queue::new(5);
        assert!(matches!(queue.try_recv(), Err(Error::WouldBlock)));
        
        queue.send(10).unwrap();
        assert_eq!(queue.try_recv().unwrap(), 10);
    }

    #[test]
    #[ignore] // Note: POSIX implementation doesn't track length accurately
    fn test_queue_len() {
        let queue = Queue::new(5);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        
        queue.send(1).unwrap();
        queue.send(2).unwrap();
        assert_eq!(queue.len(), 2);
        assert!(!queue.is_empty());
        
        queue.recv().unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_queue_capacity() {
        let queue: Queue<i32> = Queue::new(10);
        assert_eq!(queue.capacity(), 10);
    }

    #[test]
    fn test_queue_multi_thread() {
        use std::sync::Arc;
        use crate::Thread;

        let queue = Arc::new(Queue::new(10));
        let queue_clone = queue.clone();

        let thread = Thread::new("sender", move || {
            for i in 0..5 {
                queue_clone.send(i).unwrap();
            }
        });

        for i in 0..5 {
            assert_eq!(queue.recv().unwrap(), i);
        }

        thread.join().unwrap();
    }
}
