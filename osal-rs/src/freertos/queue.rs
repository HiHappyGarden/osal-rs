use core::ffi::c_void;
use core::ops::Deref;

use super::ffi::{QueueHandle, pdFALSE, vQueueDelete, xQueueCreateCountingSemaphore, xQueueReceive, xQueueReceiveFromISR};
use super::types::{BaseType, UBaseType};
use super::system::System;
use crate::traits::{ToTick, QueueFn, SystemFn};
use crate::utils::{Result, Error};
use crate::{xQueueSendToBack, xQueueSendToBackFromISR};


pub struct Queue (QueueHandle);

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

impl QueueFn for Queue {
    fn new (size: UBaseType, message_size: super::types::UBaseType) -> Result<Self> {
        let handle = unsafe { xQueueCreateCountingSemaphore(size, message_size) };
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

    fn fetch(&self, buffer: &mut [u8], time: impl ToTick) -> Result<()> {
        let ret = unsafe {
            xQueueReceive(
                self.0,
                buffer.as_mut_ptr() as *mut c_void,
                time.to_tick(),
            )
        };
        if ret == 0 {
            Err(Error::Timeout)
        } else {
            Ok(())
        }
    }

    fn fetch_from_isr(&self, buffer: &mut [u8]) -> Result<()> {

        let mut task_woken_by_receive: BaseType = pdFALSE;

        let ret = unsafe {
            xQueueReceiveFromISR(
                self.0,
                buffer.as_mut_ptr() as *mut c_void,
                &mut task_woken_by_receive
            )
        };
        if ret == 0 {
            Err(Error::Timeout)
        } else {

            System::yield_from_isr(task_woken_by_receive);
            
            Ok(())
        }
    }

    fn post(&self, item: &[u8], time: impl ToTick) -> Result<()> {
        let ret = xQueueSendToBack!(
                            self.0,
                            item.as_ptr() as *const c_void,
                            time.to_tick()
                        );
        
        if ret == 0 {
            Err(Error::Timeout)
        } else {
            Ok(())
        }
    }

    fn post_from_isr(&self, item: &[u8]) -> Result<()> {

        let mut task_woken_by_receive: BaseType = pdFALSE;

        let ret = xQueueSendToBackFromISR!(
                            self.0,
                            item.as_ptr() as *const c_void,
                            &mut task_woken_by_receive
                        );
        
        if ret == 0 {
            Err(Error::Timeout)
        } else {
            System::yield_from_isr(task_woken_by_receive);

            Ok(())
        }
    }

    fn delete(&mut self) {
        unsafe {
            vQueueDelete(self.0);
            self.0 = core::ptr::null_mut();
        }
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
    }
}

impl Deref for Queue {
    type Target = QueueHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// pub struct QueueTyped (QueueHandle);

// impl QueueTypedFn<QueueTyped> for QueueTyped {
//     fn typed_new (size: UBaseType, message_size: UBaseType) -> Result<Self>{
//         todo!()
//     }

//     fn typed_fetch(&self, buffer: &mut QueueTyped, time: impl ToTick) -> Result<()> {
//         todo!()
//     }

//     fn typed_fetch_from_isr(&self, buffer: &mut QueueTyped) -> Result<()> {
//         todo!()
//     }

//     fn typed_post(&self, item: &QueueTyped, time: impl ToTick) -> Result<()> {
//         todo!()
//     }

//     fn typed_post_from_isr(&self, item: &QueueTyped) -> Result<()> {
//         todo!()
//     }

//     fn typed_delete(&mut self) {
//         todo!()
//     }
// }