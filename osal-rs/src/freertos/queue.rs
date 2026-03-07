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
use crate::traits::{Serialize, Deserialize};

#[cfg(feature = "serde")]
use osal_rs_serde::{Serialize, Deserialize, to_bytes};

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

    /// Receives data from the queue with a convertible timeout.
    /// 
    /// This is a convenience method that accepts any type implementing `ToTick`
    /// (like `Duration`) and converts it to ticks before calling `fetch()`.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable slice to receive data into
    /// * `time` - Timeout value (e.g., `Duration::from_millis(100)`)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Data successfully received
    /// * `Err(Error::Timeout)` - No data available within timeout
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Queue, QueueFn};
    /// use core::time::Duration;
    /// 
    /// let queue = Queue::new(5, 16).unwrap();
    /// let mut buffer = [0u8; 16];
    /// queue.fetch_with_to_tick(&mut buffer, Duration::from_millis(100))?;
    /// ```
    #[inline]
    pub fn fetch_with_to_tick(&self, buffer: &mut [u8], time: impl ToTick) -> Result<()> {
        self.fetch(buffer, time.to_ticks())
    }

    /// Sends data to the queue with a convertible timeout.
    /// 
    /// This is a convenience method that accepts any type implementing `ToTick`
    /// (like `Duration`) and converts it to ticks before calling `post()`.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Slice of data to send
    /// * `time` - Timeout value (e.g., `Duration::from_millis(100)`)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Data successfully sent
    /// * `Err(Error::Timeout)` - Queue full, could not send within timeout
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Queue, QueueFn};
    /// use core::time::Duration;
    /// 
    /// let queue = Queue::new(5, 16).unwrap();
    /// let data = [1u8, 2, 3, 4];
    /// queue.post_with_to_tick(&data, Duration::from_millis(100))?;
    /// ```
    #[inline]
    pub fn post_with_to_tick(&self, item: &[u8], time: impl ToTick) -> Result<()> {
        self.post(item, time.to_ticks())
    }
}

impl QueueFn for Queue {

    /// Receives data from the queue, blocking until data is available or timeout.
    /// 
    /// This function blocks the calling thread until data is available or the
    /// specified timeout expires.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable byte slice to receive data into
    /// * `time` - Timeout in system ticks (0 = no wait, MAX = wait forever)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Data successfully received into buffer
    /// * `Err(Error::Timeout)` - No data available within timeout period
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Queue, QueueFn};
    /// 
    /// let queue = Queue::new(5, 16).unwrap();
    /// let mut buffer = [0u8; 16];
    /// 
    /// // Wait up to 1000 ticks for data
    /// match queue.fetch(&mut buffer, 1000) {
    ///     Ok(()) => println!("Received data: {:?}", buffer),
    ///     Err(_) => println!("Timeout"),
    /// }
    /// ```
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

    /// Receives data from the queue in an interrupt service routine (ISR).
    /// 
    /// This is the ISR-safe version of `fetch()`. It does not block and will
    /// trigger a context switch if a higher priority task is woken.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable byte slice to receive data into
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Data successfully received
    /// * `Err(Error::Timeout)` - Queue is empty
    /// 
    /// # Safety
    /// 
    /// Must only be called from ISR context.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// // In interrupt handler
    /// use osal_rs::os::{Queue, QueueFn};
    /// 
    /// fn irq_handler(queue: &Queue) {
    ///     let mut buffer = [0u8; 16];
    ///     if queue.fetch_from_isr(&mut buffer).is_ok() {
    ///         // Process received data
    ///     }
    /// }
    /// ```
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

    /// Sends data to the back of the queue, blocking until space is available.
    /// 
    /// This function blocks the calling thread until space becomes available
    /// or the timeout expires.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Byte slice to send
    /// * `time` - Timeout in system ticks (0 = no wait, MAX = wait forever)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Data successfully sent
    /// * `Err(Error::Timeout)` - Queue full, could not send within timeout
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Queue, QueueFn};
    /// 
    /// let queue = Queue::new(5, 16).unwrap();
    /// let data = [0xAA, 0xBB, 0xCC, 0xDD];
    /// 
    /// // Wait up to 1000 ticks to send
    /// queue.post(&data, 1000)?;
    /// ```
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

    /// Sends data to the queue from an interrupt service routine (ISR).
    /// 
    /// This is the ISR-safe version of `post()`. It does not block and will
    /// trigger a context switch if a higher priority task is woken.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Byte slice to send
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Data successfully sent
    /// * `Err(Error::Timeout)` - Queue is full
    /// 
    /// # Safety
    /// 
    /// Must only be called from ISR context.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// // In interrupt handler
    /// use osal_rs::os::{Queue, QueueFn};
    /// 
    /// fn irq_handler(queue: &Queue) {
    ///     let data = [0x01, 0x02, 0x03];
    ///     queue.post_from_isr(&data).ok();
    /// }
    /// ```
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

    /// Deletes the queue and frees its resources.
    /// 
    /// This function destroys the queue and releases any memory allocated for it.
    /// After calling this, the queue should not be used. The handle is set to null.
    /// 
    /// # Safety
    /// 
    /// Ensure no threads are waiting on this queue before deleting it.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Queue, QueueFn};
    /// 
    /// let mut queue = Queue::new(5, 16).unwrap();
    /// // Use the queue...
    /// queue.delete();
    /// ```
    fn delete(&mut self) {
        unsafe {
            vQueueDelete(self.0);
            self.0 = core::ptr::null_mut();
        }
    }
}

/// Automatically deletes the queue when it goes out of scope.
/// 
/// This ensures proper cleanup of FreeRTOS resources.
impl Drop for Queue {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
    }
}

/// Allows dereferencing to the underlying FreeRTOS queue handle.
impl Deref for Queue {
    type Target = QueueHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Formats the queue for debugging purposes.
impl Debug for Queue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Queue")
            .field("handle", &self.0)
            .finish()
    }
}

/// Formats the queue for display purposes.
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
pub struct QueueStreamed<T: StructSerde> (Queue, PhantomData<T>);

unsafe impl<T: StructSerde> Send for QueueStreamed<T> {}
unsafe impl<T: StructSerde> Sync for QueueStreamed<T> {}

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

    /// Receives a typed message with a convertible timeout.
    /// 
    /// This is a convenience method that accepts any type implementing `ToTick`.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable reference to receive the message into
    /// * `time` - Timeout value (e.g., `Duration::from_millis(100)`)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully received and deserialized
    /// * `Err(Error)` - Timeout or deserialization error
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::QueueStreamed;
    /// use core::time::Duration;
    /// 
    /// let queue: QueueStreamed<MyMessage> = QueueStreamed::new(5, size_of::<MyMessage>()).unwrap();
    /// let mut msg = MyMessage::default();
    /// queue.fetch_with_to_tick(&mut msg, Duration::from_millis(100))?;
    /// ```
    #[inline]
    fn fetch_with_to_tick(&self, buffer: &mut T, time: impl ToTick) -> Result<()> {
        self.fetch(buffer, time.to_ticks())
    }

    /// Sends a typed message with a convertible timeout.
    /// 
    /// This is a convenience method that accepts any type implementing `ToTick`.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Reference to the message to send
    /// * `time` - Timeout value (e.g., `Duration::from_millis(100)`)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully serialized and sent
    /// * `Err(Error)` - Timeout or serialization error
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::QueueStreamed;
    /// use core::time::Duration;
    /// 
    /// let queue: QueueStreamed<MyMessage> = QueueStreamed::new(5, size_of::<MyMessage>()).unwrap();
    /// let msg = MyMessage { id: 1, value: 42 };
    /// queue.post_with_to_tick(&msg, Duration::from_millis(100))?;
    /// ```
    #[inline]
    fn post_with_to_tick(&self, item: &T, time: impl ToTick) -> Result<()> {
        self.post(item, time.to_ticks())
    }
}

#[cfg(not(feature = "serde"))]
impl<T> QueueStreamedFn<T> for QueueStreamed<T> 
where 
    T: StructSerde {

    /// Receives a typed message from the queue (without serde feature).
    /// 
    /// Deserializes the message from bytes using the custom serialization traits.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable reference to receive the deserialized message
    /// * `time` - Timeout in system ticks
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully received and deserialized
    /// * `Err(Error::Timeout)` - Queue empty or timeout
    /// * `Err(Error)` - Deserialization error
    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());         

        if let Ok(()) = self.0.fetch(&mut buf_bytes, time) {
            *buffer = T::from_bytes(&buf_bytes)?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    /// Receives a typed message from ISR context (without serde feature).
    /// 
    /// ISR-safe version that does not block. Deserializes the message from bytes.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable reference to receive the deserialized message
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully received and deserialized
    /// * `Err(Error::Timeout)` - Queue is empty
    /// * `Err(Error)` - Deserialization error
    /// 
    /// # Safety
    /// 
    /// Must only be called from ISR context.
    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());      

        if let Ok(()) = self.0.fetch_from_isr(&mut buf_bytes) {
            *buffer = T::from_bytes(&buf_bytes)?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    /// Sends a typed message to the queue (without serde feature).
    /// 
    /// Serializes the message to bytes using the custom serialization traits.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Reference to the message to send
    /// * `time` - Timeout in system ticks
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully serialized and sent
    /// * `Err(Error::Timeout)` - Queue full
    /// * `Err(Error)` - Serialization error
    #[inline]
    fn post(&self, item: &T, time: TickType) -> Result<()> {
        self.0.post(&item.to_bytes(), time)
    }

    /// Sends a typed message from ISR context (without serde feature).
    /// 
    /// ISR-safe version that does not block. Serializes the message to bytes.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Reference to the message to send
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully serialized and sent
    /// * `Err(Error::Timeout)` - Queue is full
    /// * `Err(Error)` - Serialization error
    /// 
    /// # Safety
    /// 
    /// Must only be called from ISR context.
    #[inline]
    fn post_from_isr(&self, item: &T) -> Result<()> {
        self.0.post_from_isr(&item.to_bytes())
    }

    /// Deletes the typed queue.
    /// 
    /// Delegates to the underlying byte queue's delete method.
    #[inline]
    fn delete(&mut self) {
        self.0.delete()
    }
}

#[cfg(feature = "serde")]
impl<T> QueueStreamedFn<T> for QueueStreamed<T> 
where 
    T: StructSerde {

    /// Receives a typed message from the queue (with serde feature).
    /// 
    /// Deserializes the message from bytes using the serde framework.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable reference to receive the deserialized message
    /// * `time` - Timeout in system ticks
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully received and deserialized
    /// * `Err(Error::Timeout)` - Queue empty or timeout
    /// * `Err(Error::Unhandled)` - Deserialization error
    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());     

        if let Ok(()) = self.0.fetch(&mut buf_bytes, time) {
            
            to_bytes(buffer, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    /// Receives a typed message from ISR context (with serde feature).
    /// 
    /// ISR-safe version that does not block. Deserializes using serde.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Mutable reference to receive the deserialized message
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully received and deserialized
    /// * `Err(Error::Timeout)` - Queue is empty
    /// * `Err(Error::Unhandled)` - Deserialization error
    /// 
    /// # Safety
    /// 
    /// Must only be called from ISR context.
    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()> {
        let mut buf_bytes = Vec::with_capacity(buffer.len());       

        if let Ok(()) = self.0.fetch_from_isr(&mut buf_bytes) {
            to_bytes(buffer, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    /// Sends a typed message to the queue (with serde feature).
    /// 
    /// Serializes the message to bytes using the serde framework.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Reference to the message to send
    /// * `time` - Timeout in system ticks
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully serialized and sent
    /// * `Err(Error::Timeout)` - Queue full
    /// * `Err(Error::Unhandled)` - Serialization error
    fn post(&self, item: &T, time: TickType) -> Result<()> {


        let mut buf_bytes = Vec::with_capacity(item.len()); 

        to_bytes(item, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

        self.0.post(&buf_bytes, time)
    }

    /// Sends a typed message from ISR context (with serde feature).
    /// 
    /// ISR-safe version that does not block. Serializes using serde.
    /// 
    /// # Arguments
    /// 
    /// * `item` - Reference to the message to send
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message successfully serialized and sent
    /// * `Err(Error::Timeout)` - Queue is full
    /// * `Err(Error::Unhandled)` - Serialization error
    /// 
    /// # Safety
    /// 
    /// Must only be called from ISR context.
    fn post_from_isr(&self, item: &T) -> Result<()> {

        let mut buf_bytes = Vec::with_capacity(item.len()); 

        to_bytes(item, &mut buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

        self.0.post_from_isr(&buf_bytes)
    }

    /// Deletes the typed queue (serde version).
    /// 
    /// Delegates to the underlying byte queue's delete method.
    #[inline]
    fn delete(&mut self) {
        self.0.delete()
    }
}

/// Allows dereferencing to the underlying FreeRTOS queue handle.
impl<T> Deref for QueueStreamed<T> 
where 
    T: StructSerde {
    type Target = QueueHandle;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }   
}

/// Formats the typed queue for debugging purposes.
impl<T> Debug for QueueStreamed<T> 
where 
    T: StructSerde {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueueStreamed")
            .field("handle", &self.0.0)
            .finish()
    }
}

/// Formats the typed queue for display purposes.
impl<T> Display for QueueStreamed<T> 
where 
    T: StructSerde {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "QueueStreamed {{ handle: {:?} }}", self.0.0)
    }
}

