/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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
//! ```ignore
//! use osal_rs::os::Queue;
//!
//! // Create a queue for 10 messages of 16 bytes each
//! let queue = Queue::new(10, 16).unwrap();
//!
//! // Producer task
//! let data = [1, 2, 3, 4];
//! queue.post(&data, 1000).unwrap();
//!
//! // Consumer task
//! let mut buffer = [0u8; 16];
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
/// ```ignore
/// use osal_rs::os::Queue;
/// 
/// // Create queue: 10 slots, 32 bytes per message
/// let queue = Queue::new(10, 32).unwrap();
/// 
/// // Producer sends data
/// let data = [1, 2, 3, 4];
/// queue.post(&data, 100).unwrap();
/// 
/// // Consumer receives data
/// let mut buffer = [0u8; 32];
/// queue.fetch(&mut buffer, 100).unwrap();
/// assert_eq!(&buffer[..4], &[1, 2, 3, 4]);
/// ```
pub trait Queue {
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
    /// ```ignore
    /// let mut buffer = [0u8; 16];
    /// 
    /// // Wait up to 1000 ticks
    /// match queue.fetch(&mut buffer, 1000) {
    ///     Ok(()) => println!("Received: {:?}", buffer),
    ///     Err(_) => println!("Timeout - no message available"),
    /// }
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
    /// ```ignore
    /// // In interrupt handler
    /// let mut buffer = [0u8; 16];
    /// if queue.fetch_from_isr(&mut buffer).is_ok() {
    ///     // Process message quickly
    /// }
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
    /// ```ignore
    /// let data = [1, 2, 3, 4];
    /// 
    /// // Try to send, wait up to 1000 ticks if full
    /// match queue.post(&data, 1000) {
    ///     Ok(()) => println!("Sent successfully"),
    ///     Err(_) => println!("Queue full, couldn't send"),
    /// }
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
    /// ```ignore
    /// // In interrupt handler
    /// let data = [0x42, 0x13];
    /// if queue.post_from_isr(&data).is_err() {
    ///     // Queue full, message dropped
    /// }
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
    /// ```ignore
    /// let mut queue = Queue::new(10, 16).unwrap();
    /// // Use queue...
    /// queue.delete();
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
/// ```ignore
/// use osal_rs::os::QueueStreamed;
/// use osal_rs::traits::Deserialize;
/// 
/// #[derive(Clone, Copy)]
/// struct SensorData {
///     id: u32,
///     temperature: i16,
///     humidity: u8,
/// }
/// 
/// impl Deserialize for SensorData {
///     fn from_bytes(bytes: &[u8]) -> Result<Self> {
///         // Deserialization logic
///     }
/// }
/// 
/// let queue = QueueStreamed::<SensorData>::new(10, size_of::<SensorData>()).unwrap();
/// 
/// // Producer
/// let data = SensorData { id: 1, temperature: 235, humidity: 65 };
/// queue.post(&data, 100).unwrap();
/// 
/// // Consumer
/// let mut received = SensorData { id: 0, temperature: 0, humidity: 0 };
/// queue.fetch(&mut received, 100).unwrap();
/// assert_eq!(received.id, 1);
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
    /// ```ignore
    /// let mut msg = Message::default();
    /// 
    /// match queue.fetch(&mut msg, 1000) {
    ///     Ok(()) => println!("Received message: {:?}", msg),
    ///     Err(_) => println!("No message available"),
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
    /// ```ignore
    /// // In interrupt handler
    /// let mut msg = Message::default();
    /// if queue.fetch_from_isr(&mut msg).is_ok() {
    ///     // Process message
    /// }
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
    /// ```ignore
    /// let msg = Message { id: 42, value: 100 };
    /// 
    /// match queue.post(&msg, 1000) {
    ///     Ok(()) => println!("Sent successfully"),
    ///     Err(_) => println!("Failed to send"),
    /// }
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
    /// ```ignore
    /// // In interrupt handler
    /// let msg = Message { id: 1, value: 42 };
    /// if queue.post_from_isr(&msg).is_err() {
    ///     // Queue full, message dropped
    /// }
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
    /// ```ignore
    /// let mut queue = QueueStreamed::<Message>::new(10, size_of::<Message>()).unwrap();
    /// // Use queue...
    /// queue.delete();
    /// ```
    fn delete(&mut self);
}
