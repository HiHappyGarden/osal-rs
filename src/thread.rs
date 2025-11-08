//! Thread/Task abstraction
//!
//! Provides a unified interface for creating and managing threads/tasks
//! across different operating systems.

use crate::Result;

#[cfg(feature = "posix")]
mod posix;

#[cfg(feature = "freertos")]
mod freertos;

/// Thread handle that can be used to wait for thread completion
pub struct Thread {
    #[cfg(feature = "posix")]
    inner: posix::PosixThread,
    #[cfg(feature = "freertos")]
    inner: freertos::FreeRtosThread,
}

impl Thread {
    /// Create and start a new thread with the given name and function
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the thread (for debugging purposes)
    /// * `f` - Function to run in the thread
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osal_rs::Thread;
    ///
    /// let thread = Thread::new("worker", || {
    ///     println!("Hello from thread!");
    /// });
    /// thread.join();
    /// ```
    #[cfg(feature = "posix")]
    pub fn new<F>(name: &str, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            inner: posix::PosixThread::new(name, f),
        }
    }

    #[cfg(feature = "freertos")]
    pub fn new<F>(name: &str, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            inner: freertos::FreeRtosThread::new(name, f),
        }
    }

    /// Wait for the thread to complete
    pub fn join(self) -> Result<()> {
        self.inner.join()
    }

    /// Get the current thread's ID
    pub fn current_id() -> ThreadId {
        #[cfg(feature = "posix")]
        return posix::PosixThread::current_id();
        
        #[cfg(feature = "freertos")]
        return freertos::FreeRtosThread::current_id();
    }

    /// Sleep the current thread for the specified duration
    pub fn sleep(duration: crate::time::Duration) -> Result<()> {
        #[cfg(feature = "posix")]
        return posix::PosixThread::sleep(duration);
        
        #[cfg(feature = "freertos")]
        return freertos::FreeRtosThread::sleep(duration);
    }

    /// Yield the current thread, allowing other threads to run
    pub fn yield_now() -> Result<()> {
        #[cfg(feature = "posix")]
        return posix::PosixThread::yield_now();
        
        #[cfg(feature = "freertos")]
        return freertos::FreeRtosThread::yield_now();
    }
}

/// Thread identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(u64);

impl ThreadId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_creation() {
        use std::sync::{Arc, Mutex as StdMutex};
        let counter = Arc::new(StdMutex::new(0));
        let counter_clone = counter.clone();

        let thread = Thread::new("test_thread", move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
        });

        thread.join().unwrap();
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_thread_current_id() {
        let id1 = Thread::current_id();
        let id2 = Thread::current_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_thread_sleep() {
        use crate::time::Duration;
        let result = Thread::sleep(Duration::from_millis(10));
        assert!(result.is_ok());
    }

    #[test]
    fn test_thread_yield() {
        let result = Thread::yield_now();
        assert!(result.is_ok());
    }
}
