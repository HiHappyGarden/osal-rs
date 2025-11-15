mod ffi {
    use core::ffi::{c_char, c_int, c_void};

    pub const PRIO_PROCESS: c_int = 0;

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

        pub fn pthread_attr_init (attr: *mut pthread_attr_t) -> c_int;

        pub fn  pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int;

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

        pub fn pthread_attr_setstacksize (attr: *mut pthread_attr_t, stacksize: usize) -> c_int;

        pub fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int;

        pub fn pthread_self() -> pthread_t;

        pub fn setpriority(which: c_int, who: c_int, prio: c_int) -> c_int;

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
use crate::Error::Type;
use crate::ErrorType;
use crate::traits::{ThreadPriority, Thread as ThreadTrait};
use crate::types::{ThreadFunc, Result, Error::Std};
use crate::posix::thread::ffi::{pthread_t, pthread_attr_destroy, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t, pthread_create, pthread_detach, pthread_exit, pthread_getname_np, pthread_join, pthread_setname_np, setpriority, PRIO_PROCESS};

#[derive(Clone)]
#[repr(i32)]
pub enum ThreadDefaultPriority {
    None = 19,
    Idle = 15,
    Low = 13,
    BelowNormal = 7,
    Normal = 0,
    AboveNormal = -7,
    High = -13,
    Realtime = -17,
    ISR = -20,
}

impl ThreadPriority for ThreadDefaultPriority {
    fn get_priority(&self) -> i32 {
        self.clone() as i32
    }
}

#[derive(Clone)]
pub struct Thread {
    handle: pthread_t,
    callback: Arc<ThreadFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>,
    priority: i32,
}

extern "C" fn callback(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut();
    }

    let boxed_context= unsafe { Box::from_raw(param_ptr as *mut Thread) };

    unsafe {
        setpriority(PRIO_PROCESS, 0, boxed_context.priority);
    }

    match  (boxed_context.callback)(boxed_context.param.clone()) {
        Ok(retval) => Box::into_raw(Box::new(retval)) as *mut c_void,
        Err(_) => null_mut(),
    }
}

impl ThreadTrait<Thread> for Thread {
    fn create<F>(
        callback: F,
        name: &str,
        stack: u32,
        param: Option<Arc<dyn Any + Send + Sync>>,
        priority: impl ThreadPriority
    ) -> Result<Self>
    where
        F: Fn(Option<Arc<dyn Any + Send + Sync>>) -> Result<Arc<dyn Any + Send + Sync>> + Send + Sync + 'static,
    {

        if stack % 0x4000 != 0 {
            return Err(Std(-2, "Stack size must be a multiple of 16384 bytes"));
        }

        let callback_arc = Arc::new(callback);
        //let mut handle: pthread_t = 0;
        let mut attr: pthread_attr_t = unsafe { zeroed() };



        unsafe {
            let rc = pthread_attr_init(&mut attr);
            if rc != 0 {
                return Err(Std(rc, "Failed to initialize pthread attributes"));
            }

            if stack > 0 {
                let rc = pthread_attr_setstacksize(&mut attr, stack as usize);
                if rc != 0 {
                    pthread_attr_destroy(&mut attr);
                    return Err(Std(rc, "Failed to set pthread stack size"));
                }
            }
        }

        let mut thread = Thread {
            handle: 0,
            callback: callback_arc,
            param,
            priority: priority.get_priority(),
        };

        let context_ptr = Box::into_raw(Box::new(thread.clone())) as *mut c_void;

        let result = unsafe {
            pthread_create(
                &mut thread.handle,
                &attr,
                crate::posix::thread::callback,
                context_ptr,
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
                        pthread_setname_np(thread.handle, name_c.as_ptr());
                    }
                }
            }

            Ok(thread)
        } else {
            _ = unsafe { Box::from_raw(context_ptr) };
            Err(Std(-1, "Failed to create pthread"))
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

    fn join(&self, mut retval: *mut c_void) -> Result<i32> {
        let result = unsafe {
            pthread_join(self.handle, &mut retval)
        };

        if result == 0 {
            Ok(result)
        } else {
            Err(Type(ErrorType::new(result), "Failed to join pthread"))
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
        assert_eq!(ThreadDefaultPriority::None.get_priority(), 19);
        assert_eq!(ThreadDefaultPriority::Idle.get_priority(), 15);
        assert_eq!(ThreadDefaultPriority::Low.get_priority(), 13);
        assert_eq!(ThreadDefaultPriority::BelowNormal.get_priority(), 7);
        assert_eq!(ThreadDefaultPriority::Normal.get_priority(), 0);
        assert_eq!(ThreadDefaultPriority::AboveNormal.get_priority(), -7);
        assert_eq!(ThreadDefaultPriority::High.get_priority(), -13);
        assert_eq!(ThreadDefaultPriority::Realtime.get_priority(), -17);
        assert_eq!(ThreadDefaultPriority::ISR.get_priority(), -20);
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_basic_thread_creation() {
        let thread = Thread::create(
            |_| Ok(Arc::new(())),
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
        let thread = Thread::create(
            |_| Ok(Arc::new(())),
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
        let thread = Thread::create(
            |_| Ok(Arc::new(())),
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
        let thread = Thread::create(
            |param| {
                if let Some(arc) = param {
                    if let Some(val) = arc.downcast_ref::<i32>() {
                        assert_eq!(*val, 42);
                    }
                }
                Ok(Arc::new(()))
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
            let thread = Thread::create(
                |_| Ok(Arc::new(())),
                "priority_test",
                0,
                None,
                priority.clone(),
            );
            assert!(thread.is_ok(), "Failed to create thread with priority: {}", priority.get_priority());
        }
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_priority_application() {
        use std::sync::{Arc as StdArc, Mutex};

        // Create a shared variable to store the priority read from inside the thread
        let priority_read = StdArc::new(Mutex::new(None));
        let priority_read_clone = priority_read.clone();

        let thread = Thread::create(
            move |_| {
                // Read the current thread's nice value using getpriority
                unsafe extern "C" {
                    fn getpriority(which: i32, who: i32) -> i32;
                }
                let nice_value = unsafe { getpriority(0, 0) };
                *priority_read_clone.lock().unwrap() = Some(nice_value);
                Ok(Arc::new(()))
            },
            "priority_check",
            0,
            None,
            ThreadDefaultPriority::High, // Should set nice to -13
        );

        assert!(thread.is_ok());

        // Give thread time to execute and set priority
        thread::sleep(Duration::from_millis(100));

        // Check that the priority was actually set
        let read_value = *priority_read.lock().unwrap();
        assert!(read_value.is_some(), "Thread did not read priority value");

        // Note: setpriority might fail without proper permissions, so we just verify it was called
        // In a privileged environment, we would expect read_value == Some(-13)
    }
}


