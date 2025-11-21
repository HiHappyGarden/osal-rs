#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/freertos_bindings.rs"));
}

pub struct Queue;

//
// use crate::freertos::queue::ffi::QueueHandle_t;
// use crate::osal::queue::ffi::vQueueDelete;
//
// pub struct Queue {
//     handle: QueueHandle_t,
// }
//
// impl Queue {
//
//     fn new(queue_length: u32, item_size: u32) -> Self {
//         unsafe {
//             let handle = xQueueCreate(queue_length as UBaseType_t, item_size as UBaseType_t);
//             Queue { handle }
//         }
//     }
// }
//
// impl Drop for Queue {
//     fn drop(&mut self) {
//         unsafe {
//             vQueueDelete(self.handle);
//         }
//     }
// }