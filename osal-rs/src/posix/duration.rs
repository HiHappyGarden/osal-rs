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

use core::time::Duration;

use crate::posix::config::TICK_PERIOD_MS;
use crate::posix::types::TickType;
use crate::traits::{FromTick, ToTick};

impl ToTick for Duration {
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

impl FromTick for Duration {
    fn ticks(&mut self, tick: TickType) {
        *self = Duration::from_millis(tick.saturating_mul(TICK_PERIOD_MS as TickType));
    }
}