use crate::freertos::ffi::UBaseType_t;
use crate::freertos::ffi::QueueHandle_t;
use crate::freertos::queue::ffi::xQueueCreate;
use crate::osal::queue::ffi::vQueueDelete;

#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    use core::ffi::{c_void, c_uint, c_int};
    use crate::freertos::ffi::{BaseType_t, TickType_t, UBaseType_t};

    pub type QueueHandle_t = *mut c_void;

    unsafe extern "C" {
        pub fn xQueueCreate(uxQueueLength: UBaseType_t, uxItemSize: UBaseType_t) -> QueueHandle_t;
        pub fn vQueueDelete(xQueue: QueueHandle_t);
        pub fn xQueueSend(
            xQueue: QueueHandle_t,
            pvItemToQueue: *const c_void,
            xTicksToWait: TickType_t,
        ) -> BaseType_t;

        pub fn xQueueSendFromISR(
            xQueue: QueueHandle_t,
            pvItemToQueue: *const c_void,
            pxHigherPriorityTaskWoken: *mut BaseType_t,
        ) -> BaseType_t;

        pub fn xQueueReceive(
            xQueue: QueueHandle_t,
            pvBuffer: *mut c_void,
            xTicksToWait: TickType_t,
        ) -> BaseType_t;

        pub fn xQueueReceiveFromISR(
            xQueue: QueueHandle_t,
            pvBuffer: *mut c_void,
            pxHigherPriorityTaskWoken: *mut BaseType_t,
        ) -> BaseType_t;
    }
}

pub struct Queue {
    handle: QueueHandle_t,
}

impl Queue {

    fn new(queue_length: u32, item_size: u32) -> Self {
        unsafe {
            let handle = xQueueCreate(queue_length as UBaseType_t, item_size as UBaseType_t);
            Queue { handle }
        }
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            vQueueDelete(self.handle);
        }
    }
}