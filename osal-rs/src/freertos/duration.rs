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
    #[inline]
    fn to_ticks(&self) -> TickType {
        // Do the arithmetic in u128 - the type `as_millis` already returns -
        // and only narrow at the very end. Truncating to `TickType` up front
        // would wrap a long duration into a short one, and saturating before
        // the division would be undone by it: `MAX_DELAY` used to saturate to
        // `TickType::MAX` and then divide down to `TickType::MAX / 1000`,
        // losing the "wait forever" sentinel.
        let ticks = self.as_millis() * tick_rate_hz!() as u128 / 1000;

        if ticks > TickType::MAX as u128 {
            TickType::MAX
        } else {
            ticks as TickType
        }
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
    #[inline]
    fn ticks(&mut self, tick: TickType) {
        // Widen before multiplying: `saturating_mul` on `TickType` would clamp
        // a large tick count to `TickType::MAX` and the division would then
        // silently scale that clamp down, so a long delay came back short.
        let millis = tick as u64 * 1000 / tick_rate_hz!() as u64;
        *self = Duration::from_millis(millis);
    }
}