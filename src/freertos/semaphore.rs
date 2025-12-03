use core::fmt::Debug;
use core::ptr::null_mut;

use crate::traits::SemaphoreTrait;
use crate::freertos::ffi::{SemaphoreHandle_t, pdTRUE, queueSEND_TO_BACK, semGIVE_BLOCK_TIME, vQueueDelete, xQueueCreateCountingSemaphore, xQueueGenericSend, xQueueGenericSendFromISR, xQueueGiveFromISR, xQueueReceiveFromISR, xQueueSemaphoreTake};
use crate::types::{Result, Error::Std};

pub struct Semaphore {
    handle: SemaphoreHandle_t
}

impl SemaphoreTrait for Semaphore {
    fn new(count: usize) -> Self 
    where 
        Self: Sized {
        Self { 
            handle: unsafe { xQueueCreateCountingSemaphore(u64::MAX, count as u64) }
        }
    }

    fn wait(&mut self, time: u64) -> Result<()> {
        if self.handle.is_null() {
            return Err(Std(-1, "Semaphore handle is null"))
        }
        unsafe {
            let res = xQueueSemaphoreTake(self.handle, time);
            if res == pdTRUE {
                Ok(())
            } else {
                Err(Std(res as i32, "Failed to take semaphore"))
            }
        }
    }

    fn wait_from_isr(&mut self, _time: u64) -> Result<()> {
        if self.handle.is_null() {
            return Err(Std(-1, "Semaphore handle is null"))
        }
        unsafe {
            let res = xQueueReceiveFromISR(self.handle, null_mut(), null_mut());
            if res == pdTRUE {
                Ok(())
            } else {
                Err(Std(res as i32, "Failed to take semaphore"))
            }
        }
    }

    fn signal(&mut self) {
        if self.handle.is_null() {
            return
        }
        unsafe {
            let _ = xQueueGenericSend(self.handle, null_mut(), semGIVE_BLOCK_TIME, queueSEND_TO_BACK);
        }
    }

    fn signal_from_isr(&mut self) {
        if self.handle.is_null() {
            return
        }
        unsafe {
            let _ = xQueueGiveFromISR(self.handle, null_mut());
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            vQueueDelete(self.handle);
        }
    }
}

impl Debug for Semaphore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Semaphore")
            .field("handle", &self.handle)
            .finish()
    }
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}