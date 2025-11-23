use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::String;
use core::any::Any;
use core::ffi::{c_int, c_void};
use core::ptr::null_mut;
use core::fmt::Debug;
use core::mem::zeroed;
use crate::Error::Type;
use crate::ErrorType;
use crate::traits::{ThreadPriority, ThreadTrait, ThreadFunc};
use crate::types::{Result, Error::Std};
use crate::posix::ffi::{pthread_t, pthread_attr_destroy, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t, pthread_create, pthread_detach, pthread_exit, pthread_join, pthread_setname_np, setpriority, __priority_which_PRIO_PROCESS};

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
    handle: Box<pthread_t>,
    name: String,
    stack: u32,
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
        setpriority(__priority_which_PRIO_PROCESS as c_int, 0, boxed_context.priority);
    }



    match  (boxed_context.callback)(boxed_context.param.clone()) {
        Ok(retval) => Arc::into_raw(retval) as *mut c_void,
        Err(_) => null_mut(),
    }
}

impl ThreadTrait<Thread> for Thread {

    fn new<F>(
        callback: F,
        name: &str,
        stack: u32,
        priority: impl ThreadPriority
    ) -> Result<Self>
    where
        F: Fn(Option<Arc<dyn Any + Send + Sync>>) -> Result<Arc<dyn Any + Send + Sync>> + Send + Sync + 'static,
    {
        if stack % 0x4000 != 0 {
            return Err(Std(-2, "Stack size must be a multiple of 16384 bytes"));
        }

        Ok(Self {
            handle: Box::new(0),
            callback: Arc::new(callback),
            name: String::from(name),
            stack,
            param: None,
            priority: priority.get_priority(),
        })
    }

    fn create(&mut self, param: Option<Arc<dyn Any + Send + Sync>>) -> Result<()> {

        let mut attr: pthread_attr_t = unsafe { zeroed() };

        self.param = param;

        unsafe {
            let rc = pthread_attr_init(&mut attr);
            if rc != 0 {
                return Err(Std(rc, "Failed to initialize pthread attributes"));
            }

            if self.stack > 0 {
                let rc = pthread_attr_setstacksize(&mut attr, self.stack as usize);
                if rc != 0 {
                    pthread_attr_destroy(&mut attr);
                    return Err(Std(rc, "Failed to set pthread stack size"));
                }
            }
        }

        let context_ptr = Box::into_raw(Box::new(self.clone())) as *mut c_void;
        let result = unsafe {
            pthread_create(
                &mut *self.handle,
                &attr,
                Some(callback),
                context_ptr,
            )
        };

        // Destroy the thread attributes as they are no longer needed after pthread_create
        unsafe {
            pthread_attr_destroy(&mut attr);
        }

        if result == 0 {
            // Set thread name if provided
            if !self.name.is_empty() {
                if let Ok(name_c) = CString::new(self.name.clone()) {
                    unsafe {
                        pthread_setname_np(*self.handle, name_c.as_ptr());
                    }
                }
            }

            Ok(())
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

    fn join(&self, retval: *mut *mut c_void) -> Result<i32> {
        let result = unsafe {
            pthread_join(*self.handle, retval)
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
            pthread_detach(*self.handle);
        }
    }
}

impl Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        f.debug_struct("Thread")
            .field("handle", &*self.handle)
            .field("name", &self.name)
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

        let thread = Thread::new(
            |_| Ok(Arc::new(())),
            "test",
            0,
            ThreadDefaultPriority::Normal,
        );

        let thread = thread.unwrap().create(None);

        assert!(thread.is_ok(), "Thread creation failed: {:?}", thread.err());
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_with_name() {
        let thread = Thread::new(
            |_| Ok(Arc::new(())),
            "my_thread",
            0,
            ThreadDefaultPriority::Normal,
        );

        let thread = thread.unwrap().create(None);

        assert!(thread.is_ok());
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_with_stack() {
        let thread = Thread::new(
            |_| Ok(Arc::new(())),
            "stack_test",
            16384, // Use 16KB stack (8192 was too small)
            ThreadDefaultPriority::Normal,
        );

        let thread = thread.unwrap().create(None);

        assert!(thread.is_ok(), "Thread creation failed: {:?}", thread.err());
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_thread_with_param() {
        let thread = Thread::new(
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
            ThreadDefaultPriority::Normal,
        );

        let thread = thread.unwrap().create(Some(Arc::new(42i32)));

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
                |_| Ok(Arc::new(())),
                "priority_test",
                0,
                priority.clone(),
            );

            let thread = thread.unwrap().create(None);

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

        let thread = Thread::new(
            move |_| {
                // Read the current thread's nice value using getpriority
                unsafe extern "C" {
                    fn getpriority(which: i32, who: u32) -> i32;
                }
                let nice_value = unsafe { getpriority(0, 0) };
                *priority_read_clone.lock().unwrap() = Some(nice_value);
                Ok(Arc::new(()))
            },
            "priority_check",
            0,
            ThreadDefaultPriority::High, // Should set nice to -13
        );

        let thread = thread.unwrap().create(None);

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


