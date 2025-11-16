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
use crate::{Result, Error, ErrorType, ErrorType::*};
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
                _ => todo!(),
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