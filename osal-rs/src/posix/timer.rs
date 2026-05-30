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

use core::fmt::{Debug, Display};
use core::ops::Deref;
use core::ptr::null;

use alloc::string::{String, ToString};
use alloc::sync::Arc;

use crate::posix::types::{TickType, TimerHandle};
use crate::traits::{TimerFn, TimerFnPtr, TimerParam, ToTick};
use crate::utils::{OsalRsBool, Result};

fn dummy_timer_handle() -> TimerHandle {
	1usize as TimerHandle
}

#[derive(Clone)]
pub struct Timer {
	pub handle: TimerHandle,
	name: String,
	callback: Option<Arc<TimerFnPtr>>,
	param: Option<TimerParam>,
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl Timer {
	#[inline]
	pub fn new_with_to_tick<F>(name: &str, timer_period_in_ticks: impl ToTick, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
	where
		F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static,
	{
		Self::new(name, timer_period_in_ticks.to_ticks(), auto_reload, param, callback)
	}

	#[inline]
	pub fn start_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
		self.start(ticks_to_wait.to_ticks())
	}

	#[inline]
	pub fn stop_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
		self.stop(ticks_to_wait.to_ticks())
	}

	#[inline]
	pub fn reset_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
		self.reset(ticks_to_wait.to_ticks())
	}

	#[inline]
	pub fn change_period_with_to_tick(&self, new_period_in_ticks: impl ToTick, new_period_ticks: impl ToTick) -> OsalRsBool {
		self.change_period(new_period_in_ticks.to_ticks(), new_period_ticks.to_ticks())
	}

	#[inline]
	pub fn delete_with_to_tick(&mut self, ticks_to_wait: impl ToTick) -> OsalRsBool {
		self.delete(ticks_to_wait.to_ticks())
	}

	pub fn new<F>(name: &str, _timer_period_in_ticks: TickType, _auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
	where
		F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static,
	{
		Ok(Self {
			handle: dummy_timer_handle(),
			name: name.to_string(),
			callback: Some(Arc::new(callback)),
			param,
		})
	}
}

impl TimerFn for Timer {
	fn start(&self, _ticks_to_wait: TickType) -> OsalRsBool {
		OsalRsBool::True
	}

	fn stop(&self, _ticks_to_wait: TickType) -> OsalRsBool {
		OsalRsBool::True
	}

	fn reset(&self, _ticks_to_wait: TickType) -> OsalRsBool {
		OsalRsBool::True
	}

	fn change_period(&self, _new_period_in_ticks: TickType, _new_period_ticks: TickType) -> OsalRsBool {
		OsalRsBool::True
	}

	fn delete(&mut self, _ticks_to_wait: TickType) -> OsalRsBool {
		self.handle = null();
		OsalRsBool::True
	}
}

impl Drop for Timer {
	fn drop(&mut self) {
		self.handle = null();
	}
}

impl Deref for Timer {
	type Target = TimerHandle;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}

impl Debug for Timer {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Timer")
			.field("handle", &self.handle)
			.field("name", &self.name)
			.field("has_callback", &self.callback.is_some())
			.field("has_param", &self.param.is_some())
			.finish()
	}
}

impl Display for Timer {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "Timer {{ name: {}, handle: {:?} }}", self.name, self.handle)
	}
}