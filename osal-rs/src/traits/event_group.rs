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
/// ```ignore
/// use osal_rs::os::EventGroup;
/// use osal_rs::os::types::EventBits;
/// 
/// let events = EventGroup::new().unwrap();
/// 
/// // Task 1: Wait for specific bits
/// let bits = events.wait(0b0011, 1000);
/// if bits & 0b0011 == 0b0011 {
///     println!("Both bits 0 and 1 are set");
/// }
/// 
/// // Task 2: Set multiple bits
/// events.set(0b0011);
/// ```
pub trait EventGroup {
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
    /// The event bits value before the bits were set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Set bit 0
    /// let old = events.set(0b0001);
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
    /// ```ignore
    /// // In an interrupt handler
    /// events.set_from_isr(0b0100).ok();
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
    /// ```ignore
    /// let current = events.get();
    /// if current & 0b0001 != 0 {
    ///     // Bit 0 is set
    /// }
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
    /// ```ignore
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
    /// Blocks the calling task until ALL specified bits in the mask are set,
    /// or until the timeout expires.
    ///
    /// # Parameters
    ///
    /// * `mask` - Bit mask of bits to wait for (waits for ALL bits in mask)
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
    /// ```ignore
    /// // Wait for bits 0 and 2 with 1000 tick timeout
    /// let result = events.wait(0b0101, 1000);
    /// 
    /// if result & 0b0101 == 0b0101 {
    ///     // Success: both bits 0 and 2 are set
    ///     println!("Condition met!");
    /// } else {
    ///     // Timeout: not all bits were set in time
    ///     println!("Timeout - current bits: {:#b}", result);
    /// }
    /// 
    /// // Wait forever for a single bit
    /// let result = events.wait(0b0001, TickType::MAX);
    /// ```
    fn wait(&self, mask: EventBits, timeout_ticks: TickType) -> EventBits;

    /// Deletes the event group and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are waiting on this event group before deletion.
    /// Calling this while tasks are waiting may cause undefined behavior.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut events = EventGroup::new().unwrap();
    /// 
    /// // Use event group
    /// events.set(0b0001);
    /// 
    /// // Clean up when done
    /// events.delete();
    /// ```
    fn delete(&mut self);
}