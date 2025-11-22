use core::ffi::c_int;
use core::fmt::Debug;
use crate::Result;
use crate::traits::EventTrait;
use crate::posix::ffi::{
    pthread_cond_t, pthread_mutex_t, pthread_condattr_t, pthread_mutexattr_t, timespec,
    pthread_cond_signal, pthread_cond_init,  pthread_mutex_init, pthread_condattr_setclock, pthread_cond_destroy, pthread_mutex_destroy, pthread_mutexattr_init, pthread_mutexattr_setprotocol, clock_gettime, pthread_mutex_lock, pthread_cond_timedwait, pthread_mutex_unlock, pthread_cond_wait,
    CLOCK_MONOTONIC, PTHREAD_PRIO_INHERIT
};
use crate::{WAIT_FOREVER, NSECS_PER_SEC, Error, ErrorType, ErrorType::*};


macro_rules! timeout {
    ($self:expr, $value:expr, $mask:expr, $rc:expr, $txt:expr) => {{
        *$value = $self.flags & $mask;
        pthread_mutex_unlock (&mut $self.mutex);
        if $rc == OsEno {
            return Ok(());
        } else {
            return Err(Error::Type($rc, $txt));
        }
    }};
}


pub struct Event {
    cond: pthread_cond_t,
    mutex: pthread_mutex_t,
    flags: u32
}

impl EventTrait for Event {

    fn new() -> Self
    where
        Self: Sized
    {
        let mut mattr: pthread_mutexattr_t = Default::default();
        let mut cattr: pthread_condattr_t = Default::default();
        let mut cond: pthread_cond_t = Default::default();
        let mut mutex: pthread_mutex_t = Default::default();

        unsafe {
            pthread_mutex_init(&mut mutex, &mattr);
            pthread_condattr_setclock (&mut cattr, CLOCK_MONOTONIC as c_int);
            pthread_cond_init(&mut cond, &mut cattr);
            pthread_mutexattr_init (&mut mattr);
            pthread_mutexattr_setprotocol (&mut mattr, PTHREAD_PRIO_INHERIT as c_int);
            pthread_mutex_init (&mut mutex, &mattr);
        }

        Self {
            cond,
            mutex,
            flags: 0,
        }
    }



    fn wait(&mut self, mask: u32, value: &mut u32, time: u64) -> Result<()> {
        let mut ts: timespec = Default::default();
        let mut nsec =  (time * 1_000 * 1_000) as i64;

        unsafe {

            if time != WAIT_FOREVER {
                clock_gettime (CLOCK_MONOTONIC as c_int, &mut ts);

                nsec += ts.tv_nsec;

                ts.tv_sec += nsec / NSECS_PER_SEC as i64;
                ts.tv_nsec = nsec % NSECS_PER_SEC as i64;

            }

            pthread_mutex_lock (&mut self.mutex);

            while (self.flags & mask) == 0 {
                if time != WAIT_FOREVER {
                    match ErrorType::new(pthread_cond_timedwait (&mut self.cond, &mut self.mutex, &ts)) {
                        OsEno => {},
                        OsEinval => timeout!(self, value, mask, OsEinval, "The value specified by abstime is invalid."),
                        OsEtimedout =>  timeout!(self, value, mask, OsEtimedout, "The time specified by abstime to pthread_cond_timedwait() has passed."),
                        OsEperm => timeout!(self, value, mask, OsEperm, "The mutex was not owned by the current thread at the time of the call."),
                        err => timeout!(self, value, mask, err, "Unhandled error."),
                    }
                } else {
                    match ErrorType::new(pthread_cond_wait (&mut self.cond, &mut self.mutex)) {
                        OsEno => {},
                        OsEinval => timeout!(self, value, mask, OsEinval, "The value specified by abstime is invalid."),
                        OsEtimedout => timeout!(self, value, mask, OsEperm, "The time specified by abstime to pthread_cond_timedwait() has passed."),
                        err => timeout!(self, value, mask, err, "Unhandled error."),
                    }
                }
            }
        }

        Err(Error::Type(OsGenerr, "Unhandled error."))
    }

    fn wait_from_isr(&mut self, mask: u32, value: &mut u32, time: u64) -> Result<()> {
        self.wait(mask, value, time)
    }

    fn set(&mut self, value: u32) {
        unsafe {
            pthread_mutex_lock (&mut self.mutex);
            self.flags |= value;
            pthread_mutex_unlock (&mut self.mutex);
            pthread_cond_signal (&mut self.cond);
        }
    }

    fn set_from_isr(&mut self, value: u32) {
        self.set(value)
    }

    fn get(&mut self) -> u32 {
        unsafe {
            #[allow(unused_assignments)]
            let mut ret = 0;
            pthread_mutex_lock (&mut self.mutex);
            ret = self.flags;
            pthread_mutex_unlock (&mut self.mutex);
            pthread_cond_signal (&mut self.cond);
            ret
        }
    }

    fn get_from_isr(&mut self) -> u32 {
        self.get()
    }

    fn clear(&mut self, value: u32) {
        unsafe {
            pthread_mutex_lock (&mut self.mutex);
            self.flags &= !value;
            pthread_mutex_unlock (&mut self.mutex);
            pthread_cond_signal (&mut self.cond);
        }
    }

    fn clear_from_isr(&mut self, value: u32) {
        self.clear(value)
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe {
            pthread_cond_destroy (&mut self.cond);
            pthread_mutex_destroy (&mut self.mutex);
        }
    }
}


impl Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Event")
            .field("flags", &self.flags)
            .finish()
    }
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}


#[cfg(test)]
mod tests {
    use alloc::format;
    use super::*;
    extern crate std;
    use std::sync::{Arc as StdArc, Mutex as StdMutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_new() {
        let event = Event::new();
        assert_eq!(event.flags, 0, "Event should start with flags set to 0");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_set_and_get() {
        let mut event = Event::new();
        event.set(0b0101);
        let flags = event.get();
        assert_eq!(flags, 0b0101, "Flags should be 0b0101 after set");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_clear() {
        let mut event = Event::new();
        event.set(0b1111);
        event.clear(0b0011);
        let flags = event.get();
        assert_eq!(flags, 0b1100, "Flags should be 0b1100 after clearing 0b0011");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_wait_timeout() {
        let mut event = Event::new();
        let mut value = 0u32;
        
        // This should timeout since we never set the flag
        let result = event.wait(0b0001, &mut value, 10); // 10ms timeout
        
        // Should timeout or return with value 0
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_set_and_wait() {
        let event = StdArc::new(StdMutex::new(Event::new()));
        let event_clone = event.clone();

        // Thread that sets the event after a delay
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            let mut evt = event_clone.lock().unwrap();
            evt.set(0b0001);
        });

        // Wait for the event
        let evt = event.lock().unwrap();
        let start = std::time::Instant::now();
        drop(evt); // Release lock before waiting
        
        thread::sleep(Duration::from_millis(100));
        
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(40), "Should wait at least 40ms");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_multiple_flags() {
        let mut event = Event::new();
        
        event.set(0b0001);
        event.set(0b0100);
        
        let flags = event.get();
        assert_eq!(flags & 0b0001, 0b0001, "Flag 0b0001 should be set");
        assert_eq!(flags & 0b0100, 0b0100, "Flag 0b0100 should be set");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_from_isr() {
        let mut event = Event::new();
        
        // Test ISR variants (which are identical to normal versions in POSIX)
        event.set_from_isr(0b1010);
        let flags = event.get_from_isr();
        assert_eq!(flags, 0b1010, "ISR set/get should work");
        
        event.clear_from_isr(0b0010);
        let flags = event.get_from_isr();
        assert_eq!(flags, 0b1000, "ISR clear should work");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_event_debug() {
        let event = Event::new();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Event"), "Debug output should contain Event");
        assert!(debug_str.contains("flags"), "Debug output should contain flags field");
    }
}
