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
    impl Default for pthread_mutexattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_condattr_t {
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

use core::ffi::c_int;
use core::fmt::Debug;
use crate::Result;
use crate::traits::Event as EventTrait;
use crate::posix::event::ffi::{
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
        let mut ret = Self {
            cond: Default::default(),
            mutex: Default::default(),
            flags: 0,
        };
        let mut mattr: pthread_mutexattr_t = Default::default();
        let mut cattr: pthread_condattr_t = Default::default();

        unsafe {
            pthread_mutex_init(&mut ret.mutex, &mattr);
            pthread_condattr_setclock (&mut cattr, CLOCK_MONOTONIC as c_int);
            pthread_cond_init(&mut ret.cond, &mut cattr);
            pthread_mutexattr_init (&mut mattr);
            pthread_mutexattr_setprotocol (&mut mattr, PTHREAD_PRIO_INHERIT as c_int);
            pthread_mutex_init (&mut ret.mutex, &mattr);
        }

        ret
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
