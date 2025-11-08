//! POSIX timer implementation

use crate::{Error, Result, time::Duration};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Duration as StdDuration;

pub struct PosixTimer {
    callback: Arc<Mutex<Box<dyn FnMut() + Send>>>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl PosixTimer {
    pub fn new<F>(name: &str, callback: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let _ = name; // Name is for debugging, not used in basic implementation
        Self {
            callback: Arc::new(Mutex::new(Box::new(callback))),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn start_oneshot(&mut self, delay: Duration) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Err(Error::Other("Timer already running"));
        }

        self.running.store(true, Ordering::SeqCst);
        let callback = Arc::clone(&self.callback);
        let running = Arc::clone(&self.running);
        let delay_std = StdDuration::from(delay);

        self.handle = Some(thread::spawn(move || {
            thread::sleep(delay_std);
            if running.load(Ordering::SeqCst) {
                let mut cb = callback.lock().unwrap();
                cb();
                running.store(false, Ordering::SeqCst);
            }
        }));

        Ok(())
    }

    pub fn start_periodic(&mut self, period: Duration) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Err(Error::Other("Timer already running"));
        }

        self.running.store(true, Ordering::SeqCst);
        let callback = Arc::clone(&self.callback);
        let running = Arc::clone(&self.running);
        let period_std = StdDuration::from(period);

        self.handle = Some(thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                thread::sleep(period_std);
                if running.load(Ordering::SeqCst) {
                    let mut cb = callback.lock().unwrap();
                    cb();
                }
            }
        }));

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(false, Ordering::SeqCst);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for PosixTimer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
