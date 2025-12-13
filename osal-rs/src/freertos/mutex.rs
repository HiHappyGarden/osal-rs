use crate::freertos::ffi::{self, MutexHandle, osal_rs_port_yield_from_isr, pdFALSE, pdTRUE, xQueueGiveMutexRecursive};
use crate::freertos::types::OsalRsBool;
use crate::freertos::types::MAX_DELAY;
use crate::utils::{Result, Error};
use crate::{vSemaphoreDelete, xSemaphoreCreateRecursiveMutex, xSemaphoreGiveFromISR, xSemaphoreGiveRecursive, xSemaphoreTake, xSemaphoreTakeFromISR};
use crate::traits::{ToTick, MutexFn};

pub struct  Mutex(MutexHandle);

impl MutexFn for Mutex {
    fn new() -> Result<Self> {
        let handle = xSemaphoreCreateRecursiveMutex!();
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Mutex(handle))
        }
    }
    
    fn lock(&self) -> OsalRsBool {
        let res = xSemaphoreTake!(self.0, MAX_DELAY.to_tick());
        if res == pdTRUE {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn lock_from_isr(&self) -> OsalRsBool {

        let mut higher_priority_task_woken = pdFALSE;
        let res = xSemaphoreTakeFromISR!(self.0, &mut higher_priority_task_woken);
        if res == pdTRUE {

            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);
            }

            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn unlock(&self) -> OsalRsBool {
        let res = xSemaphoreGiveRecursive!(self.0);
        if res == pdTRUE {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn unlock_from_isr(&self) -> OsalRsBool {
        let mut higher_priority_task_woken = pdFALSE;
        let res = xSemaphoreGiveFromISR!(self.0, &mut higher_priority_task_woken);
        if res == pdTRUE {

            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);
            }

            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn delete(&mut self) {
        vSemaphoreDelete!(self.0);
        self.0 = core::ptr::null();
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
    }
}