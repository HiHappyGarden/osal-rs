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

//! [`Duration`] ⇄ tick conversions for the POSIX backend.
//!
//! The POSIX backend has no periodic tick interrupt the way FreeRTOS does;
//! it just treats one tick as one millisecond (see
//! [`crate::posix::config::TICK_PERIOD_MS`]). This module supplies the
//! [`ToTick`]/[`FromTick`] implementations for [`Duration`] that the rest of
//! the crate's `_with_to_tick` convenience methods (e.g.
//! [`crate::os::Semaphore::wait`], [`crate::os::System::delay_with_to_tick`])
//! rely on to accept a `Duration` wherever a raw tick count is expected.

use core::time::Duration;

use crate::posix::config::TICK_PERIOD_MS;
use crate::posix::types::TickType;
use crate::traits::{FromTick, ToTick};

/// Converts a [`Duration`] to POSIX ticks (milliseconds).
///
/// Any sub-millisecond remainder is discarded, matching [`Duration::as_millis`].
///
/// # Examples
///
/// ```
/// use osal_rs::os::ToTick;
/// use core::time::Duration;
///
/// assert_eq!(Duration::from_millis(250).to_ticks(), 250);
/// // Sub-millisecond precision is discarded, not rounded up.
/// assert_eq!(Duration::from_micros(1500).to_ticks(), 1);
/// ```
impl ToTick for Duration {
    #[inline]
    fn to_ticks(&self) -> TickType {
        let millis = self.as_millis() as TickType;
        let period = TICK_PERIOD_MS as TickType;

        if period == 0 {
            TickType::MAX
        } else {
            millis / period
        }
    }
}

/// Builds a [`Duration`] from a POSIX tick (millisecond) count.
///
/// # Examples
///
/// ```
/// use osal_rs::os::FromTick;
/// use core::time::Duration;
///
/// let mut d = Duration::ZERO;
/// d.ticks(500);
/// assert_eq!(d, Duration::from_millis(500));
/// ```
impl FromTick for Duration {
    #[inline]
    fn ticks(&mut self, tick: TickType) {
        *self = Duration::from_millis(tick.saturating_mul(TICK_PERIOD_MS as TickType));
    }
}