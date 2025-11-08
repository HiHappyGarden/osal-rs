//! Semaphore primitives for synchronization

use crate::{Result, time::Duration};

#[cfg(feature = "posix")]
mod posix;

#[cfg(feature = "freertos")]
mod freertos;

/// A counting semaphore
pub struct Semaphore {
    #[cfg(feature = "posix")]
    inner: posix::PosixSemaphore,
    #[cfg(feature = "freertos")]
    inner: freertos::FreeRtosSemaphore,
}

impl Semaphore {
    /// Creates a new semaphore with the given initial count
    ///
    /// # Arguments
    ///
    /// * `initial` - Initial count of the semaphore
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osal_rs::Semaphore;
    ///
    /// let sem = Semaphore::new(1);
    /// ```
    pub fn new(initial: usize) -> Self {
        #[cfg(feature = "posix")]
        return Self {
            inner: posix::PosixSemaphore::new(initial),
        };

        #[cfg(feature = "freertos")]
        return Self {
            inner: freertos::FreeRtosSemaphore::new(initial),
        };
    }

    /// Acquires the semaphore, blocking until it's available
    pub fn wait(&self) -> Result<()> {
        self.inner.wait()
    }

    /// Attempts to acquire the semaphore without blocking
    pub fn try_wait(&self) -> Result<()> {
        self.inner.try_wait()
    }

    /// Attempts to acquire the semaphore, waiting for at most the specified duration
    pub fn wait_timeout(&self, timeout: Duration) -> Result<()> {
        self.inner.wait_timeout(timeout)
    }

    /// Releases the semaphore, incrementing its count
    pub fn post(&self) -> Result<()> {
        self.inner.post()
    }
}

/// A binary semaphore (0 or 1)
pub struct BinarySemaphore {
    inner: Semaphore,
}

impl BinarySemaphore {
    /// Creates a new binary semaphore
    ///
    /// # Arguments
    ///
    /// * `available` - Whether the semaphore is initially available
    pub fn new(available: bool) -> Self {
        Self {
            inner: Semaphore::new(if available { 1 } else { 0 }),
        }
    }

    /// Acquires the binary semaphore
    pub fn wait(&self) -> Result<()> {
        self.inner.wait()
    }

    /// Attempts to acquire the binary semaphore without blocking
    pub fn try_wait(&self) -> Result<()> {
        self.inner.try_wait()
    }

    /// Releases the binary semaphore
    pub fn post(&self) -> Result<()> {
        self.inner.post()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn test_semaphore_basic() {
        let sem = Semaphore::new(1);
        assert!(sem.wait().is_ok());
        assert!(sem.post().is_ok());
    }

    #[test]
    fn test_semaphore_try_wait() {
        let sem = Semaphore::new(0);
        assert!(matches!(sem.try_wait(), Err(Error::WouldBlock)));
        
        sem.post().unwrap();
        assert!(sem.try_wait().is_ok());
    }

    #[test]
    fn test_binary_semaphore() {
        let sem = BinarySemaphore::new(true);
        assert!(sem.wait().is_ok());
        assert!(matches!(sem.try_wait(), Err(Error::WouldBlock)));
        assert!(sem.post().is_ok());
        assert!(sem.wait().is_ok());
    }

    #[test]
    fn test_semaphore_multi_thread() {
        use std::sync::Arc;
        use crate::Thread;

        let sem = Arc::new(Semaphore::new(0));
        let sem_clone = sem.clone();

        let thread = Thread::new("test", move || {
            sem_clone.post().unwrap();
        });

        sem.wait().unwrap();
        thread.join().unwrap();
    }
}
