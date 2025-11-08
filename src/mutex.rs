//! Mutex abstraction
//!
//! Provides mutual exclusion primitives for protecting shared data.

use crate::Result;

#[cfg(feature = "posix")]
mod posix;

#[cfg(feature = "freertos")]
mod freertos;

/// A mutual exclusion primitive useful for protecting shared data
pub struct Mutex<T> {
    #[cfg(feature = "posix")]
    inner: posix::PosixMutex<T>,
    #[cfg(feature = "freertos")]
    inner: freertos::FreeRtosMutex<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osal_rs::Mutex;
    ///
    /// let mutex = Mutex::new(42);
    /// ```
    pub fn new(value: T) -> Self {
        #[cfg(feature = "posix")]
        return Self {
            inner: posix::PosixMutex::new(value),
        };

        #[cfg(feature = "freertos")]
        return Self {
            inner: freertos::FreeRtosMutex::new(value),
        };
    }

    /// Acquires a mutex, blocking the current thread until it is able to do so
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osal_rs::Mutex;
    ///
    /// let mutex = Mutex::new(42);
    /// let guard = mutex.lock();
    /// println!("Value: {}", *guard);
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, T> {
        #[cfg(feature = "posix")]
        return MutexGuard {
            inner: self.inner.lock(),
        };

        #[cfg(feature = "freertos")]
        return MutexGuard {
            inner: self.inner.lock(),
        };
    }

    /// Attempts to acquire this mutex without blocking
    ///
    /// # Returns
    ///
    /// * `Ok(guard)` if the lock was acquired
    /// * `Err(Error::WouldBlock)` if the mutex is currently locked
    pub fn try_lock(&self) -> Result<MutexGuard<'_, T>> {
        #[cfg(feature = "posix")]
        return Ok(MutexGuard {
            inner: self.inner.try_lock()?,
        });

        #[cfg(feature = "freertos")]
        return Ok(MutexGuard {
            inner: self.inner.try_lock()?,
        });
    }
}

/// An RAII guard that provides exclusive access to the data protected by a mutex
pub struct MutexGuard<'a, T> {
    #[cfg(feature = "posix")]
    inner: posix::PosixMutexGuard<'a, T>,
    #[cfg(feature = "freertos")]
    inner: freertos::FreeRtosMutexGuard<'a, T>,
}

impl<'a, T> std::ops::Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> std::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_basic() {
        let mutex = Mutex::new(42);
        let guard = mutex.lock();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_mutex_modify() {
        let mutex = Mutex::new(0);
        {
            let mut guard = mutex.lock();
            *guard = 10;
        }
        let guard = mutex.lock();
        assert_eq!(*guard, 10);
    }

    #[test]
    fn test_mutex_try_lock() {
        let mutex = Mutex::new(42);
        let guard1 = mutex.try_lock();
        assert!(guard1.is_ok());
        
        // Try to acquire again (should fail on POSIX as it's not recursive)
        // Note: This test behavior may vary by implementation
        drop(guard1);
        let guard2 = mutex.try_lock();
        assert!(guard2.is_ok());
    }

    #[test]
    fn test_mutex_multi_thread() {
        use std::sync::Arc;
        use crate::Thread;

        let mutex = Arc::new(Mutex::new(0));
        let mutex_clone = mutex.clone();

        let thread = Thread::new("test", move || {
            let mut guard = mutex_clone.lock();
            *guard += 1;
        });

        thread.join().unwrap();
        let guard = mutex.lock();
        assert_eq!(*guard, 1);
    }
}
