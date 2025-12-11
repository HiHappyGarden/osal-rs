use crate::freertos::ffi::{EventGroupHandle, pdFALSE, pdPASS, xEventGroupClearBits, xEventGroupClearBitsFromISR, xEventGroupCreate, xEventGroupGetBitsFromISR, xEventGroupSetBits, xEventGroupSetBitsFromISR};
use crate::traits::{ToTick, EventGroupFn};
use crate::freertos::types::{BaseType, EventBits};
use crate::utils::{Result, Error};
use crate::xEventGroupGetBits;

pub struct EventGroup {
    handle: EventGroupHandle
}

unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}

impl EventGroupFn for EventGroup {
    fn new() -> Self {;
        Self { handle: unsafe { xEventGroupCreate() } }
    }

    fn set(&mut self, bits: EventBits) -> EventBits {
        unsafe { xEventGroupSetBits(self.handle, bits) }
    }

    fn set_from_isr(&mut self, bits: EventBits, higher_priority_task_woken: &mut BaseType) -> Result<()> {
        let ret = unsafe { xEventGroupSetBitsFromISR(self.handle, bits, higher_priority_task_woken) }
        if ret != pdPASS {
            Ok(())
        } else {
            Err(Error::QueueFull)
        }
    }

    fn get(&self) -> EventBits {
        xEventGroupGetBits!(self.handle) 
    }

    fn get_from_isr(&self) -> EventBits{
        unsafe { xEventGroupGetBitsFromISR(self.handle) }
    }


    fn clear(&mut self, bits: EventBits) -> EventBits {
        unsafe { xEventGroupClearBits(self.handle, bits) }
    }
    fn clear_from_isr(&mut self, bits: EventBits) -> Result<()> {
        let ret = unsafe { xEventGroupClearBitsFromISR(self.handle, bits) };
        if ret != 0 {
            Ok(())
        } else {
            Err(Error::QueueFull)
        }
    }

    fn wait(&mut self, mask: EventBits, timeout_ticks: impl ToTick) -> EventBits {
        unsafe {
            crate::freertos::ffi::xEventGroupWaitBits(
                self.handle,
                mask,
                pdFALSE, 
                pdFALSE, 
                timeout_ticks.to_tick(),
            )
        }
    }
}
