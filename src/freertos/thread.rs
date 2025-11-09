#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    use core::ffi::{c_char, c_void};
    use crate::freertos::ffi::{BaseType_t, TickType_t, UBaseType_t};

    pub type TaskHandle_t = *mut c_void;

    pub type TaskFunction_t = unsafe extern "C" fn(*mut c_void);

    unsafe extern "C" {

        // Task Management
        pub fn xTaskCreate(
            pvTaskCode: TaskFunction_t,
            pcName: *const c_char,
            usStackDepth: u16,
            pvParameters: *mut c_void,
            uxPriority: UBaseType_t,
            pxCreatedTask: *mut TaskHandle_t,
        ) -> BaseType_t;

        pub fn vTaskDelete(xTaskToDelete: TaskHandle_t);
        pub fn vTaskSuspend(xTaskToSuspend: TaskHandle_t);
        pub fn vTaskResume(xTaskToResume: TaskHandle_t);
    }
}

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::sync::Arc;
use core::any::Any;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use crate::freertos::ffi::{pdPASS};
use crate::freertos::thread::ffi::{xTaskCreate, TaskHandle_t};
use crate::osal::thread::ffi::{vTaskDelete, vTaskResume, vTaskSuspend};

pub trait ThreadPriority {
    fn get_priority(&self) -> u32;
}

#[derive(Clone)]
pub enum ThreadDefaultPriority {
    None,
    Idle,
    Low,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
    ISR,
}

impl ThreadPriority for ThreadDefaultPriority {
    fn get_priority(&self) -> u32 {
        self.clone() as u32
    }
}

pub type ThreadFunc = dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static;

#[derive(Clone)]
pub struct Thread {
    handler: TaskHandle_t,
    callback: Arc<ThreadFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>
}

unsafe extern "C" fn callback(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    // Recreate the Box\<Thread\> we passed to the RTOS and run the callback.
    let boxed_thread: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut Thread) };

    let param_arc: Arc<dyn Any + Send + Sync> = boxed_thread
        .param
        .clone()
        .unwrap_or_else(|| Arc::new(()) as Arc<dyn Any + Send + Sync>);

    (boxed_thread.callback)(param_arc);
}


impl Thread {
    pub fn new<F>(
        callback: F,
        name: &str,
        stack: u32,
        param: Option<Arc<dyn Any + Send + Sync>>,
        priority: impl ThreadPriority
    ) -> Result<Self, &'static str>
    where
        F: Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        let name_c = CString::new(name).map_err(|_| "Name not valid")?;
        let name_ptr = name_c.as_ptr() as *const c_char;


        let mut handler = null_mut();
        let callback_arc: Arc<ThreadFunc> = Arc::new(callback);
        let thread =  Thread  {
            handler,
            callback: callback_arc.clone(),
            param: param.clone(),
        };
        let thread_box = Box::new(thread);

        let result = unsafe {
            xTaskCreate(
                crate::freertos::thread::callback,
                name_ptr,
                stack as u16,
                Box::into_raw(thread_box) as *mut c_void,
                priority.get_priority(),
                &mut handler,
            )
        };

        if result == pdPASS {
            Ok(Thread { handler, callback: callback_arc, param })
        } else {
            Err("Impossible create thread")
        }
    }

    pub fn delete(&self) {
        unsafe {
            vTaskDelete(self.handler);
        }
    }

    pub fn delete_current() {
        unsafe {
            vTaskDelete(null_mut());
        }
    }

    pub fn suspend(&self) {
        unsafe {
            vTaskSuspend(self.handler);
        }
    }

    pub fn resume(&self) {
        unsafe {
            vTaskResume(self.handler);
        }
    }

}

