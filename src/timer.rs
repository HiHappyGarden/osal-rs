//! Timer functionality for periodic and one-shot callbacks

use crate::{Result, time::Duration};

#[cfg(feature = "posix")]
mod posix;

#[cfg(feature = "freertos")]
mod freertos;

/// A timer that can execute callbacks after a delay or periodically
pub struct Timer {
    #[cfg(feature = "posix")]
    inner: posix::PosixTimer,
    #[cfg(feature = "freertos")]
    inner: freertos::FreeRtosTimer,
}

impl Timer {
    /// Creates a new timer with the given name and callback
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the timer (for debugging)
    /// * `callback` - Function to call when timer expires
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osal_rs::{Timer, time::Duration};
    /// use std::sync::{Arc, Mutex};
    ///
    /// let counter = Arc::new(Mutex::new(0));
    /// let counter_clone = counter.clone();
    ///
    /// let timer = Timer::new("test", move || {
    ///     let mut c = counter_clone.lock().unwrap();
    ///     *c += 1;
    /// });
    /// ```
    pub fn new<F>(name: &str, callback: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        #[cfg(feature = "posix")]
        return Self {
            inner: posix::PosixTimer::new(name, callback),
        };

        #[cfg(feature = "freertos")]
        return Self {
            inner: freertos::FreeRtosTimer::new(name, callback),
        };
    }

    /// Starts the timer as a one-shot timer
    ///
    /// # Arguments
    ///
    /// * `delay` - Time to wait before executing the callback
    pub fn start_oneshot(&mut self, delay: Duration) -> Result<()> {
        self.inner.start_oneshot(delay)
    }

    /// Starts the timer as a periodic timer
    ///
    /// # Arguments
    ///
    /// * `period` - Time between callback executions
    pub fn start_periodic(&mut self, period: Duration) -> Result<()> {
        self.inner.start_periodic(period)
    }

    /// Stops the timer
    pub fn stop(&mut self) -> Result<()> {
        self.inner.stop()
    }

    /// Checks if the timer is currently running
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration as StdDuration;

    #[test]
    fn test_timer_oneshot() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let mut timer = Timer::new("test", move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
        });

        timer.start_oneshot(Duration::from_millis(50)).unwrap();
        assert!(timer.is_running());

        std::thread::sleep(StdDuration::from_millis(100));
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_timer_periodic() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let mut timer = Timer::new("test", move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
        });

        timer.start_periodic(Duration::from_millis(30)).unwrap();
        std::thread::sleep(StdDuration::from_millis(100));
        
        let count = *counter.lock().unwrap();
        assert!(count >= 2); // Should have fired at least twice
        
        timer.stop().unwrap();
        assert!(!timer.is_running());
    }

    #[test]
    fn test_timer_stop() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let mut timer = Timer::new("test", move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
        });

        timer.start_periodic(Duration::from_millis(20)).unwrap();
        std::thread::sleep(StdDuration::from_millis(50));
        timer.stop().unwrap();
        
        let count_at_stop = *counter.lock().unwrap();
        std::thread::sleep(StdDuration::from_millis(50));
        let count_after = *counter.lock().unwrap();
        
        // Count should not increase after stopping
        assert_eq!(count_at_stop, count_after);
    }
}
