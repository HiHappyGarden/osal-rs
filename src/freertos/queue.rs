
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