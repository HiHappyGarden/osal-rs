use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::fmt::format;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::any::Any;
use core::ffi::{c_char, c_ushort, c_void};
use core::fmt::Debug;
use core::ptr::{null_mut};
use crate::freertos::ffi::{StackType_t, TaskHandle_t, UBaseType_t, pdPASS, vTaskDelete, vTaskResume, vTaskSuspend, xTaskCreate};
use crate::types::{Result, Error::Std};
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
    name: String,
    stack: u32,
    priority: u32,
    handler: TaskHandle_t,
    callback: Arc<ThreadFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>,

}


unsafe extern "C" fn callback(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    let boxed_thread: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut Thread) };

    let _param_arc: Arc<dyn Any + Send + Sync> = boxed_thread
        .param
        .clone()
        .unwrap_or_else(|| Arc::new(()) as Arc<dyn Any + Send + Sync>);

    //(boxed_thread.callback)(param_arc);
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
        Ok(Thread {
            name: name.to_string(),
            stack,
            priority: priority.get_priority() as u32,
            handler: null_mut(),
            callback: Arc::new(callback),
            param: None
        })  
    }
    

    fn create(&mut self, param: Option<Arc<dyn Any + Send + Sync>>) -> Result<()> {

        let c_name = CString::new(self.name.clone()).map_err(|_| Std(-1, "Failed to convert thread name to CString"))?;

        let handler: *mut TaskHandle_t =  null_mut();

        let boxed_thread = Box::new(self.clone());

        let ret = unsafe {
            xTaskCreate(
                Some(super::thread::callback),
                c_name.as_ptr(),
                self.stack as StackType_t,
                Box::into_raw(boxed_thread) as *mut c_void,
                self.priority as UBaseType_t,
                handler as *mut TaskHandle_t
            )
        };

        if ret != pdPASS {
            return Err(Std(-2, "Failed to create thread: xTaskCreate returned {}"));
        }

        self.handler = unsafe { *handler };

        Ok(())

    }

    fn delete_current() {
        unsafe { vTaskDelete( null_mut() ); } 
    }

    fn suspend(&self) {
        if !self.handler.is_null() {
            unsafe { vTaskSuspend( self.handler ); } 
        }
    }

    fn resume(&self) {
        if !self.handler.is_null() {
            unsafe { vTaskResume( self.handler ); } 
        }
    }

    fn join(&self, _: *mut *mut c_void) -> Result<i32> {
        if self.handler.is_null() {
            return Err(Std(-1, "Thread handler is null"));
        }
        unsafe { vTaskDelete( self.handler ); } 
        Ok(0)
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        if !self.handler.is_null() {
            unsafe { vTaskDelete( self.handler ); } 
        }
    }
}

impl Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thread")
            .field("name", &self.name)
            .field("stack", &self.stack)
            .field("priority", &self.priority)
            .field("handler", &self.handler)
            .finish()
    }
}