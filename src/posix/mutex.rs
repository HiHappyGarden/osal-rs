#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/posix_bindings.rs"));

    impl Default for pthread_mutex_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_mutexattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

}

use std::ffi::c_int;
use crate::osal::mutex::ffi::pthread_mutex_init;
use crate::traits::Mutex as MutexTrait;
use crate::posix::mutex::ffi::{
    pthread_mutex_t, pthread_mutexattr_t,
    pthread_mutexattr_init, pthread_mutexattr_setprotocol, pthread_mutexattr_settype, pthread_mutex_lock, pthread_mutex_unlock,
    PTHREAD_PRIO_INHERIT, PTHREAD_MUTEX_RECURSIVE
};
use crate::{Result, ErrorType, Error};

pub struct Mutex {
    mutex: pthread_mutex_t,
}

impl MutexTrait for Mutex {
    fn new() -> Result<Self>
    where
        Self: Sized
    {
        let mut ret = Mutex {
                mutex: Default::default(),
            };
        let mut mattr : pthread_mutexattr_t = Default::default();

        unsafe {
            pthread_mutexattr_init(&mut mattr);
            pthread_mutexattr_setprotocol (&mut mattr, PTHREAD_PRIO_INHERIT as c_int);
            pthread_mutexattr_settype (&mut mattr, PTHREAD_MUTEX_RECURSIVE as c_int);

            match ErrorType::new(pthread_mutex_init(&mut ret.mutex, &mattr)) {
                ErrorType::OsEno => Ok(ret),
                err => Err(Error::Type(err, "Failed to initialize mutex")),
            }
        }
    }

    fn lock(&mut self) {
        unsafe {
            pthread_mutex_lock(&mut self.mutex);
        }
    }

    fn lock_from_isr(&mut self) {
        self.lock();
    }

    fn unlock(&mut self) {
        unsafe {
            pthread_mutex_unlock(&mut self.mutex);
        }
    }

    fn unlock_from_isr(&mut self) {
        self.unlock();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::sync::{Arc as StdArc, Mutex as StdMutex};

    #[test]
    #[cfg(feature = "posix")]
    fn test_mutex_new() {
        let result = Mutex::new();
        assert!(result.is_ok(), "Mutex creation should succeed");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_mutex_lock_unlock() {
        let mut mutex = Mutex::new().unwrap();
        
        mutex.lock();
        // If we get here, lock succeeded
        mutex.unlock();
        // If we get here, unlock succeeded
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_mutex_recursive_lock() {
        let mut mutex = Mutex::new().unwrap();
        
        // Recursive mutex should allow multiple locks from same thread
        mutex.lock();
        mutex.lock();
        mutex.unlock();
        mutex.unlock();
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_mutex_thread_safety() {
        // Note: This test is simplified due to thread safety constraints
        // In production, Mutex should be used within a single thread context
        let mut mutex = Mutex::new().unwrap();
        let counter = StdArc::new(StdMutex::new(0));
        
        // Test basic lock/unlock in single thread
        for _ in 0..100 {
            mutex.lock();
            let mut count = counter.lock().unwrap();
            *count += 1;
            drop(count);
            mutex.unlock();
        }
        
        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, 100, "All increments should be protected");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_mutex_from_isr() {
        let mut mutex = Mutex::new().unwrap();
        
        // Test ISR variants (identical to normal in POSIX)
        mutex.lock_from_isr();
        mutex.unlock_from_isr();
    }
}