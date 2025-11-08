//! POSIX semaphore implementation

use crate::{Error, Result, time::Duration};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration as StdDuration;

pub struct PosixSemaphore {
    inner: Arc<SemaphoreInner>,
}

struct SemaphoreInner {
    count: Mutex<usize>,
    condvar: Condvar,
}

impl PosixSemaphore {
    pub fn new(initial: usize) -> Self {
        Self {
            inner: Arc::new(SemaphoreInner {
                count: Mutex::new(initial),
                condvar: Condvar::new(),
            }),
        }
    }

    pub fn wait(&self) -> Result<()> {
        let mut count = self.inner.count.lock().unwrap();
        while *count == 0 {
            count = self.inner.condvar.wait(count).unwrap();
        }
        *count -= 1;
        Ok(())
    }

    pub fn try_wait(&self) -> Result<()> {
        let mut count = self.inner.count.lock().unwrap();
        if *count == 0 {
            return Err(Error::WouldBlock);
        }
        *count -= 1;
        Ok(())
    }

    pub fn wait_timeout(&self, timeout: Duration) -> Result<()> {
        let mut count = self.inner.count.lock().unwrap();
        let deadline = StdDuration::from(timeout);
        
        while *count == 0 {
            let result = self.inner.condvar.wait_timeout(count, deadline).unwrap();
            count = result.0;
            
            if result.1.timed_out() {
                return Err(Error::Timeout);
            }
        }
        *count -= 1;
        Ok(())
    }

    pub fn post(&self) -> Result<()> {
        let mut count = self.inner.count.lock().unwrap();
        *count += 1;
        self.inner.condvar.notify_one();
        Ok(())
    }
}

impl Clone for PosixSemaphore {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
