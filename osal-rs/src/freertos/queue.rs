use core::ffi::c_void;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ops::Deref;

use alloc::vec;

use super::ffi::{QueueHandle, pdFALSE, vQueueDelete, xQueueCreateCountingSemaphore, xQueueReceive, xQueueReceiveFromISR};
use super::types::{BaseType, UBaseType, TickType};
use super::system::System;
use crate::traits::{ToTick, QueueFn, SystemFn, QueueStreamedFn, ToBytes, BytesHasLen, FromBytes};
use crate::utils::{Result, Error};
use crate::{xQueueSendToBack, xQueueSendToBackFromISR};


pub struct Queue (QueueHandle);

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

impl Queue {
    #[inline]
    fn fetch_with_to_tick(&self, buffer: &mut [u8], time: impl ToTick) -> Result<()> {
        self.fetch(buffer, time.to_ticks())
    }

    #[inline]
    fn post_with_to_tick(&self, item: &[u8], time: impl ToTick) -> Result<()> {
        self.post(item, time.to_ticks())
    }
}

impl QueueFn for Queue {
    fn new (size: UBaseType, message_size: super::types::UBaseType) -> Result<Self> {
        let handle = unsafe { xQueueCreateCountingSemaphore(size, message_size) };
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

    fn fetch(&self, buffer: &mut [u8], time: TickType) -> Result<()> {
        let ret = unsafe {
            xQueueReceive(
                self.0,
                buffer.as_mut_ptr() as *mut c_void,
                time,
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

    fn post(&self, item: &[u8], time: TickType) -> Result<()> {
        let ret = xQueueSendToBack!(
                            self.0,
                            item.as_ptr() as *const c_void,
                            time
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

impl Debug for Queue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Queue")
            .field("handle", &self.0)
            .finish()
    }
}

impl Display for Queue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Queue {{ handle: {:?} }}", self.0)
    }
}

pub struct QueueStreamed<T: ToBytes + BytesHasLen + FromBytes> (Queue, PhantomData<T>);

unsafe impl<T: ToBytes + BytesHasLen + FromBytes> Send for QueueStreamed<T> {}
unsafe impl<T: ToBytes + BytesHasLen + FromBytes> Sync for QueueStreamed<T> {}

impl<T> QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    #[inline]
    fn fetch_with_to_tick(&self, buffer: &mut T, time: impl ToTick) -> Result<()> {
        self.fetch(buffer, time.to_ticks())
    }

    #[inline]
    fn post_with_to_tick(&self, item: &T, time: impl ToTick) -> Result<()> {
        self.post(item, time.to_ticks())
    }
}

impl<T> QueueStreamedFn<T> for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {

    #[inline]
    fn new (size: UBaseType, message_size: UBaseType) -> Result<Self> {
        Ok(Self (Queue::new(size, message_size)?, PhantomData))
    }

    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()> {
        let mut buf_bytes = vec![0u8; buffer.len()];        

        if let Ok(()) = self.0.fetch(&mut buf_bytes, time) {
            *buffer = T::from_bytes(&buf_bytes)?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()> {
        let mut buf_bytes = vec![0u8; buffer.len()];        

        if let Ok(()) = self.0.fetch_from_isr(&mut buf_bytes) {
            *buffer = T::from_bytes(&buf_bytes)?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    #[inline]
    fn post(&self, item: &T, time: TickType) -> Result<()> {
        self.0.post(&item.to_bytes(), time)
    }

    #[inline]
    fn post_from_isr(&self, item: &T) -> Result<()> {
        self.0.post_from_isr(&item.to_bytes())
    }

    #[inline]
    fn delete(&mut self) {
        self.0.delete()
    }
}

impl<T> Deref for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    type Target = QueueHandle;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }   
}

impl<T> Debug for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueueStreamed")
            .field("handle", &self.0.0)
            .finish()
    }
}

impl<T> Display for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "QueueStreamed {{ handle: {:?} }}", self.0.0)
    }
}