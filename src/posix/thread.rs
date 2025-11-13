mod ffi {
    use core::ffi::{c_char, c_int, c_void};

    pub const SCHED_OTHER : c_int = 0;
    pub const SCHED_FIFO: c_int = 1;
    pub const SCHED_RR: c_int = 2;

    pub const PTHREAD_EXPLICIT_SCHED : c_int = 1;

    #[allow(non_camel_case_types)]
    pub type pthread_t = usize;

    #[allow(non_camel_case_types)]
    pub type pthread_attr_t = [u8; 56]; // Size may vary by platform

    #[allow(non_camel_case_types)]
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct sched_param {
        pub sched_priority: c_int,
    }

    unsafe extern "C" {

        pub fn pthread_attr_init (attr: *mut pthread_attr_t) -> i32;

        pub fn  pthread_attr_destroy(attr: *mut pthread_attr_t) -> i32;

        pub fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: i32) -> i32;

        pub fn pthread_attr_getschedpolicy(attr: *const pthread_attr_t, policy: *mut i32) -> i32;

        pub fn pthread_attr_setschedparam(attr: *mut pthread_attr_t, param: *const sched_param ) -> c_int;

        pub fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inherit: c_int,) -> c_int;

        pub fn pthread_create(
            thread: *mut pthread_t,
            attr: *const pthread_attr_t,
            start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
            arg: *mut c_void,
        ) -> c_int;

        pub fn pthread_detach(thread: pthread_t) -> c_int;

        pub fn pthread_exit(retval: *mut c_void) -> !;

        pub fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> c_int;

        pub fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: usize) -> c_int;

        pub fn pthread_attr_setstacksize (attr: *mut pthread_attr_t, stacksize: usize) -> i32;

        pub fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int;

    }

}

use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::ffi::CString;
use core::any::Any;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use core::fmt::Debug;
use core::mem::zeroed;
use crate::traits::ThreadPriority;
use crate::traits::Thread as ThreadTrait;
use crate::commons::ThreadFunc;
use crate::posix::thread::ffi::{pthread_attr_t, pthread_create, pthread_detach, pthread_exit, pthread_setname_np, pthread_t, pthread_attr_setschedpolicy, pthread_attr_init, pthread_join, pthread_attr_setstacksize, pthread_getname_np, pthread_attr_setinheritsched, pthread_attr_setschedparam, sched_param, SCHED_RR, pthread_attr_destroy};

#[derive(Clone)]
pub enum ThreadDefaultPriority {
    None = 0,
    Idle = 15,
    Low = 25,
    BelowNormal = 40,
    Normal = 50,
    AboveNormal = 60,
    High = 70,
    Realtime = 80,
    ISR = 99,
}

impl ThreadPriority for ThreadDefaultPriority {
    fn get_priority(&self) -> i32 {
        self.clone() as i32
    }
}


pub struct Thread {
    handle: pthread_t
}

struct ThreadContext {
    callback: Arc<ThreadFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>,
}

extern "C" fn callback(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut();
    }

    // Recreate the Box<ThreadContext> we passed to pthread_create and run the callback.
    let boxed_context: Box<ThreadContext> = unsafe { Box::from_raw(param_ptr as *mut ThreadContext) };

    let param_arc: Arc<dyn Any + Send + Sync> = boxed_context
        .param
        .clone()
        .unwrap_or_else(|| Arc::new(()) as Arc<dyn Any + Send + Sync>);

    (boxed_context.callback)(param_arc);

    null_mut()
}

impl ThreadTrait<Thread> for Thread {
    fn new<F>(
        callback: F,
        name: &str,
        stack: u32,
        param: Option<Arc<dyn Any + Send + Sync>>,
        priority: impl ThreadPriority
    ) -> Result<Self, (&'static str)>
    where
        F: Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        let callback_arc = Arc::new(callback);
        let mut handle: pthread_t = 0;
        let mut attr: pthread_attr_t = unsafe { zeroed() };
        let mut err = 0i32;

        unsafe {
            let rc = pthread_attr_init(&mut attr);
            if rc != 0 {
                return Err("Failed to initialize pthread attributes");
            }

            if priority.get_priority() > 0 {
                let rc = pthread_attr_setschedpolicy(&mut attr, SCHED_RR as i32);
                if rc != 0 {
                    return Err("Failed to set pthread scheduling policy");
                }

                let param = sched_param {
                    sched_priority: priority.get_priority()
                };

                let rc = pthread_attr_setschedparam(&mut attr, &param);
                if rc != 0 {
                    return Err("Failed to set pthread scheduling parameters");
                }

                let rc = pthread_attr_setinheritsched(&mut attr, ffi::PTHREAD_EXPLICIT_SCHED);
                if rc != 0 {
                    return Err("Failed to set pthread inherit scheduler attribute");
                }
            }

            if stack > 0 {
                let rc = pthread_attr_setstacksize(&mut attr, stack as usize);
                if rc != 0 {
                    return Err("Failed to set pthread stack size");
                }
            }
        }

        let context_ptr = Box::into_raw(Box::new(ThreadContext {
            callback: callback_arc.clone(),
            param: param.clone(),
        }));

        let result = unsafe {
            pthread_create(
                &mut handle,
                &attr,
                crate::posix::thread::callback,
                context_ptr as *mut c_void,
            )
        };

        // Destroy the thread attributes as they are no longer needed after pthread_create
        unsafe {
            pthread_attr_destroy(&mut attr);
        }

        if result == 0 {
            // Set thread name if provided
            if !name.is_empty() {
                if let Ok(name_c) = CString::new(name) {
                    unsafe {
                        pthread_setname_np(handle, name_c.as_ptr());
                    }
                }
            }

            Ok(Thread {
                handle,
            })
        } else {
            // Ripristina il Box per evitare memory leak
            _ = unsafe { Box::from_raw(context_ptr) };
            Err("Failed to create pthread")
        }
    }

    fn delete_current() {
        unsafe {
            pthread_exit(null_mut());
        }
    }

    fn suspend(&self) {
        // POSIX threads don't have a direct suspend/resume mechanism
        // This would require custom signal handling or condition variables
        // For now, this is a no-op
    }

    fn resume(&self) {
        // POSIX threads don't have a direct suspend/resume mechanism
        // This would require custom signal handling or condition variables
        // For now, this is a no-op
    }

    fn join(&self, mut retval: *mut c_void) -> Result<(), &'static str> {
        let result = unsafe {
            pthread_join(self.handle, &mut retval)
        };

        if result == 0 {
            Ok(())
        } else {
            Err("Failed to join pthread")
        }
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        unsafe {
            pthread_detach(self.handle);
        }
    }
}

impl Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut name: [c_char; 16] = [0; 16];
        unsafe {
            pthread_getname_np(self.handle, name.as_mut_ptr(), name.len());
        }

        // Convert the C string to a Rust string safely without taking ownership
        let name_str = unsafe {
            let cstr = core::ffi::CStr::from_ptr(name.as_ptr());
            cstr.to_str().unwrap_or("<invalid>")
        };

        f.debug_struct("Thread")
            .field("handle", &self.handle)
            .field("name", &name_str)
            .finish()
    }
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::thread;
    use std::time::Duration;
    use alloc::vec::Vec;

    #[test]
    fn test_thread_priority_mapping() {
        assert_eq!(ThreadDefaultPriority::None.get_priority(), 0);
        assert_eq!(ThreadDefaultPriority::Idle.get_priority(), 15);
        assert_eq!(ThreadDefaultPriority::Low.get_priority(), 25);
        assert_eq!(ThreadDefaultPriority::BelowNormal.get_priority(), 40);
        assert_eq!(ThreadDefaultPriority::Normal.get_priority(), 50);
        assert_eq!(ThreadDefaultPriority::AboveNormal.get_priority(), 60);
        assert_eq!(ThreadDefaultPriority::High.get_priority(), 70);
        assert_eq!(ThreadDefaultPriority::Realtime.get_priority(), 80);
        assert_eq!(ThreadDefaultPriority::ISR.get_priority(), 99);
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_basic_thread_creation() {
        let thread = Thread::new(
            |_| Arc::new(()),
            "test",
            0,
            None,
            ThreadDefaultPriority::Normal,
        );
        assert!(thread.is_ok(), "Thread creation failed: {:?}", thread.err());
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_with_name() {
        let thread = Thread::new(
            |_| Arc::new(()),
            "my_thread",
            0,
            None,
            ThreadDefaultPriority::Normal,
        );
        assert!(thread.is_ok());
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_with_stack() {
        let thread = Thread::new(
            |_| Arc::new(()),
            "stack_test",
            16384, // Use 16KB stack (8192 was too small)
            None,
            ThreadDefaultPriority::Normal,
        );
        assert!(thread.is_ok(), "Thread creation failed: {:?}", thread.err());
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_with_param() {
        let thread = Thread::new(
            |param| {
                if let Some(val) = param.downcast_ref::<i32>() {
                    assert_eq!(*val, 42);
                }
                Arc::new(())
            },
            "param_test",
            0,
            Some(Arc::new(42i32)),
            ThreadDefaultPriority::Normal,
        );
        assert!(thread.is_ok());

        // Give thread time to execute
        thread::sleep(Duration::from_millis(50));
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_priorities() {
        let priorities = Vec::from([
            ThreadDefaultPriority::Idle,
            ThreadDefaultPriority::Low,
            ThreadDefaultPriority::BelowNormal,
            ThreadDefaultPriority::Normal,
            ThreadDefaultPriority::AboveNormal,
            ThreadDefaultPriority::High,
            ThreadDefaultPriority::Realtime,
        ]);

        for priority in priorities {
            let thread = Thread::new(
                |_| Arc::new(()),
                "priority_test",
                0,
                None,
                priority.clone(),
            );
            assert!(thread.is_ok(), "Failed to create thread with priority: {}", priority.get_priority());
        }
    }
}


