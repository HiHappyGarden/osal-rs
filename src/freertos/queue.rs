use crate::freertos::ffi::{QueueHandle_t, queueQUEUE_TYPE_BASE, vQueueDelete, xQueueGenericCreate, xQueueReceive};
use crate::traits::QueueTrait;
use crate::types::{Error::Std, Result};

pub struct Queue {
    handle: QueueHandle_t,
}

impl QueueTrait for Queue {
    fn new(size: usize, message_size: usize) -> Self
    where
        Self: Sized
    {
        Self{
            handle: unsafe { xQueueGenericCreate(size as u64, message_size as u64, queueQUEUE_TYPE_BASE) }
        }
    }

    fn fetch<T>(&mut self, msg: &mut T, time: u64) -> Result<()>
    where
        T: Sized
    {
        if self.handle.is_null() {
            return Err(Std(-1, "Invalid queue handle"));
        }
        //xQueueReceive(self.handle, pvBuffer, xTicksToWait);
        todo!()
    }

    fn fetch_from_isr<T>(&mut self, msg: &mut T, time: u64) -> Result<()>
    where
        T: Sized
    {
        todo!()
    }

    fn post<T>(&mut self, msg: T, time: u64) -> Result<()>
    where
        T: Sized
    {
        todo!()
    }

    fn post_from_isr<T>(&mut self, msg: T, time: u64) -> Result<()>
    where
        T: Sized
    {
        todo!()
    }

    fn size(&self) -> usize {
        todo!()
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            vQueueDelete(self.handle);
        }
    }
}