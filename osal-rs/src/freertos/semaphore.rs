use crate::freertos::ffi::{SemaphoreHandle, osal_rs_port_yield_from_isr, pdFAIL, pdFALSE, pdTRUE};
use crate::os::ToTick;
use crate::os::types::{BaseType, OsalRsBool, UBaseType};
use crate::traits::SemaphoreFn;
use crate::utils::{Error, Result};
use crate::{vSemaphoreDelete, xSemaphoreCreateCounting, xSemaphoreGive, xSemaphoreGiveFromISR, xSemaphoreTake, xSemaphoreTakeFromISR};

pub struct Semaphore {
    handle: SemaphoreHandle,
}

impl SemaphoreFn for Semaphore {
    fn new(max_count: UBaseType, initial_count: UBaseType) -> Result<Self> {
        let handle = xSemaphoreCreateCounting!(max_count, initial_count);
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self { handle })
        }
    }

    fn new_with_count(initial_count: UBaseType) -> Result<Self> {
        let handle = xSemaphoreCreateCounting!(UBaseType::MAX, initial_count);
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self { handle })
        }
    }

    fn wait(&mut self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        if xSemaphoreTake!(self.handle, ticks_to_wait.to_tick()) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn wait_from_isr(&mut self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreTakeFromISR!(self.handle, &mut higher_priority_task_woken) != pdFAIL {
            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);   
            }
            OsalRsBool::True
        } else {

            OsalRsBool::False
        }
    }
    
    fn signal(&mut self) -> OsalRsBool {
        if xSemaphoreGive!(self.handle) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }
    
    fn signal_from_isr(&mut self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreGiveFromISR!(self.handle, &mut higher_priority_task_woken) != pdFAIL {
            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);   
            }
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }
    
    fn delete(&mut self) {
        vSemaphoreDelete!(self.handle);
    }


}