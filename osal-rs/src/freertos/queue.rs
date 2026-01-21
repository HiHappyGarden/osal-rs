/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Queue-based inter-thread communication for FreeRTOS.
//!
//! This module provides FIFO queue primitives for safe message passing between threads
//! and interrupt service routines. Supports both byte-based and typed queues.

use core::ffi::c_void;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ops::Deref;

use alloc::vec::Vec;

use super::ffi::{QueueHandle, pdFALSE, vQueueDelete, xQueueCreateCountingSemaphore, xQueueReceive, xQueueReceiveFromISR};
use super::types::{BaseType, UBaseType, TickType};
use super::system::System;
use crate::traits::{ToTick, QueueFn, SystemFn, QueueStreamedFn, BytesHasLen};
#[cfg(not(feature = "serde"))]
use crate::traits::{ToBytes, FromBytes};
    
#[cfg(not(feature = "serde"))]
pub trait StructSerde : ToBytes + BytesHasLen + FromBytes {}

#[cfg(feature = "serde")]
use osal_rs_serde::{Serialize, Deserialize, to_bytes};

#[cfg(feature = "serde")]
pub trait StructSerde : Serialize + BytesHasLen + Deserialize {}

use crate::utils::{Result, Error};
use crate::{xQueueSendToBack, xQueueSendToBackFromISR};


/// A FIFO queue for byte-based message passing.
///
/// Provides a thread-safe queue implementation for sending and receiving
/// raw byte slices between threads. Supports both blocking and ISR-safe operations.
///
/// # Examples
///
/// ## Basic queue usage
///
/// ```ignore
/// use osal_rs::os::{Queue, QueueFn};
/// use core::time::Duration;
/// 
/// // Create a queue with 10 slots, each 32 bytes
/// let queue = Queue::new(10, 32).unwrap();
/// 
/// // Send data
/// let data = [1u8, 2, 3, 4];
/// queue.post_with_to_tick(&data, Duration::from_millis(100)).unwrap();
/// 
/// // Receive data
/// let mut buffer = [0u8; 4];
/// queue.fetch_with_to_tick(&mut buffer, Duration::from_millis(100)).unwrap();
/// assert_eq!(buffer, [1, 2, 3, 4]);
/// ```
///
/// ## Producer-consumer pattern
///
/// ```ignore
/// use osal_rs::os::{Queue, QueueFn, Thread};
/// use alloc::sync::Arc;
/// use core::time::Duration;
/// 
/// let queue = Arc::new(Queue::new(5, 4).unwrap());
/// let queue_clone = queue.clone();
/// 
/// // Consumer thread
/// let consumer = Thread::new("consumer", 2048, 5, move || {
///     let mut buffer = [0u8; 4];
///     loop {
///         if queue_clone.fetch(&mut buffer, 1000).is_ok() {
///             println!("Received: {:?}", buffer);
///         }
///     }
/// }).unwrap();
/// 
/// consumer.start().unwrap();
/// 
/// // Producer
/// let data = [0xAA, 0xBB, 0xCC, 0xDD];
/// queue.post(&data, 1000).unwrap();
/// ```
pub struct Queue (QueueHandle);

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

impl Queue {
    /// Creates a new queue.
    ///
    /// # Parameters
    ///
    /// * `size` - Maximum number of messages the queue can hold
    /// * `message_size` - Size in bytes of each message
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created queue
    /// * `Err(Error)` - Creation failed (insufficient memory, etc.)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Queue, QueueFn};
    /// 
    /// // Queue for 5 messages of 16 bytes each
    /// let queue = Queue::new(5, 16).unwrap();
    /// ```
    pub fn new (size: UBaseType, message_size: UBaseType) -> Result<Self> {
        let handle = unsafe { xQueueCreateCountingSemaphore(size, message_size) };
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

    #[inline]
    pub fn fetch_with_to_tick(&self, buffer: &mut [u8], time: impl ToTick) -> Result<()> {
        self.fetch(buffer, time.to_ticks())
    }

    #[inline]
    pub fn post_with_to_tick(&self, item: &[u8], time: impl ToTick) -> Result<()> {
        self.post(item, time.to_ticks())
    }
}

impl QueueFn for Queue {

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

/// A type-safe FIFO queue for message passing.
///
/// Unlike [`Queue`], which works with raw byte slices, `QueueStreamed` provides
/// a type-safe interface for sending and receiving structured data. The type must
/// implement serialization traits.
///
/// # Type Parameters
///
/// * `T` - The message type. Must implement `ToBytes`, `BytesHasLen`, and `FromBytes`
///
/// # Examples
///
/// ## Basic typed queue usage
///
/// ```ignore
/// use osal_rs::os::{QueueStreamed, QueueStreamedFn};
/// use core::time::Duration;
/// 
/// #[derive(Debug, Clone, Copy)]
/// struct Message {
///     id: u32,
///     value: i16,
/// }
/// 
/// // Assuming Message implements the required traits
/// let queue: QueueStreamed<Message> = QueueStreamed::new(10, size_of::<Message>()).unwrap();
/// 
/// // Send a message
/// let msg = Message { id: 1, value: 42 };
/// queue.post_with_to_tick(&msg, Duration::from_millis(100)).unwrap();
/// 
/// // Receive a message
/// let mut received = Message { id: 0, value: 0 };
/// queue.fetch_with_to_tick(&mut received, Duration::from_millis(100)).unwrap();
/// assert_eq!(received.id, 1);
/// assert_eq!(received.value, 42);
/// ```
///
/// ## Command queue pattern
///
/// ```ignore
/// use osal_rs::os::{QueueStreamed, Thread};
/// use alloc::sync::Arc;
/// 
/// enum Command {
///     Start,
///     Stop,
///     SetValue(u32),
/// }
/// 
/// let cmd_queue = Arc::new(QueueStreamed::<Command>::new(10, 8).unwrap());
/// let queue_clone = cmd_queue.clone();
/// 
/// let handler = Thread::new("handler", 2048, 5, move || {
///     loop {
///         let mut cmd = Command::Stop;
///         if queue_clone.fetch(&mut cmd, 1000).is_ok() {
///             match cmd {
///                 Command::Start => { /* start operation */ },
///                 Command::Stop => { /* stop operation */ },
///                 Command::SetValue(val) => { /* set value */ },
///             }
///         }
///     }
/// }).unwrap();
/// ```
#[cfg(not(feature = "serde"))]
pub struct QueueStreamed<T: ToBytes + BytesHasLen + FromBytes> (Queue, PhantomData<T>);

#[cfg(not(feature = "serde"))]
unsafe impl<T: ToBytes + BytesHasLen + FromBytes> Send for QueueStreamed<T> {}

#[cfg(not(feature = "serde"))]
unsafe impl<T: ToBytes + BytesHasLen + FromBytes> Sync for QueueStreamed<T> {}

#[cfg(not(feature = "serde"))]
impl<T> QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    /// Creates a new type-safe queue.
    ///
    /// # Parameters
    ///
    /// * `size` - Maximum number of messages
    /// * `message_size` - Size of each message (typically `size_of::<T>()`)
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created queue
    /// * `Err(Error)` - Creation failed
    #[inline]
    pub fn new (size: UBaseType, message_size: UBaseType) -> Result<Self> {
        Ok(Self (Queue::new(size, message_size)?, PhantomData))
    }

    #[inline]
    fn fetch_with_to_tick(&self, buffer: &mut T, time: impl ToTick) -> Result<()> {
        self.fetch(buffer, time.to_ticks())
    }

    #[inline]
    fn post_with_to_tick(&self, item: &T, time: impl ToTick) -> Result<()> {
        self.post(item, time.to_ticks())
    }
}

#[cfg(not(feature = "serde"))]
impl<T> QueueStreamedFn<T> for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {

    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());         

        if let Ok(()) = self.0.fetch(&mut buf_bytes, time) {
            *buffer = T::from_bytes(&buf_bytes)?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());      

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

#[cfg(not(feature = "serde"))]
impl<T> Deref for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    type Target = QueueHandle;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }   
}

#[cfg(not(feature = "serde"))]
impl<T> Debug for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueueStreamed")
            .field("handle", &self.0.0)
            .finish()
    }
}

#[cfg(not(feature = "serde"))]
impl<T> Display for QueueStreamed<T> 
where 
    T: ToBytes + BytesHasLen + FromBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "QueueStreamed {{ handle: {:?} }}", self.0.0)
    }
}


#[cfg(feature = "serde")]
pub struct QueueStreamed<T: StructSerde> (Queue, PhantomData<T>);

#[cfg(feature = "serde")]
unsafe impl<T: StructSerde> Send for QueueStreamed<T> {}

#[cfg(feature = "serde")]
unsafe impl<T: StructSerde> Sync for QueueStreamed<T> {}

#[cfg(feature = "serde")]
impl<T> QueueStreamed<T> 
where 
    T: StructSerde {
    /// Creates a new type-safe queue.
    ///
    /// # Parameters
    ///
    /// * `size` - Maximum number of messages
    /// * `message_size` - Size of each message (typically `size_of::<T>()`)
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created queue
    /// * `Err(Error)` - Creation failed
    #[inline]
    pub fn new (size: UBaseType, message_size: UBaseType) -> Result<Self> {
        Ok(Self (Queue::new(size, message_size)?, PhantomData))
    }

    #[inline]
    fn fetch_with_to_tick(&self, buffer: &mut T, time: impl ToTick) -> Result<()> {
        self.fetch(buffer, time.to_ticks())
    }

    #[inline]
    fn post_with_to_tick(&self, item: &T, time: impl ToTick) -> Result<()> {
        self.post(item, time.to_ticks())
    }
}

#[cfg(feature = "serde")]
impl<T> QueueStreamedFn<T> for QueueStreamed<T> 
where 
    T: StructSerde {

    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());     

        if let Ok(()) = self.0.fetch(&mut buf_bytes, time) {
            
            to_bytes(buffer, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());       

        if let Ok(()) = self.0.fetch_from_isr(&mut buf_bytes) {
            to_bytes(buffer, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    fn post(&self, item: &T, time: TickType) -> Result<()> {


        let mut buf_bytes = Vec::with_capacity(item.len()); 

        to_bytes(item, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

        self.0.post(&buf_bytes, time)
    }

    fn post_from_isr(&self, item: &T) -> Result<()> {

        let mut buf_bytes = Vec::with_capacity(item.len()); 

        to_bytes(item, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

        self.0.post_from_isr(&buf_bytes)
    }

    #[inline]
    fn delete(&mut self) {
        self.0.delete()
    }
}

#[cfg(feature = "serde")]
impl<T> Deref for QueueStreamed<T> 
where 
    T: StructSerde {
    type Target = QueueHandle;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }   
}

#[cfg(feature = "serde")]
impl<T> Debug for QueueStreamed<T> 
where 
    T: StructSerde {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueueStreamed")
            .field("handle", &self.0.0)
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<T> Display for QueueStreamed<T> 
where 
    T: StructSerde {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "QueueStreamed {{ handle: {:?} }}", self.0.0)
    }
}