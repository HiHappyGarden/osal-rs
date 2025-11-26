use core::fmt::Debug;
use core::ptr::null_mut;

use crate::us_to_ticks;
use crate::{EventTrait, freertos::ffi::EventGroupHandle_t, Result, Error::Std};
use crate::freertos::ffi::{pdFALSE, vEventGroupDelete, xEventGroupClearBits, xEventGroupClearBitsFromISR, xEventGroupCreate, xEventGroupGetBitsFromISR, xEventGroupSetBits, xEventGroupSetBitsFromISR, xEventGroupWaitBits};

pub struct Event {
    handle: EventGroupHandle_t
}

impl EventTrait for Event {
    fn new() -> Self
    where
        Self: Sized
    {
        Self {
            handle: unsafe { xEventGroupCreate() }
        }
    }

    fn wait(&mut self, mask: u32, value: &mut u32, time: u64) -> Result<()> {
        if self.handle.is_null() {
            return Err(Std(-1, "Invalid event group handle"))
        }

        let rc = unsafe {
            xEventGroupWaitBits(
                self.handle,
                u64::from(mask),
                pdFALSE,
                pdFALSE,
                us_to_ticks!(time),
            )
        };
        if rc == pdFALSE as u64 {
            Err(Std(-2, "Timeout waiting for event"))
        } else {
            *value = rc as u32 & mask;
            Ok(())
        }
    }

    fn wait_from_isr(&mut self, mask: u32, value: &mut u32, time: u64) -> crate::Result<()> {
        if self.handle.is_null() {
            return Err(Std(-1, "Invalid event group handle"))
        }
        self.wait(mask, value, time)
    }

    fn set(&mut self, value: u32) {
        if self.handle.is_null() {
            return
        }
        unsafe {
            xEventGroupSetBits(self.handle, value as u64);
        }
    }

    fn set_from_isr(&mut self, value: u32) {
        if self.handle.is_null() {
            return
        }
        unsafe {
            xEventGroupSetBitsFromISR(self.handle, value as u64, null_mut());
        }
    }

    fn get(&mut self) -> u32 {
        if self.handle.is_null() {
            return 0
        }
        unsafe {
             xEventGroupClearBits(self.handle, 0 ) as u32
        }
    }

    fn get_from_isr(&mut self) -> u32 {
        unsafe {
            xEventGroupGetBitsFromISR(self.handle) as u32
        }
    }

    fn clear(&mut self, value: u32) {
        unsafe {
            xEventGroupClearBits(self.handle, value as u64);
        }
    }

    fn clear_from_isr(&mut self, value: u32) {
        unsafe {
            xEventGroupClearBitsFromISR(self.handle, value as u64);
        }
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe {
            vEventGroupDelete(self.handle);
        }
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Event")
            .field("handle", &self.handle)
            .finish()
    }
}