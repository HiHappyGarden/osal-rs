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

use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null;

use crate::posix::types::{EventBits, EventGroupHandle, TickType};
use crate::traits::{EventGroupFn, ToTick};
use crate::utils::Result;

pub struct EventGroup(EventGroupHandle);

unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}

impl EventGroup {
	pub const MAX_MASK: EventBits = EventBits::MAX >> 8;

	pub fn wait_with_to_tick(&self, mask: EventBits, timeout_ticks: impl ToTick) -> EventBits {
		self.wait(mask, timeout_ticks.to_ticks())
	}

	pub fn new() -> Result<Self> {
		Ok(Self(null()))
	}
}

impl EventGroupFn for EventGroup {
	fn set(&self, _bits: EventBits) -> EventBits {
		0
	}

	fn set_from_isr(&self, _bits: EventBits) -> Result<()> {
		Ok(())
	}

	fn get(&self) -> EventBits {
		0
	}

	fn get_from_isr(&self) -> EventBits {
		0
	}

	fn clear(&self, _bits: EventBits) -> EventBits {
		0
	}

	fn clear_from_isr(&self, _bits: EventBits) -> Result<()> {
		Ok(())
	}

	fn wait(&self, _mask: EventBits, _timeout_ticks: TickType) -> EventBits {
		0
	}

	fn delete(&mut self) {
		self.0 = null();
	}
}

impl Drop for EventGroup {
	fn drop(&mut self) {
		self.0 = null();
	}
}

impl Deref for EventGroup {
	type Target = EventGroupHandle;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Debug for EventGroup {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "EventGroup {{ handle: {:?} }}", self.0)
	}
}

impl Display for EventGroup {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "EventGroup {{ handle: {:?} }}", self.0)
	}
}