use core::ffi::c_void;
use core::fmt::Debug;
use core::ptr::null_mut;

use alloc::boxed::Box;

use crate::freertos::ffi::{QueueHandle_t, queueQUEUE_TYPE_BASE, queueSEND_TO_BACK, uxQueueSpacesAvailable, vQueueDelete, xQueueGenericCreate, xQueueGenericSend, xQueueGenericSendFromISR, xQueueReceive, xQueueReceiveFromISR};
use crate::tmo_to_ticks;
use crate::traits::QueueTrait;
use crate::types::{Error::Std, Result};

pub struct Queue {
    handle: QueueHandle_t,
    size: usize,
    message_size: usize
}

impl QueueTrait for Queue {
    fn new(size: usize, message_size: usize) -> Self
    where
        Self: Sized
    {
        Self {
            handle: unsafe { xQueueGenericCreate(size as u64, message_size as u64, queueQUEUE_TYPE_BASE) },
            size,
            message_size
        }
    }

    fn fetch<T>(&mut self, msg: &mut T, time: u64) -> Result<()>
    where
        T: Sized
    {
        if self.handle.is_null() {
            return Err(Std(-1, "Invalid queue handle"));
        }
        unsafe {
            let _ = xQueueReceive(self.handle, msg as *mut T as *mut c_void, tmo_to_ticks!(time));
        }

        Ok(())
    }

    fn fetch_from_isr<T>(&mut self, msg: &mut T, time: u64) -> Result<()>
    where
        T: Sized
    {
        if self.handle.is_null() {
            return Err(Std(-1, "Invalid queue handle"));
        }
        unsafe {
            let _ = xQueueReceiveFromISR(self.handle, msg as *mut T as *mut c_void, null_mut());
        }

        Ok(())
    }

    fn post<T>(&mut self, msg: &T, time: u64) -> Result<()>
    where
        T: Sized
    {
        if self.handle.is_null() {
            return Err(Std(-1, "Invalid queue handle"));
        }
        unsafe {
            let _ = xQueueGenericSend(self.handle, msg as *const T as *const c_void, tmo_to_ticks!(time), queueSEND_TO_BACK);
        }

        Ok(())
    }

    fn post_from_isr<T>(&mut self, msg: &T, time: u64) -> Result<()>
    where
        T: Sized
    {
        if self.handle.is_null() {
            return Err(Std(-1, "Invalid queue handle"));
        }
        unsafe {
            let _ = xQueueGenericSendFromISR(self.handle, msg as *const T as *const c_void, null_mut(), queueSEND_TO_BACK);
        }

        Ok(())
    }

    fn size(&self) -> usize {
        unsafe {
            self.size - (self.message_size * uxQueueSpacesAvailable(self.handle) as usize)
        }
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            vQueueDelete(self.handle);
        }
    }
}

impl Debug for Queue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Queue")
            .field("handle", &self.handle)
            .field("size", &self.size)
            .field("message_size", &self.message_size)
            .finish()
    }
}