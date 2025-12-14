use core::ops::Deref;
use core::ptr::null_mut;

use super::ffi::{SemaphoreHandle, pdFAIL, pdFALSE};
use super::system::System;
use super::types::{BaseType, UBaseType};
use crate::traits::{SemaphoreFn, SystemFn, ToTick};
use crate::utils::{Error, Result, OsalRsBool};
use crate::{vSemaphoreDelete, xSemaphoreCreateCounting, xSemaphoreGive, xSemaphoreGiveFromISR, xSemaphoreTake, xSemaphoreTakeFromISR};

pub struct Semaphore (SemaphoreHandle);

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}


impl SemaphoreFn for Semaphore {
    fn new(max_count: UBaseType, initial_count: UBaseType) -> Result<Self> {
        let handle = xSemaphoreCreateCounting!(max_count, initial_count);
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

    fn new_with_count(initial_count: UBaseType) -> Result<Self> {
        let handle = xSemaphoreCreateCounting!(UBaseType::MAX, initial_count);
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

    fn wait(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        if xSemaphoreTake!(self.0, ticks_to_wait.to_tick()) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn wait_from_isr(&self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreTakeFromISR!(self.0, &mut higher_priority_task_woken) != pdFAIL {

            System::yield_from_isr(higher_priority_task_woken);

            OsalRsBool::True
        } else {

            OsalRsBool::False
        }
    }
    
    fn signal(&self) -> OsalRsBool {
        if xSemaphoreGive!(self.0) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }
    
    fn signal_from_isr(&self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreGiveFromISR!(self.0, &mut higher_priority_task_woken) != pdFAIL {
            
            System::yield_from_isr(higher_priority_task_woken);

            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }
    
    fn delete(&mut self) {
        vSemaphoreDelete!(self.0);
        self.0 = null_mut();
    }


}


impl Drop for Semaphore {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
    }
}

impl Deref for Semaphore {
    type Target = SemaphoreHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
