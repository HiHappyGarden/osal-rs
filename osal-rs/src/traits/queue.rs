/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Queue traits for inter-task communication.
//!
//! Provides both raw byte-based queues and type-safe streamed queues
//! for message passing between tasks.
//!
//! # Overview
//!
//! Queues implement FIFO (First-In-First-Out) message passing between tasks,
//! enabling the producer-consumer pattern and other inter-task communication
//! patterns. Messages are copied into and out of the queue.
//!
//! # Queue Types
//!
//! - **`Queue`**: Raw byte-oriented queue for variable-sized or untyped data
//! - **`QueueStreamed<T>`**: Type-safe queue for structured messages
//!
//! # Communication Patterns
//!
//! - **Producer-Consumer**: One or more producers send messages, one consumer processes them
//! - **Work Queue**: Distribute tasks among multiple worker tasks
//! - **Event Notification**: Send status updates or notifications between tasks
//!
//! # Timeout Behavior
//!
//! - `0`: Non-blocking - return immediately if queue is full/empty
//! - `n`: Wait up to `n` ticks for space/data to become available
//! - `TickType::MAX`: Block indefinitely until operation succeeds
//!
//! # Examples
//!
//! ```
//! use osal_rs::os::{Queue, QueueFn};
//!
//! // Create a queue for 10 messages of 4 bytes each
//! let queue = Queue::new(10, 4).unwrap();
//!
//! // Producer task
//! let data = [1u8, 2, 3, 4];
//! queue.post(&data, 1000).unwrap();
//!
//! // Consumer task
//! let mut buffer = [0u8; 4];
//! queue.fetch(&mut buffer, 1000).unwrap();
//! ```
#[cfg(not(feature = "serde"))]
use crate::os::Deserialize;

#[cfg(feature = "serde")]
use osal_rs_serde::Deserialize;

use crate::os::types::TickType;
use crate::utils::Result;

/// Raw byte-oriented queue for inter-task message passing.
///
/// This trait defines a FIFO queue that works with raw byte arrays,
/// suitable for variable-sized messages or when type safety is not required.
///
/// # Memory Layout
///
/// The queue capacity is fixed at creation time. Each message slot can
/// hold up to the maximum message size specified during creation.
///
/// # Thread Safety
///
/// All methods are thread-safe. Multiple producers and consumers can
/// safely access the same queue concurrently.
///
/// # Performance
///
/// Messages are copied into and out of the queue. For large messages,
/// consider using a queue of pointers or references instead.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
///
/// // Create queue: 10 slots, 32 bytes per message
/// let queue = Queue::new(10, 32).unwrap();
///
/// // Producer sends data - a whole message-sized slot at a time
/// let mut data = [0u8; 32];
/// data[..4].copy_from_slice(&[1, 2, 3, 4]);
/// queue.post(&data, 100).unwrap();
///
/// // Consumer receives data
/// let mut buffer = [0u8; 32];
/// queue.fetch(&mut buffer, 100).unwrap();
/// assert_eq!(&buffer[..4], &[1, 2, 3, 4]);
/// ```
pub trait Queue {

    /// Returns `true` if the underlying OS handle is null, i.e. the mutex
    /// has not been created yet or has already been deleted.
    fn is_null(&self) -> bool;

    /// Fetches a message from the queue (blocking).
    ///
    /// Removes and retrieves the oldest message from the queue (FIFO order).
    /// Blocks the calling task if the queue is empty.
    ///
    /// # Parameters
    ///
    /// * `buffer` - Buffer to receive the message data (should match queue message size)
    /// * `time` - Maximum ticks to wait for a message:
    ///   - `0`: Return immediately if empty
    ///   - `n`: Wait up to `n` ticks
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received successfully
    /// * `Err(Error::Timeout)` - Queue was empty for entire timeout period
    /// * `Err(Error)` - Other error occurred
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let queue = Queue::new(4, 16).unwrap();
    /// queue.post(&[0xAAu8; 16], 100).unwrap();
    ///
    /// let mut buffer = [0u8; 16];
    ///
    /// // Wait up to 1000 ticks
    /// match queue.fetch(&mut buffer, 1000) {
    ///     Ok(()) => assert_eq!(buffer, [0xAAu8; 16]),
    ///     Err(_) => panic!("timeout - no message available"),
    /// }
    ///
    /// // The queue is empty again, so this one does time out.
    /// assert!(queue.fetch(&mut buffer, 10).is_err());
    /// ```
    fn fetch(&self, buffer: &mut [u8], time: TickType) -> Result<()>;

    /// Fetches a message from ISR context (non-blocking).
    ///
    /// ISR-safe version of `fetch()`. Returns immediately without blocking.
    /// Must only be called from interrupt context.
    ///
    /// # Parameters
    ///
    /// * `buffer` - Buffer to receive the message data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received successfully
    /// * `Err(Error)` - Queue is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let queue = Queue::new(4, 16).unwrap();
    /// queue.post(&[7u8; 16], 100).unwrap();
    ///
    /// // In interrupt handler
    /// let mut buffer = [0u8; 16];
    /// if queue.fetch_from_isr(&mut buffer).is_ok() {
    ///     // Process message quickly
    ///     assert_eq!(buffer, [7u8; 16]);
    /// }
    ///
    /// // Nothing left: reported immediately instead of blocking the "ISR".
    /// assert!(queue.fetch_from_isr(&mut buffer).is_err());
    /// ```
    fn fetch_from_isr(&self, buffer: &mut [u8]) -> Result<()>;
    
    /// Posts a message to the queue (blocking).
    ///
    /// Adds a new message to the end of the queue (FIFO order).
    /// Blocks the calling task if the queue is full.
    ///
    /// # Parameters
    ///
    /// * `item` - The message data to send (must not exceed queue message size)
    /// * `time` - Maximum ticks to wait if queue is full:
    ///   - `0`: Return immediately if full
    ///   - `n`: Wait up to `n` ticks for space
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(Error::Timeout)` - Queue was full for entire timeout period
    /// * `Err(Error)` - Other error occurred
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// // Room for a single 4-byte message.
    /// let queue = Queue::new(1, 4).unwrap();
    /// let data = [1u8, 2, 3, 4];
    ///
    /// // Try to send, wait up to 1000 ticks if full
    /// match queue.post(&data, 1000) {
    ///     Ok(()) => (), // sent successfully
    ///     Err(_) => panic!("queue full, couldn't send"),
    /// }
    ///
    /// // The only slot is taken and nobody is fetching: this one times out.
    /// assert!(queue.post(&data, 10).is_err());
    /// ```
    fn post(&self, item: &[u8], time: TickType) -> Result<()>;
    
    /// Posts a message from ISR context (non-blocking).
    ///
    /// ISR-safe version of `post()`. Returns immediately without blocking.
    /// Must only be called from interrupt context.
    ///
    /// # Parameters
    ///
    /// * `item` - The message data to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(Error)` - Queue is full
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let queue = Queue::new(1, 2).unwrap();
    ///
    /// // In interrupt handler
    /// let data = [0x42u8, 0x13];
    /// if queue.post_from_isr(&data).is_err() {
    ///     // Queue full, message dropped
    /// }
    ///
    /// // The single slot is now taken, so the next one really is dropped.
    /// assert!(queue.post_from_isr(&data).is_err());
    /// ```
    fn post_from_isr(&self, item: &[u8]) -> Result<()>;

    /// Deletes the queue and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are blocked on this queue before deletion.
    /// Calling this while tasks are waiting may cause undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let mut queue = Queue::new(10, 16).unwrap();
    /// // Use queue...
    /// queue.delete();
    /// assert!(queue.is_null());
    /// ```
    fn delete(&mut self);
}

/// Type-safe queue for structured message passing.
///
/// This trait provides a queue that works with specific types,
/// offering compile-time type safety for queue operations.
///
/// # Type Safety
///
/// Unlike raw `Queue`, `QueueStreamed` ensures that only messages
/// of type `T` can be sent and received, preventing type confusion
/// at compile time.
///
/// # Serialization
///
/// Messages are automatically serialized when sent and deserialized
/// when received. The type `T` must implement the `Deserialize` trait.
///
/// # Type Parameters
///
/// * `T` - The message type (must implement `Deserialize`)
///
/// # Examples
///
#[cfg_attr(not(feature = "serde"), doc = "```")]
#[cfg_attr(feature = "serde", doc = "```ignore")]
/// use osal_rs::os::*;
/// use osal_rs::utils::{Error, Result};
///
/// // Wire format: 4 bytes of `id`, 2 of `temperature`, 1 of `humidity`,
/// // little-endian. Keeping the message in its encoded form is what lets
/// // `to_bytes` hand out a borrowed slice.
/// const SENSOR_DATA_LEN: usize = 7;
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// struct SensorData([u8; SENSOR_DATA_LEN]);
///
/// impl SensorData {
///     fn new(id: u32, temperature: i16, humidity: u8) -> Self {
///         let mut raw = [0u8; SENSOR_DATA_LEN];
///         raw[..4].copy_from_slice(&id.to_le_bytes());
///         raw[4..6].copy_from_slice(&temperature.to_le_bytes());
///         raw[6] = humidity;
///         Self(raw)
///     }
///
///     fn id(&self) -> u32 {
///         u32::from_le_bytes(self.0[..4].try_into().unwrap())
///     }
/// }
///
/// impl BytesHasLen for SensorData {
///     fn len(&self) -> usize { SENSOR_DATA_LEN }
/// }
///
/// impl Serialize for SensorData {
///     fn to_bytes(&self) -> &[u8] { &self.0 }
/// }
///
/// impl Deserialize for SensorData {
///     fn from_bytes(bytes: &[u8]) -> Result<Self> {
///         if bytes.len() < SENSOR_DATA_LEN {
///             return Err(Error::OutOfIndex);
///         }
///         let mut raw = [0u8; SENSOR_DATA_LEN];
///         raw.copy_from_slice(&bytes[..SENSOR_DATA_LEN]);
///         Ok(Self(raw))
///     }
/// }
///
/// let queue = QueueStreamed::<SensorData>::new(10, SENSOR_DATA_LEN as _).unwrap();
///
/// // Producer
/// let data = SensorData::new(1, 235, 65);
/// queue.post(&data, 100).unwrap();
///
/// // Consumer
/// let mut received = SensorData::new(0, 0, 0);
/// queue.fetch(&mut received, 100).unwrap();
/// assert_eq!(received.id(), 1);
/// assert_eq!(received, data);
/// ```

pub trait QueueStreamed<T> 
where 
    T: Deserialize + Sized {

    /// Fetches a typed message from the queue (blocking).
    ///
    /// Removes and deserializes the oldest message from the queue.
    /// Blocks the calling task if the queue is empty.
    ///
    /// # Parameters
    ///
    /// * `buffer` - Mutable reference to receive the deserialized message
    /// * `time` - Maximum ticks to wait for a message:
    ///   - `0`: Return immediately if empty
    ///   - `n`: Wait up to `n` ticks
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received and deserialized successfully
    /// * `Err(Error::Timeout)` - Queue was empty for entire timeout period
    /// * `Err(Error)` - Deserialization error or other error
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "serde"), doc = "```")]
    #[cfg_attr(feature = "serde", doc = "```ignore")]
    /// use osal_rs::os::*;
    /// # use osal_rs::utils::{Error, Result};
    /// # #[derive(Clone, Copy, Debug, Default, PartialEq)]
    /// # struct Message([u8; 4]);
    /// # impl Message {
    /// #     fn new(id: u32) -> Self { Self(id.to_le_bytes()) }
    /// #     fn id(&self) -> u32 { u32::from_le_bytes(self.0) }
    /// # }
    /// # impl BytesHasLen for Message { fn len(&self) -> usize { 4 } }
    /// # impl Serialize for Message { fn to_bytes(&self) -> &[u8] { &self.0 } }
    /// # impl Deserialize for Message {
    /// #     fn from_bytes(bytes: &[u8]) -> Result<Self> {
    /// #         if bytes.len() < 4 { return Err(Error::OutOfIndex); }
    /// #         Ok(Self(bytes[..4].try_into().unwrap()))
    /// #     }
    /// # }
    /// let queue = QueueStreamed::<Message>::new(4, 4).unwrap();
    /// queue.post(&Message::new(42), 100).unwrap();
    ///
    /// let mut msg = Message::default();
    ///
    /// match queue.fetch(&mut msg, 1000) {
    ///     Ok(()) => assert_eq!(msg.id(), 42),
    ///     Err(_) => panic!("no message available"),
    /// }
    /// ```
    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()>;

    /// Fetches a typed message from ISR context (non-blocking).
    ///
    /// ISR-safe version of `fetch()`. Returns immediately without blocking.
    /// Must only be called from interrupt context.
    ///
    /// # Parameters
    ///
    /// * `buffer` - Mutable reference to receive the deserialized message
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received and deserialized successfully
    /// * `Err(Error)` - Queue is empty or deserialization failed
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "serde"), doc = "```")]
    #[cfg_attr(feature = "serde", doc = "```ignore")]
    /// use osal_rs::os::*;
    /// # use osal_rs::utils::{Error, Result};
    /// # #[derive(Clone, Copy, Debug, Default, PartialEq)]
    /// # struct Message([u8; 4]);
    /// # impl Message {
    /// #     fn new(id: u32) -> Self { Self(id.to_le_bytes()) }
    /// #     fn id(&self) -> u32 { u32::from_le_bytes(self.0) }
    /// # }
    /// # impl BytesHasLen for Message { fn len(&self) -> usize { 4 } }
    /// # impl Serialize for Message { fn to_bytes(&self) -> &[u8] { &self.0 } }
    /// # impl Deserialize for Message {
    /// #     fn from_bytes(bytes: &[u8]) -> Result<Self> {
    /// #         if bytes.len() < 4 { return Err(Error::OutOfIndex); }
    /// #         Ok(Self(bytes[..4].try_into().unwrap()))
    /// #     }
    /// # }
    /// let queue = QueueStreamed::<Message>::new(4, 4).unwrap();
    /// queue.post(&Message::new(7), 100).unwrap();
    ///
    /// // In interrupt handler
    /// let mut msg = Message::default();
    /// if queue.fetch_from_isr(&mut msg).is_ok() {
    ///     // Process message
    ///     assert_eq!(msg.id(), 7);
    /// }
    ///
    /// // Queue empty: reported immediately instead of blocking the "ISR".
    /// assert!(queue.fetch_from_isr(&mut msg).is_err());
    /// ```
    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()>;
    
    /// Posts a typed message to the queue (blocking).
    ///
    /// Serializes and adds a new message to the end of the queue.
    /// Blocks the calling task if the queue is full.
    ///
    /// # Parameters
    ///
    /// * `item` - Reference to the message to serialize and send
    /// * `time` - Maximum ticks to wait if queue is full:
    ///   - `0`: Return immediately if full
    ///   - `n`: Wait up to `n` ticks for space
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message serialized and sent successfully
    /// * `Err(Error::Timeout)` - Queue was full for entire timeout period
    /// * `Err(Error)` - Serialization error or other error
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "serde"), doc = "```")]
    #[cfg_attr(feature = "serde", doc = "```ignore")]
    /// use osal_rs::os::*;
    /// # use osal_rs::utils::{Error, Result};
    /// # #[derive(Clone, Copy, Debug, Default, PartialEq)]
    /// # struct Message([u8; 4]);
    /// # impl Message {
    /// #     fn new(id: u32) -> Self { Self(id.to_le_bytes()) }
    /// #     fn id(&self) -> u32 { u32::from_le_bytes(self.0) }
    /// # }
    /// # impl BytesHasLen for Message { fn len(&self) -> usize { 4 } }
    /// # impl Serialize for Message { fn to_bytes(&self) -> &[u8] { &self.0 } }
    /// # impl Deserialize for Message {
    /// #     fn from_bytes(bytes: &[u8]) -> Result<Self> {
    /// #         if bytes.len() < 4 { return Err(Error::OutOfIndex); }
    /// #         Ok(Self(bytes[..4].try_into().unwrap()))
    /// #     }
    /// # }
    /// // Room for a single message.
    /// let queue = QueueStreamed::<Message>::new(1, 4).unwrap();
    /// let msg = Message::new(42);
    ///
    /// match queue.post(&msg, 1000) {
    ///     Ok(()) => (), // sent successfully
    ///     Err(_) => panic!("failed to send"),
    /// }
    ///
    /// // The only slot is taken and nobody is fetching: this one times out.
    /// assert!(queue.post(&msg, 10).is_err());
    /// ```
    fn post(&self, item: &T, time: TickType) -> Result<()>;

    /// Posts a typed message from ISR context (non-blocking).
    ///
    /// ISR-safe version of `post()`. Returns immediately without blocking.
    /// Must only be called from interrupt context.
    ///
    /// # Parameters
    ///
    /// * `item` - Reference to the message to serialize and send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message serialized and sent successfully
    /// * `Err(Error)` - Queue is full or serialization failed
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "serde"), doc = "```")]
    #[cfg_attr(feature = "serde", doc = "```ignore")]
    /// use osal_rs::os::*;
    /// # use osal_rs::utils::{Error, Result};
    /// # #[derive(Clone, Copy, Debug, Default, PartialEq)]
    /// # struct Message([u8; 4]);
    /// # impl Message {
    /// #     fn new(id: u32) -> Self { Self(id.to_le_bytes()) }
    /// #     fn id(&self) -> u32 { u32::from_le_bytes(self.0) }
    /// # }
    /// # impl BytesHasLen for Message { fn len(&self) -> usize { 4 } }
    /// # impl Serialize for Message { fn to_bytes(&self) -> &[u8] { &self.0 } }
    /// # impl Deserialize for Message {
    /// #     fn from_bytes(bytes: &[u8]) -> Result<Self> {
    /// #         if bytes.len() < 4 { return Err(Error::OutOfIndex); }
    /// #         Ok(Self(bytes[..4].try_into().unwrap()))
    /// #     }
    /// # }
    /// let queue = QueueStreamed::<Message>::new(1, 4).unwrap();
    ///
    /// // In interrupt handler
    /// let msg = Message::new(1);
    /// if queue.post_from_isr(&msg).is_err() {
    ///     // Queue full, message dropped
    /// }
    ///
    /// // The single slot is now taken, so the next one really is dropped.
    /// assert!(queue.post_from_isr(&msg).is_err());
    /// ```
    fn post_from_isr(&self, item: &T) -> Result<()>;

    /// Deletes the queue and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are blocked on this queue before deletion.
    /// Calling this while tasks are waiting may cause undefined behavior.
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "serde"), doc = "```")]
    #[cfg_attr(feature = "serde", doc = "```ignore")]
    /// use osal_rs::os::*;
    /// # use osal_rs::utils::{Error, Result};
    /// # #[derive(Clone, Copy, Debug, Default, PartialEq)]
    /// # struct Message([u8; 4]);
    /// # impl Message {
    /// #     fn new(id: u32) -> Self { Self(id.to_le_bytes()) }
    /// #     fn id(&self) -> u32 { u32::from_le_bytes(self.0) }
    /// # }
    /// # impl BytesHasLen for Message { fn len(&self) -> usize { 4 } }
    /// # impl Serialize for Message { fn to_bytes(&self) -> &[u8] { &self.0 } }
    /// # impl Deserialize for Message {
    /// #     fn from_bytes(bytes: &[u8]) -> Result<Self> {
    /// #         if bytes.len() < 4 { return Err(Error::OutOfIndex); }
    /// #         Ok(Self(bytes[..4].try_into().unwrap()))
    /// #     }
    /// # }
    /// let mut queue = QueueStreamed::<Message>::new(10, core::mem::size_of::<Message>() as _).unwrap();
    ///
    /// // Use queue...
    /// queue.post(&Message::new(1), 100).unwrap();
    ///
    /// queue.delete();
    /// ```
    fn delete(&mut self);
}
