//! POSIX thread implementation

use crate::{Error, Result};
use std::thread::{self, JoinHandle};

pub struct PosixThread {
    handle: Option<JoinHandle<()>>,
}

impl PosixThread {
    pub fn new<F>(name: &str, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let handle = thread::Builder::new()
            .name(name.to_string())
            .spawn(f)
            .expect("Failed to spawn thread");

        Self {
            handle: Some(handle),
        }
    }

    pub fn join(mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| Error::Other("Thread panicked"))?;
        }
        Ok(())
    }

    pub fn current_id() -> crate::thread::ThreadId {
        let id = thread::current().id();
        // Convert std ThreadId to our ThreadId
        // We use a hash of the thread id as a u64
        crate::thread::ThreadId::new(hash_thread_id(id))
    }

    pub fn sleep(duration: crate::time::Duration) -> Result<()> {
        thread::sleep(duration.into());
        Ok(())
    }

    pub fn yield_now() -> Result<()> {
        thread::yield_now();
        Ok(())
    }
}

// Helper function to convert std::thread::ThreadId to u64
fn hash_thread_id(id: thread::ThreadId) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}
