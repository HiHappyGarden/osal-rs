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

//! Duration conversion traits for FreeRTOS tick-based timing.
//!
//! This module implements conversion traits between standard `Duration` types and
//! FreeRTOS ticks, allowing seamless integration with RTOS timing primitives.

use core::time::Duration;

use crate::traits::{ToTick, FromTick};
use crate::tick_rate_hz;
use super::types::TickType;

/// Converts a `Duration` to FreeRTOS ticks.
///
/// # Examples
///
/// ```ignore
/// use core::time::Duration;
/// use osal_rs::os::ToTick;
/// 
/// let duration = Duration::from_millis(100);
/// let ticks = duration.to_ticks();  // Converts to FreeRTOS ticks
/// ```
///
/// # Notes
///
/// - Saturates at maximum value on overflow
/// - Conversion is based on `configTICK_RATE_HZ` from FreeRTOS configuration
impl ToTick for Duration {
    fn to_ticks(&self) -> TickType {
        let millis = self.as_millis() as TickType;
        
        // Check for potential overflow and saturate at max value
        millis.saturating_mul(tick_rate_hz!() as TickType) / 1000
    }
}

/// Converts FreeRTOS ticks to a `Duration`.
///
/// # Examples
///
/// ```ignore
/// use core::time::Duration;
/// use osal_rs::os::FromTick;
/// 
/// let mut duration = Duration::from_secs(0);
/// duration.ticks(100);  // Set duration from 100 ticks
/// ```
///
/// # Notes
///
/// - Conversion is based on `configTICK_RATE_HZ` from FreeRTOS configuration
/// - Saturates at maximum value on overflow
impl FromTick for Duration {
    fn ticks(&mut self, tick: TickType) {
        let millis = tick.saturating_mul(1000) / tick_rate_hz!() as TickType;
        *self = Duration::from_millis(millis as u64);
    }
}