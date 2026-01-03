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

use core::time::Duration;

use crate::traits::{ToTick, FromTick};
use crate::tick_rate_hz;
use super::types::TickType;

impl ToTick for Duration {
    fn to_ticks(&self) -> TickType {
        let millis = self.as_millis() as TickType;
        
        // Check for potential overflow and saturate at max value
        millis.saturating_mul(tick_rate_hz!() as TickType) / 1000
    }
}

impl FromTick for Duration {
    fn ticks(&mut self, tick: TickType) {
        let millis = tick.saturating_mul(1000) / tick_rate_hz!() as TickType;
        *self = Duration::from_millis(millis as u64);
    }
}