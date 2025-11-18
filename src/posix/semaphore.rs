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

    impl Default for pthread_cond_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_condattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_mutexattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for timespec {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }
}

use std::ffi::c_int;
use crate::posix::semaphore::ffi::{clock_gettime, pthread_cond_destroy, pthread_cond_init, pthread_cond_signal, pthread_cond_t, pthread_cond_timedwait, pthread_cond_wait, pthread_condattr_init, pthread_condattr_setclock, pthread_condattr_t, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock, pthread_mutexattr_init, pthread_mutexattr_setprotocol, pthread_mutexattr_t, timespec, CLOCK_MONOTONIC, PTHREAD_PRIO_INHERIT};
use crate::traits::Semaphore as SemaphoreTrait;
use crate::types::NSECS_PER_SEC;
use crate::{ErrorType, ErrorType::*, Error::Type, WAIT_FOREVER, Result};

macro_rules! timeout {
    ($self:expr, $rc:expr, $txt:expr) => {{
        pthread_mutex_unlock (&mut $self.mutex);
        if $rc == OsEno {
            return Ok(());
        } else {
            return Err(Type($rc, $txt));
        }
    }};
}


pub struct Semaphore {
    cond: pthread_cond_t,
    mutex: pthread_mutex_t,
    count: usize
}

impl SemaphoreTrait for Semaphore {
    fn new(count: usize) -> Self {

        let mut mattr: pthread_mutexattr_t = Default::default();
        let mut cattr: pthread_condattr_t = Default::default();

        let mut ret = Self {
            cond: Default::default(),
            mutex: Default::default(),
            count: 0
        };

        unsafe {
            pthread_condattr_init (&mut cattr);
            pthread_condattr_setclock (&mut cattr, CLOCK_MONOTONIC as c_int);
            pthread_cond_init (&mut ret.cond, &cattr);
            pthread_mutexattr_init (&mut mattr);
            pthread_mutexattr_setprotocol (&mut mattr, PTHREAD_PRIO_INHERIT as c_int);
            pthread_mutex_init (&mut ret.mutex, &mattr);
            ret.count = count;
        }

        ret
    }

    fn wait(&mut self, time: u64) -> Result<()> {
        let mut ts: timespec = Default::default();
        let mut nsec = time * 1_000_000;

        unsafe {
            clock_gettime (CLOCK_MONOTONIC as c_int, &mut ts);
        }
        nsec += ts.tv_nsec as u64;
        if nsec > NSECS_PER_SEC
        {
            ts.tv_sec += (nsec / NSECS_PER_SEC) as i64;
            nsec %= NSECS_PER_SEC;
        }
        ts.tv_nsec = nsec as i64;

        unsafe {
            while self.count == 0 {
                if time != WAIT_FOREVER {
                    match ErrorType::new(pthread_cond_timedwait (&mut self.cond, &mut self.mutex, &ts)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, "The time specified by abstime to pthread_cond_timedwait() has passed."),
                        OsEinval => timeout!(self, OsEinval, "The value specified by abstime is invalid."),
                        OsEperm => timeout!(self, OsEperm, "The mutex was not owned by the current thread at the time of the call."),
                        err => timeout!(self, err, "Unhandled error."),
                    }
                } else {
                    match ErrorType::new(pthread_cond_wait (&mut self.cond, &mut self.mutex)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, "The time specified by abstime to pthread_cond_wait() has passed."),
                        OsEinval => timeout!(self, OsEinval, "The value specified by abstime is invalid."),
                        err => timeout!(self, err, "Unhandled error."),
                    }
                }
            }
        }

        self.count -= 1;

        Ok(())
    }

    fn wait_from_isr(&mut self, time: u64) -> Result<()> {
        self.wait(time)
    }

    fn signal(&mut self) {
        unsafe {
            pthread_mutex_lock (&mut self.mutex);
            self.count += 1;
            pthread_mutex_unlock (&mut self.mutex);
            pthread_cond_signal (&mut self.cond);
        }
    }

    fn signal_from_isr(&mut self) {
        self.signal()
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            pthread_cond_destroy (&mut self.cond);
            pthread_mutex_destroy (&mut self.mutex);
        }
    }}