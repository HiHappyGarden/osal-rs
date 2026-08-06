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

//! Event group trait for multi-bit synchronization.
//!
//! Event groups provide a mechanism for synchronizing tasks using multiple
//! independent event bits, useful for complex coordination scenarios.
//!
//! # Overview
//!
//! Event groups allow multiple tasks to synchronize based on the state of
//! multiple event bits. Each bit represents an independent event that can be
//! set, cleared, and tested independently.
//!
//! Typical use cases include:
//! - Waiting for multiple resources to become available
//! - Coordinating startup sequences
//! - Implementing state machines with multiple conditions
//! - Synchronizing multiple tasks at specific points
//!
//! # Bit Layout
//!
//! On most systems, event groups support at least 24 usable event bits.
//! The specific number depends on the underlying RTOS implementation.

use crate::utils::Result;
use crate::os::types::{EventBits, TickType};

/// Event group synchronization primitive.
///
/// Event groups allow multiple bits to be set, cleared, and waited upon,
/// enabling complex synchronization patterns between tasks.
///
/// # Thread Safety
///
/// All methods are thread-safe and can be called from multiple tasks
/// concurrently. ISR-specific methods should only be called from
/// interrupt context.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
///
/// let events = Arc::new(EventGroup::new().unwrap());
/// let producer = events.clone();
///
/// // Task 1: set both bits after a short delay
/// let mut thread = Thread::new("producer", 1024, 1);
/// let worker = thread.spawn_simple(move || {
///     System::delay(10);
///     producer.set(0b0011);
///     Ok(Arc::new(()))
/// }).unwrap();
///
/// // Task 2: block until both bits are set (or 1000 ticks elapse)
/// let bits = events.wait(0b0011, true, 1000);
/// assert_eq!(bits & 0b0011, 0b0011);
///
/// worker.delete();
/// ```
pub trait EventGroup {
    /// Returns `true` if the underlying OS handle is null, i.e. the mutex
    /// has not been created yet or has already been deleted.
    fn is_null(&self) -> bool;


    /// Sets the specified event bits.
    ///
    /// Any tasks waiting for these bits will be unblocked if their
    /// wait conditions are now satisfied. The operation performs a
    /// bitwise OR with the current event bits.
    ///
    /// # Parameters
    ///
    /// * `bits` - The bits to set (OR operation with current value)
    ///
    /// # Returns
    ///
    /// The event bits value once the bits have been set
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let events = EventGroup::new().unwrap();
    ///
    /// // Set bit 0
    /// events.set(0b0001);
    ///
    /// // Set bit 1 (bit 0 remains set)
    /// events.set(0b0010);
    ///
    /// // Now bits 0 and 1 are both set
    /// assert_eq!(events.get(), 0b0011);
    /// ```
    fn set(&self, bits: EventBits) -> EventBits;

    /// Sets event bits from an interrupt service routine.
    ///
    /// ISR-safe version of `set()`.
    ///
    /// # Parameters
    ///
    /// * `bits` - The bits to set
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Bits set successfully
    /// * `Err(Error)` - Operation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let events = EventGroup::new().unwrap();
    ///
    /// // In an interrupt handler
    /// events.set_from_isr(0b0100).ok();
    ///
    /// assert_eq!(events.get(), 0b0100);
    /// ```
    fn set_from_isr(&self, bits: EventBits) -> Result<()>;

    /// Gets the current value of the event bits.
    ///
    /// # Returns
    ///
    /// Current state of all event bits
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let events = EventGroup::new().unwrap();
    /// events.set(0b0001);
    ///
    /// let current = events.get();
    /// assert!(current & 0b0001 != 0); // bit 0 is set
    /// assert!(current & 0b0010 == 0); // bit 1 is not
    /// ```
    fn get(&self) -> EventBits;

    /// Gets event bits from an interrupt service routine.
    ///
    /// ISR-safe version of `get()`.
    ///
    /// # Returns
    ///
    /// Current state of all event bits
    fn get_from_isr(&self) -> EventBits;

    /// Clears the specified event bits.
    ///
    /// The operation performs a bitwise AND NOT with the current event bits.
    ///
    /// # Parameters
    ///
    /// * `bits` - The bits to clear (AND NOT operation with current value)
    ///
    /// # Returns
    ///
    /// The event bits value before the bits were cleared
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let events = EventGroup::new().unwrap();
    ///
    /// // Start with bits 0 and 1 set
    /// events.set(0b0011);
    ///
    /// // Clear bit 0
    /// let old = events.clear(0b0001);
    /// assert_eq!(old, 0b0011);
    ///
    /// // Now only bit 1 is set
    /// assert_eq!(events.get(), 0b0010);
    /// ```
    fn clear(&self, bits: EventBits) -> EventBits;
    
    /// Clears event bits from an interrupt service routine.
    ///
    /// ISR-safe version of `clear()`.
    ///
    /// # Parameters
    ///
    /// * `bits` - The bits to clear
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Bits cleared successfully
    /// * `Err(Error)` - Operation failed
    fn clear_from_isr(&self, bits: EventBits) -> Result<()>;

    /// Waits for specific event bits to be set.
    ///
    /// Blocks the calling task until either ALL or ANY (depending on
    /// `wait_for_all_bits`) of the bits in `mask` are set, or until the
    /// timeout expires.
    ///
    /// # Parameters
    ///
    /// * `mask` - Bit mask of bits to wait for
    /// * `wait_for_all_bits` - `true` to wait for every bit in `mask` to be
    ///   set (AND); `false` to wait for any single bit in `mask` to be set
    ///   (OR)
    /// * `timeout_ticks` - Maximum time to wait in ticks (0 = no wait, MAX = wait forever)
    ///
    /// # Returns
    ///
    /// The event bits value when the wait condition was satisfied,
    /// or the current value if timeout occurred. Check if the mask bits
    /// are set to determine success.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::os::types::TickType;
    ///
    /// let events = EventGroup::new().unwrap();
    /// events.set(0b0001);
    ///
    /// // Wait for both bits 0 and 2, with a 10 tick timeout: only bit 0 is
    /// // set, so this times out and reports the current bits instead.
    /// let result = events.wait(0b0101, true, 10);
    /// assert_ne!(result & 0b0101, 0b0101);
    ///
    /// // Waiting for *either* bit 0 or bit 2 is already satisfied, so this
    /// // returns immediately even with the "wait forever" timeout.
    /// let result = events.wait(0b0101, false, TickType::MAX);
    /// assert_eq!(result & 0b0001, 0b0001);
    /// ```
    fn wait(&self, mask: EventBits, wait_for_all_bits: bool, timeout_ticks: TickType) -> EventBits;

    /// Deletes the event group and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are waiting on this event group before deletion.
    /// Calling this while tasks are waiting may cause undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let mut events = EventGroup::new().unwrap();
    ///
    /// // Use event group
    /// events.set(0b0001);
    ///
    /// // Clean up when done
    /// events.delete();
    /// assert!(events.is_null());
    /// ```
    fn delete(&mut self);
}