use crate::freertos::ffi::EventGroupHandle;

pub struct EventGroup {
    handle: EventGroupHandle
}

unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}


