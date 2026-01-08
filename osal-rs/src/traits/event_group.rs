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

//! Event group trait for multi-bit synchronization.
//!
//! Event groups provide a mechanism for synchronizing tasks using multiple
//! independent event bits, useful for complex coordination scenarios.

use crate::utils::Result;
use crate::os::types::{EventBits, TickType};

/// Event group synchronization primitive.
///
/// Event groups allow multiple bits to be set, cleared, and waited upon,
/// enabling complex synchronization patterns between tasks.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{EventGroup, EventGroupFn};
/// 
/// let events = EventGroup::new().unwrap();
/// 
/// // Set multiple bits
/// events.set(0b0101);
/// 
/// // Wait for specific bits
/// let bits = events.wait(0b0101, 1000);
/// ```
pub trait EventGroup {
    /// Sets the specified event bits.
    ///
    /// Any tasks waiting for these bits will be unblocked if their
    /// wait conditions are now satisfied.
    ///
    /// # Parameters
    ///
    /// * `bits` - The bits to set (OR operation)
    ///
    /// # Returns
    ///
    /// The event bits value before the bits were set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// events.set(0b0001);  // Set bit 0
    /// events.set(0b0010);  // Set bit 1 (bit 0 remains set)
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
    /// # Parameters
    ///
    /// * `bits` - The bits to clear (AND NOT operation)
    ///
    /// # Returns
    ///
    /// The event bits value before the bits were cleared
    ///
    /// # Examples
    ///
    /// ```ignore
    /// events.clear(0b0001);  // Clear bit 0
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
    /// Blocks the calling task until the specified bits are set or timeout occurs.
    ///
    /// # Parameters
    ///
    /// * `mask` - Bit mask of bits to wait for
    /// * `timeout_ticks` - Maximum time to wait in ticks
    ///
    /// # Returns
    ///
    /// The event bits value when the wait condition was satisfied,
    /// or the current value if timeout occurred
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Wait for bits 0 and 2 with 1000 tick timeout
    /// let result = events.wait(0b0101, 1000);
    /// if result & 0b0101 == 0b0101 {
    ///     // Both bits were set
    /// }
    /// ```
    fn wait(&self, mask: EventBits, timeout_ticks: TickType) -> EventBits;

    /// Deletes the event group and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are waiting on this event group before deletion.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut events = EventGroup::new().unwrap();
    /// // ... use event group ...
    /// events.delete();
    /// ```
    fn delete(&mut self);
}