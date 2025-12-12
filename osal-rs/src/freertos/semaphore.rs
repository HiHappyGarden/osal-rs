use crate::freertos::ffi::{SemaphoreHandle, osal_rs_port_yield_from_isr, pdFAIL, pdFALSE};
use crate::traits::ToTick;
use crate::freertos::types::{BaseType, OsalRsBool, UBaseType};
use crate::traits::SemaphoreFn;
use crate::utils::{Error, Result};
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

    fn wait(&mut self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        if xSemaphoreTake!(self.0, ticks_to_wait.to_tick()) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn wait_from_isr(&mut self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreTakeFromISR!(self.0, &mut higher_priority_task_woken) != pdFAIL {
            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);   
            }
            OsalRsBool::True
        } else {

            OsalRsBool::False
        }
    }
    
    fn signal(&mut self) -> OsalRsBool {
        if xSemaphoreGive!(self.0) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }
    
    fn signal_from_isr(&mut self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreGiveFromISR!(self.0, &mut higher_priority_task_woken) != pdFAIL {
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
    }


}