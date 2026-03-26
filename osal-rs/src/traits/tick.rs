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
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 *
 ***************************************************************************/

//! Tick conversion traits for time-based operations.
//!
//! These traits provide conversion between high-level time types (like `Duration`)
//! and low-level RTOS tick counts.
//!
//! # Overview
//!
//! RTOS systems use a periodic tick interrupt for timing operations. The tick rate
//! (typically 100-1000 Hz) determines the time resolution of delays, timeouts, and
//! other timing operations.
//!
//! # Tick Rate
//!
//! The tick rate is configured at compile time and affects:
//! - Minimum delay/timeout resolution
//! - System responsiveness
//! - Interrupt overhead (higher rate = more overhead)
//! - Maximum timeout duration before overflow
//!
//! Common tick rates:
//! - 100 Hz (10ms per tick) - Lower overhead, suitable for many applications
//! - 1000 Hz (1ms per tick) - Higher resolution, common in modern systems
//!
//! # Conversion Accuracy
//!
//! Converting from `Duration` to ticks involves rounding. Sub-tick durations
//! are rounded up to ensure minimum delays are met.
//!
//! # Examples
//!
//! ```ignore
//! use osal_rs::traits::{ToTick, FromTick};
//! use core::time::Duration;
//!
//! // Convert Duration to ticks
//! let timeout = Duration::from_millis(100);
//! let ticks = timeout.to_ticks();
//!
//! // Convert ticks to Duration
//! let mut duration = Duration::from_secs(0);
//! duration.ticks(50);
//! ```

use crate::os::types::TickType;


/// Converts a time value to RTOS ticks.
///
/// This trait is implemented by time types (like `Duration`) to allow
/// conversion to the underlying RTOS tick count. This is useful for API
/// functions that accept tick-based timeouts.
///
/// # Rounding Behavior
///
/// Conversions typically round up to ensure that the actual delay/timeout
/// is at least as long as requested. For example, with a 1ms tick rate:
/// - 500µs → rounds up to 1 tick (1ms)
/// - 1500µs → rounds up to 2 ticks (2ms)
///
/// # Requirements
///
/// Types implementing this trait must be `Sized + Copy` to allow efficient
/// passing by value in RTOS operations.
///
/// # Use Cases
///
/// - API functions that need tick-based timeouts
/// - Converting user-friendly durations to system ticks
/// - Allowing flexible timeout specifications in APIs
///
/// # Examples
///
/// ```ignore
/// use osal_rs::traits::ToTick;
/// use core::time::Duration;
/// 
/// // Convert milliseconds to ticks
/// let duration = Duration::from_millis(100);
/// let ticks = duration.to_ticks();
/// 
/// // Use with RTOS API
/// System::delay(Duration::from_millis(500).to_ticks());
/// 
/// // Or let the API handle the conversion
/// fn delay_for(timeout: impl ToTick) {
///     let ticks = timeout.to_ticks();
///     // Use ticks...
/// }
/// 
/// delay_for(Duration::from_secs(1));
/// delay_for(1000u32);  // If u32 implements ToTick
/// ```
pub trait ToTick : Sized + Copy {
    /// Converts this value to RTOS ticks.
    ///
    /// Converts the time value to the equivalent number of system ticks
    /// based on the configured tick rate. The conversion rounds up to
    /// ensure the actual time is at least as long as requested.
    ///
    /// # Returns
    ///
    /// The number of ticks equivalent to this time value
    ///
    /// # Overflow
    ///
    /// Very large time values may overflow the `TickType`. Implementations
    /// should either saturate at `TickType::MAX` or document overflow behavior.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::traits::ToTick;
    /// use core::time::Duration;
    ///
    /// let timeout = Duration::from_millis(250);
    /// let ticks = timeout.to_ticks();
    ///
    /// // With 1000 Hz tick rate (1ms per tick)
    /// // 250ms = 250 ticks
    /// assert_eq!(ticks, 250);
    /// ```
    fn to_ticks(&self) -> TickType;
} 


/// Converts RTOS ticks to a time value.
///
/// This trait allows time types to be constructed or updated from
/// raw tick counts, useful when working with RTOS APIs that return
/// tick-based values.
///
/// # Use Cases
///
/// - Converting tick counts from RTOS APIs to `Duration`
/// - Calculating elapsed time from tick differences
/// - Interpreting tick-based timestamps
///
/// # Tick Rate Dependency
///
/// The conversion depends on the configured tick rate. For example:
/// - With 1000 Hz (1ms/tick): 100 ticks = 100ms
/// - With 100 Hz (10ms/tick): 100 ticks = 1000ms (1 second)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::traits::FromTick;
/// use core::time::Duration;
/// 
/// // Create duration from tick count
/// let mut duration = Duration::from_secs(0);
/// duration.ticks(100);  // Set from 100 ticks
/// 
/// // Calculate elapsed time
/// let start_tick = System::get_tick_count();
/// // ... do work ...
/// let end_tick = System::get_tick_count();
/// let mut elapsed = Duration::from_secs(0);
/// elapsed.ticks(end_tick - start_tick);
/// ```
pub trait FromTick {
    /// Sets this value from the given tick count.
    ///
    /// Converts the tick count to the appropriate time representation
    /// based on the configured system tick rate.
    ///
    /// # Parameters
    ///
    /// * `tick` - The tick count to convert from
    ///
    /// # Precision
    ///
    /// The resulting time value's precision is limited by the tick rate.
    /// Sub-tick resolution is not possible.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::traits::FromTick;
    /// use core::time::Duration;
    ///
    /// let mut duration = Duration::from_secs(0);
    /// 
    /// // With 1000 Hz tick rate (1ms per tick)
    /// duration.ticks(500);
    /// assert_eq!(duration.as_millis(), 500);
    ///
    /// // Calculate time between two tick counts
    /// let tick1 = 1000;
    /// let tick2 = 1250;
    /// duration.ticks(tick2 - tick1);  // 250 ticks
    /// assert_eq!(duration.as_millis(), 250);
    /// ```
    fn ticks(&mut self, tick: TickType);
}