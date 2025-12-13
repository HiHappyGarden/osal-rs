use core::ops::Deref;
use core::ptr::null_mut;

use crate::freertos::ffi::{EventGroupHandle, osal_rs_port_yield_from_isr, pdFAIL, pdFALSE, vEventGroupDelete, xEventGroupClearBits, xEventGroupClearBitsFromISR, xEventGroupCreate, xEventGroupGetBitsFromISR, xEventGroupSetBits, xEventGroupSetBitsFromISR};
use crate::traits::{ToTick, EventGroupFn};
use crate::freertos::types::{BaseType, EventBits};
use crate::utils::{Result, Error};
use crate::xEventGroupGetBits;

pub struct EventGroup (EventGroupHandle);

unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}

impl EventGroupFn for EventGroup {
    fn new() -> Result<Self> {
        let handle = unsafe { xEventGroupCreate() };
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

    fn set(&mut self, bits: EventBits) -> EventBits {
        unsafe { xEventGroupSetBits(self.0, bits) }
    }

    fn set_from_isr(&mut self, bits: EventBits) -> Result<()> {

        let mut higher_priority_task_woken: BaseType = pdFALSE;

        let ret = unsafe { xEventGroupSetBitsFromISR(self.0, bits, &mut higher_priority_task_woken) };
        if ret != pdFAIL {
            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);   
            }
            
            Ok(())
        } else {
            Err(Error::QueueFull)
        }
    }

    fn get(&self) -> EventBits {
        xEventGroupGetBits!(self.0) 
    }

    fn get_from_isr(&self) -> EventBits{
        unsafe { xEventGroupGetBitsFromISR(self.0) }
    }


    fn clear(&mut self, bits: EventBits) -> EventBits {
        unsafe { xEventGroupClearBits(self.0, bits) }
    }

    fn clear_from_isr(&mut self, bits: EventBits) -> Result<()> {
        let ret = unsafe { xEventGroupClearBitsFromISR(self.0, bits) };
        if ret != pdFAIL {
            Ok(())
        } else {
            Err(Error::QueueFull)
        }
    }

    fn wait(&mut self, mask: EventBits, timeout_ticks: impl ToTick) -> EventBits {
        unsafe {
            crate::freertos::ffi::xEventGroupWaitBits(
                self.0,
                mask,
                pdFALSE, 
                pdFALSE, 
                timeout_ticks.to_tick(),
            )
        }
    }

    fn delete(&mut self) {
        unsafe {
            vEventGroupDelete(self.0);
            self.0 = null_mut();
        }
    }
}

impl Drop for EventGroup {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
    }
}

impl Deref for EventGroup {
    type Target = EventGroupHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}