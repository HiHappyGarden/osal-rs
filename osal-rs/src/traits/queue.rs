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

//! Queue traits for inter-task communication.
//!
//! Provides both raw byte-based queues and type-safe streamed queues
//! for message passing between tasks.

use crate::os::ToBytes;
use crate::os::types::{UBaseType, TickType};
use crate::utils::Result;

/// Raw byte-oriented queue for inter-task message passing.
///
/// This trait defines a FIFO queue that works with raw byte arrays,
/// suitable for variable-sized messages or when type safety is not required.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Queue, QueueFn};
/// 
/// let queue = Queue::new(10, 32).unwrap();  // 10 messages, 32 bytes each
/// 
/// let data = [1, 2, 3, 4];
/// queue.post(&data, 100).unwrap();
/// 
/// let mut buffer = [0u8; 32];
/// queue.fetch(&mut buffer, 100).unwrap();
/// ```
pub trait Queue {
    /// Fetches a message from the queue (blocking).
    ///
    /// Removes and retrieves the oldest message from the queue.
    ///
    /// # Parameters
    ///
    /// * `buffer` - Buffer to receive the message data
    /// * `time` - Maximum ticks to wait for a message
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received successfully
    /// * `Err(Error)` - Timeout or other error occurred
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut buffer = [0u8; 16];
    /// queue.fetch(&mut buffer, 1000).unwrap();
    /// ```
    fn fetch(&self, buffer: &mut [u8], time: TickType) -> Result<()>;

    /// Fetches a message from ISR context (non-blocking).
    ///
    /// ISR-safe version of `fetch()`. Does not block.
    ///
    /// # Parameters
    ///
    /// * `buffer` - Buffer to receive the message data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received
    /// * `Err(Error)` - Queue empty or error
    fn fetch_from_isr(&self, buffer: &mut [u8]) -> Result<()>;
    
    /// Posts a message to the queue (blocking).
    ///
    /// Adds a new message to the end of the queue.
    ///
    /// # Parameters
    ///
    /// * `item` - The message data to send
    /// * `time` - Maximum ticks to wait if queue is full
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(Error)` - Timeout or error occurred
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let data = [1, 2, 3, 4];
    /// queue.post(&data, 1000).unwrap();
    /// ```
    fn post(&self, item: &[u8], time: TickType) -> Result<()>;
    
    /// Posts a message from ISR context (non-blocking).
    ///
    /// ISR-safe version of `post()`. Does not block.
    ///
    /// # Parameters
    ///
    /// * `item` - The message data to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent
    /// * `Err(Error)` - Queue full or error
    fn post_from_isr(&self, item: &[u8]) -> Result<()>;

    /// Deletes the queue and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are blocked on this queue before deletion.
    fn delete(&mut self);
}

/// Type-safe queue for structured message passing.
///
/// This trait provides a queue that works with specific types,
/// offering compile-time type safety for queue operations.
///
/// # Type Parameters
///
/// * `T` - The message type (must implement `ToBytes`)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{QueueStreamed, QueueStreamedFn, ToBytes};
/// 
/// struct Message {
///     id: u32,
///     value: i16,
/// }
/// 
/// let queue = QueueStreamed::<Message>::new(10, size_of::<Message>()).unwrap();
/// 
/// let msg = Message { id: 1, value: 42 };
/// queue.post(&msg, 100).unwrap();
/// 
/// let mut received = Message { id: 0, value: 0 };
/// queue.fetch(&mut received, 100).unwrap();
/// ```
pub trait QueueStreamed<T> 
where 
    T: ToBytes + Sized {


    /// Fetches a typed message from the queue (blocking).
    ///
    /// # Parameters
    ///
    /// * `buffer` - Mutable reference to receive the message
    /// * `time` - Maximum ticks to wait
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received
    /// * `Err(Error)` - Timeout or error
    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()>;

    /// Fetches a typed message from ISR context (non-blocking).
    ///
    /// # Parameters
    ///
    /// * `buffer` - Mutable reference to receive the message
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message received
    /// * `Err(Error)` - Queue empty or error
    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()>;
    
    /// Posts a typed message to the queue (blocking).
    ///
    /// # Parameters
    ///
    /// * `item` - Reference to the message to send
    /// * `time` - Maximum ticks to wait if full
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent
    /// * `Err(Error)` - Timeout or error
    fn post(&self, item: &T, time: TickType) -> Result<()>;

    /// Posts a typed message from ISR context (non-blocking).
    ///
    /// # Parameters
    ///
    /// * `item` - Reference to the message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent
    /// * `Err(Error)` - Queue full or error
    fn post_from_isr(&self, item: &T) -> Result<()>;

    /// Deletes the queue and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are blocked on this queue before deletion.
    fn delete(&mut self);
}