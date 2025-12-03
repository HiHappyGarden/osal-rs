use core::fmt::Debug;
use core::ptr::null_mut;

use crate::traits::MutexTrait;
use crate::{Result, WAIT_FOREVER, us_to_ticks};
use crate::freertos::ffi::{QueueHandle_t, queueQUEUE_TYPE_MUTEX, vQueueDelete, xQueueCreateMutex, xQueueGiveFromISR, xQueueGiveMutexRecursive, xQueueReceiveFromISR, xQueueTakeMutexRecursive};



pub struct Mutex {
    handler: QueueHandle_t
}
 
impl MutexTrait for Mutex {
    fn new() -> Result<Self>
    where
        Self: Sized
    {
        unsafe {
            Ok(Self {
                handler: xQueueCreateMutex(queueQUEUE_TYPE_MUTEX)
            })
        }
    }

    fn lock(&mut self) {
        if self.handler.is_null() {
            return
        }
        #[allow(unused_unsafe)]
        unsafe {
            let _ = xQueueTakeMutexRecursive( self.handler, us_to_ticks!(WAIT_FOREVER)); 
        }
    }

    fn lock_from_isr(&mut self) {
        if self.handler.is_null() {
            return
        }
        unsafe {
            let _ = xQueueReceiveFromISR( self.handler, null_mut(), null_mut()); 
        }
    }

    fn unlock(&mut self) {
        if self.handler.is_null() {
            return
        }
        unsafe {
            let _ = xQueueGiveMutexRecursive(self.handler); 
        }
    }

    fn unlock_from_isr(&mut self) {
        if self.handler.is_null() {
            return
        }
        unsafe {
            let _ = xQueueGiveFromISR( self.handler, null_mut()); 
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        if self.handler.is_null() {
            return
        }
        unsafe {
            vQueueDelete(self.handler);
        }
    }
}

impl Debug for Mutex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mutex")
         .field("handler", &self.handler)
         .finish()
    }
}


unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}