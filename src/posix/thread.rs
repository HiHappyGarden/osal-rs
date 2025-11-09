use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::ffi::CString;
use core::any::Any;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::fmt::Debug;
use crate::traits::ThreadPriority;
pub use crate::traits::Thread as ThreadTrait;
use crate::commons::{ThreadDefaultPriority, ThreadFunc};
use crate::posix::thread::ffi::{pthread_create, pthread_detach, pthread_exit, pthread_setname_np, pthread_t};

mod ffi {
    use core::ffi::{c_char, c_int, c_void};

    #[allow(non_camel_case_types)]
    pub type pthread_t = usize;

    #[allow(non_camel_case_types)]
    pub type pthread_attr_t = [u8; 56]; // Size may vary by platform

    unsafe extern "C" {
        pub fn pthread_create(
            thread: *mut pthread_t,
            attr: *const pthread_attr_t,
            start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
            arg: *mut c_void,
        ) -> c_int;

        pub fn pthread_detach(thread: pthread_t) -> c_int;
        pub fn pthread_exit(retval: *mut c_void) -> !;
        pub fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> c_int;
    }

}

impl ThreadPriority for ThreadDefaultPriority {
    fn get_priority(&self) -> u32 {
        self.clone() as u32
    }
}


pub struct Thread {
    handle: pthread_t,
    callback: Arc<ThreadFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>,
}

extern "C" fn callback(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut();
    }

    // Recreate the Box<Thread> we passed to pthread_create and run the callback.
    let boxed_thread: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut Thread) };

    let param_arc: Arc<dyn Any + Send + Sync> = boxed_thread
        .param
        .clone()
        .unwrap_or_else(|| Arc::new(()) as Arc<dyn Any + Send + Sync>);

    (boxed_thread.callback)(param_arc);

    null_mut()
}

impl ThreadTrait<Thread> for Thread {
    fn new<F>(
        callback: F,
        name: &str,
        _stack: u32,
        param: Option<Arc<dyn Any + Send + Sync>>,
        _priority: impl ThreadPriority
    ) -> Result<Self, &'static str>
    where
        F: Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        let mut handle: pthread_t = 0;
        let callback_arc: Arc<ThreadFunc> = Arc::new(callback);

        let thread = Thread {
            handle,
            callback: callback_arc.clone(),
            param: param.clone(),
        };
        let thread_box = Box::new(thread);

        let result = unsafe {
            pthread_create(
                &mut handle,
                null_mut(),
                crate::posix::thread::callback,
                Box::into_raw(thread_box) as *mut c_void,
            )
        };

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
                callback: callback_arc,
                param
            })
        } else {
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
        f.debug_struct("Thread")
            .field("handle", &self.handle)
            .finish()
    }
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}


