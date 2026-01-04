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

//! Tick conversion traits for time-based operations.
//!
//! These traits provide conversion between high-level time types (like `Duration`)
//! and low-level RTOS tick counts.

use crate::os::types::TickType;


/// Converts a time value to RTOS ticks.
///
/// This trait is implemented by time types (like `Duration`) to allow
/// conversion to the underlying RTOS tick count. This is useful for API
/// functions that accept tick-based timeouts.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::ToTick;
/// use core::time::Duration;
/// 
/// let duration = Duration::from_millis(100);
/// let ticks = duration.to_ticks();
/// ```
pub trait ToTick : Sized + Copy {
    /// Converts this value to RTOS ticks.
    ///
    /// # Returns
    ///
    /// The number of ticks equivalent to this time value
    fn to_ticks(&self) -> TickType;
} 


/// Converts RTOS ticks to a time value.
///
/// This trait allows time types to be constructed or updated from
/// raw tick counts, useful when working with RTOS APIs that return
/// tick-based values.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::FromTick;
/// use core::time::Duration;
/// 
/// let mut duration = Duration::from_secs(0);
/// duration.ticks(100);  // Set from 100 ticks
/// ```
pub trait FromTick {
    /// Sets this value from the given tick count.
    ///
    /// # Parameters
    ///
    /// * `tick` - The tick count to convert from
    fn ticks(&mut self, tick: TickType);
}