

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
use std::ptr::copy_nonoverlapping;
use crate::traits::Queue as QueueTrait;
use crate::osal::queue::ffi::{clock_gettime, pthread_cond_destroy, pthread_cond_signal, pthread_cond_timedwait, pthread_cond_wait, pthread_mutex_destroy, pthread_mutex_lock, pthread_mutex_unlock};
use crate::posix::queue::ffi::{pthread_cond_t, pthread_mutexattr_t, pthread_mutex_t, pthread_condattr_init, pthread_condattr_setclock, pthread_cond_init, pthread_mutex_init, pthread_mutexattr_init, pthread_mutexattr_setprotocol, CLOCK_MONOTONIC, PTHREAD_PRIO_INHERIT, timespec};
use crate::types::NSECS_PER_SEC;
use crate::{ErrorType, ErrorType::*, Error::Type, WAIT_FOREVER, Error};
use crate::Result;

macro_rules! timeout {
    ($self:expr, $rc:expr, $txt:expr) => {{
        pthread_mutex_unlock (&mut $self.mutex);
        pthread_cond_signal (&mut $self.cond);
        if $rc == OsEno {
            return Ok(());
        } else {
            return Err(Type($rc, $txt));
        }
    }};
}


pub struct Queue {
    cond: pthread_cond_t,
    mutex: pthread_mutex_t,
    r: usize,
    w: usize,
    count: usize,
    size: usize,
    message_size: usize,
    msg: Vec<u8>,
    buffer_size: usize
}

impl QueueTrait for Queue {

    fn new(size: usize, message_size: usize) -> Self where Self: Sized {

        let mut mattr: pthread_mutexattr_t = Default::default();
        let mut cattr = Default::default();
        let mut cond: pthread_cond_t = Default::default();
        let mut mutex: pthread_mutex_t = Default::default();



        unsafe {
            pthread_condattr_init (&mut cattr);
            pthread_condattr_setclock (&mut cattr, CLOCK_MONOTONIC as c_int);
            pthread_cond_init (&mut cond, &cattr);
            pthread_mutexattr_init (&mut mattr);
            pthread_mutexattr_setprotocol (&mut mattr, PTHREAD_PRIO_INHERIT as c_int);
            pthread_mutex_init (&mut mutex, &mattr);
        }

        Self {
            cond,
            mutex,
            r: 0,
            w: 0,
            count: 0,
            size,
            message_size,
            msg: vec![0; size * message_size],
            buffer_size: size * message_size
        }
    }

    fn fetch<T>(&mut self, msg: &mut T, time: u64 ) -> Result<()>
    where
        T: Sized
    {
        let mut ts: timespec = Default::default();
        let mut nsec = time * 1_000_000;

        if time != WAIT_FOREVER {
            unsafe {
                clock_gettime(CLOCK_MONOTONIC as i32, &mut ts);
            }
            nsec += ts.tv_nsec as u64;

            ts.tv_sec += (nsec / NSECS_PER_SEC) as i64;
            ts.tv_nsec = (nsec % NSECS_PER_SEC) as i64;
        }

        unsafe {
            pthread_mutex_lock (&mut self.mutex);

            while self.count == 0
            {
                if time != WAIT_FOREVER
                {
                    match ErrorType::new(pthread_cond_timedwait (&mut self.cond, &mut self.mutex, &ts)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, "The time specified by abstime to pthread_cond_timedwait() has passed."),
                        OsEinval => timeout!(self, OsEinval, "The value specified by abstime is invalid."),
                        OsEperm => timeout!(self, OsEperm, "The mutex was not owned by the current thread at the time of the call."),
                        _ => timeout!(self, OsGenerr, "Unhandled error."),
                    }
                } else {
                    match ErrorType::new(pthread_cond_wait (&mut self.cond, &mut self.mutex)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, "The time specified by abstime to pthread_cond_wait() has passed."),
                        OsEinval => timeout!(self, OsEinval, "The value specified by abstime is invalid."),
                        _ => timeout!(self, OsGenerr, "Unhandled error."),
                    }
                }
            }


            if self.count == 0 {
                return Err(Error::Std(-1, "Message size 0."));
            }

            // Copy message from buffer to msg
            let src_offset = self.r * self.message_size;
            let src_slice = &self.msg[src_offset..src_offset + self.message_size];
            let msg_ptr = msg as *mut T as *mut u8;
            copy_nonoverlapping(src_slice.as_ptr(), msg_ptr, self.message_size);

            // Update read position and count
            self.r = (self.r + 1) % self.size;
            self.count -= 1;

            pthread_mutex_unlock(&mut self.mutex);
            pthread_cond_signal(&mut self.cond);
        }

        Ok(())
    }

    fn fetch_from_isr<T>(&mut self, msg: &mut T, time: u64) -> Result<()>
    where
        T: Sized,
    {
        self.fetch(msg, time)
    }
    fn post<T>(&mut self, msg: T, time: u64) -> Result<()>
    where
        T: Sized
    {
        let mut ts: timespec = Default::default();
        let mut nsec = time * 1_000_000;

        if time != WAIT_FOREVER {
            unsafe {
                clock_gettime(CLOCK_MONOTONIC as c_int, &mut ts);
            }
            nsec += ts.tv_nsec as u64;

            ts.tv_sec += (nsec / NSECS_PER_SEC) as i64;
            ts.tv_nsec = (nsec % NSECS_PER_SEC) as i64;
        }

        unsafe {
            pthread_mutex_lock(&mut self.mutex);
            while self.count == self.size
            {

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

            // Copy message from msg to buffer
            let dst_offset = self.w * self.message_size;
            let dst_slice = &mut self.msg[dst_offset..dst_offset + self.message_size];
            let msg_ptr = &msg as *const T as *const u8;
            copy_nonoverlapping(msg_ptr, dst_slice.as_mut_ptr(), self.message_size);

            // Update write position and count
            self.w = (self.w + 1) % self.size;
            self.count += 1;

            pthread_mutex_unlock(&mut self.mutex);
            pthread_cond_signal(&mut self.cond);
        }

        Ok(())
    }

    fn post_from_isr<T>(&mut self, msg: T, time: u64) -> Result<()>
    where
        T: Sized,
    {
        self.post(msg, time)
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            pthread_cond_destroy (&mut self.cond);
            pthread_mutex_destroy (&mut self.mutex);
        }

        if !self.msg.is_empty()
        {
            for i in 0..self.buffer_size
            {
                self.msg[i] = 0;
            }
        }
    }
}
