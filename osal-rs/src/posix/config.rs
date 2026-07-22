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

//! POSIX configuration constants.
//!
//! Unlike the `freertos` backend - where the tick period is fixed by
//! `configTICK_RATE_HZ` at kernel-build time - the POSIX backend has no real
//! tick interrupt at all. It only ever uses a 1ms tick, so the single
//! constant here just documents that fact for the rest of the module (see
//! `crate::posix::duration`, which is what actually performs the
//! conversion).

/// Length, in milliseconds, of one OSAL-RS "tick" on the POSIX backend.
///
/// Fixed at `1`, which makes ticks and milliseconds interchangeable
/// throughout this backend: converting a [`core::time::Duration`] to ticks
/// (via [`crate::os::ToTick`]) or back (via [`crate::os::FromTick`]) is just
/// a millisecond count, with no scaling involved.
///
/// # Examples
///
/// ```
/// use osal_rs::os::config::TICK_PERIOD_MS;
/// use osal_rs::os::ToTick;
/// use core::time::Duration;
///
/// assert_eq!(TICK_PERIOD_MS, 1);
///
/// // With a 1ms tick period, ticks and milliseconds are the same number.
/// let ticks = Duration::from_millis(250).to_ticks();
/// assert_eq!(ticks, 250);
/// ```
pub const TICK_PERIOD_MS: u64 = 1;