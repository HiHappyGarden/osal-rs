use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::sync::Arc;
use core::any::Any;
use core::ffi::{c_char, c_ushort, c_void};
use core::fmt::Debug;
use core::ptr::{null, null_mut};
use crate::freertos::ffi::{TaskHandle_t, xTaskCreate};
use crate::types::{Result};
use crate::types::Error::Std;
use crate::traits::{ThreadTrait, ThreadFunc, ThreadPriority};

#[derive(Clone)]
#[repr(i32)]
pub enum ThreadDefaultPriority {
    None = 0,
    Idle = 1,
    Low = 2,
    BelowNormal = 3,
    Normal = 4,
    AboveNormal = 5,
    High = 6,
    Realtime = 7,
    ISR = 8,
}



impl ThreadPriority for ThreadDefaultPriority {
    fn get_priority(&self) -> i32 {
        self.clone() as i32
    }
}

#[derive(Clone)]
pub struct Thread {
    handler: TaskHandle_t,
    callback: Arc<ThreadFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>
}

unsafe extern "C" fn callback(param_ptr: *mut c_void) {
    // if param_ptr.is_null() {
    //     return;
    // }

    // // Recreate the Box\<Thread\> we passed to the RTOS and run the callback.
    // let boxed_thread: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut Thread) };

    // let param_arc: Arc<dyn Any + Send + Sync> = boxed_thread
    //     .param
    //     .clone()
    //     .unwrap_or_else(|| Arc::new(()) as Arc<dyn Any + Send + Sync>);

    // (boxed_thread.callback)(param_arc);
}

impl ThreadTrait<Thread> for Thread {

    fn new<F>(callback: F,
            name: &str,
            stack: u32,
            priority: impl ThreadPriority
    ) -> Result<Thread>
    where
        F: Fn(Option<Arc<dyn Any + Send + Sync>>) -> Result<Arc<dyn Any + Send + Sync>> + Send + Sync + 'static
    {

        // let c_name = CString::new(name).map_err(|e| Std(format!("Failed to convert thread name to CString: {}", e)))?;

        // let thread = Thread {
        //     handler: null_mut(),
        //     callback: Arc::new(callback),
        //     param: None
        // };

        // let boxed_thread = Box::new(thread);

        // let ret = unsafe {
        //     xTaskCreate(
        //         Some(callback as extern "C" fn(*mut c_void)),
        //         c_name.as_ptr() as *const c_char,
        //         stack as c_ushort,
        //         Box::into_raw(boxed_thread) as *mut c_void,
        //         priority.get_priority() as u32,
        //         null_mut()
        //     )
        // };

        // if ret != 1 {
        //     return Err(Std(format!("Failed to create thread: xTaskCreate returned {}", ret)));
        // }

        // Ok(*boxed_thread)
        todo!()
    }
    

    fn create(&mut self, param: Option<Arc<dyn Any + Send + Sync>>) -> Result<()> {
        todo!()
    }

    fn delete_current() {
        todo!()
    }

    fn suspend(&self) {
        todo!()
    }

    fn resume(&self) {
        todo!()
    }

    fn join(&self, retval: *mut *mut c_void) -> Result<i32> {
        todo!()
    }
}