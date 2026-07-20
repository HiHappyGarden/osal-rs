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

use core::cell::UnsafeCell;
use core::ffi::c_long;
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::time::Duration;

use crate::posix::config::TICK_PERIOD_MS;
use crate::posix::ffi::{
	CLOCK_MONOTONIC, ETIMEDOUT, PTHREAD_PRIO_INHERIT, clock_gettime, pthread_cond_broadcast, pthread_cond_destroy, pthread_cond_init, pthread_cond_t, pthread_cond_timedwait, pthread_cond_wait,
	pthread_condattr_init, pthread_condattr_setclock, pthread_condattr_t, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_trylock, pthread_mutex_unlock,
	pthread_mutexattr_init, pthread_mutexattr_setprotocol, pthread_mutexattr_t, timespec,
};
use crate::posix::types::{ClockMonotonicHandle, EventBits, EventGroupHandle, TickType};
use crate::traits::{EventGroupFn, ToTick};
use crate::utils::{Error, Result};

/// Computes an absolute deadline `timeout` from now on the monotonic clock,
/// for `pthread_cond_timedwait` (this module's condition variable is created
/// with `pthread_condattr_setclock(CLOCK_MONOTONIC)`, so its `abstime` is
/// measured against that same clock).
fn monotonic_deadline(timeout: Duration) -> timespec {
	let mut now = timespec::default();
	unsafe {
		clock_gettime(CLOCK_MONOTONIC, &mut now);
	}

	let mut tv_sec = now.tv_sec + timeout.as_secs() as c_long;
	let mut tv_nsec = now.tv_nsec + timeout.subsec_nanos() as c_long;

	if tv_nsec >= 1_000_000_000 {
		tv_sec += 1;
		tv_nsec -= 1_000_000_000;
	}

	timespec { tv_sec, tv_nsec }
}

pub struct EventGroup(UnsafeCell<EventGroupHandle>, UnsafeCell<EventBits>);

unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}

impl EventGroup {
	pub const MAX_MASK: EventBits = EventBits::MAX >> 8;

	pub fn wait_with_to_tick(&self, mask: EventBits, timeout_ticks: impl ToTick) -> EventBits {
		self.wait(mask, timeout_ticks.to_ticks())
	}

	pub fn new() -> Result<Self> {

		let mut mutex: pthread_mutex_t = Default::default();
		let mut mutex_attr: pthread_mutexattr_t = Default::default();
		let mut cond: pthread_cond_t = Default::default();
		let mut cond_attr: pthread_condattr_t = Default::default();


		unsafe {
			// Bind the condvar to CLOCK_MONOTONIC so its absolute timeouts line
			// up with the clock `monotonic_deadline` uses to build them.
			pthread_condattr_init(&mut cond_attr);
			pthread_condattr_setclock (&mut cond_attr, CLOCK_MONOTONIC);
			pthread_cond_init (&mut cond, &cond_attr);
			// Priority inheritance: a low-priority holder that blocks a
			// higher-priority waiter gets temporarily boosted, avoiding
			// priority inversion (same protocol as posix::mutex::RawMutex).
			pthread_mutexattr_init (&mut mutex_attr);
   			pthread_mutexattr_setprotocol (&mut mutex_attr, PTHREAD_PRIO_INHERIT);
   			pthread_mutex_init (&mut mutex, &mutex_attr);

		}

		Ok(Self(UnsafeCell::new(ClockMonotonicHandle(mutex, cond)), UnsafeCell::new(0)))
	}

	// "Null" means never-initialized-or-already-deleted. Unlike
	// `Semaphore::is_null`, the bits themselves are not part of this check:
	// an event group legitimately sits at `0` bits whenever nothing has been
	// set yet, so that can't be used to detect deletion.
	fn is_null(&self) -> bool {
		unsafe { (*self.0.get()).is_empty() }
	}

	// Raw pointers into the `UnsafeCell`s, needed because the pthread FFI
	// takes `*mut`. `bits_ptr()` must only be dereferenced while holding
	// `mutex_ptr()` locked, except for the racy peek in `get_from_isr()`.
	fn mutex_ptr(&self) -> *mut pthread_mutex_t {
		unsafe { &raw mut (*self.0.get()).0 }
	}

	fn cond_ptr(&self) -> *mut pthread_cond_t {
		unsafe { &raw mut (*self.0.get()).1 }
	}

	fn bits_ptr(&self) -> *mut EventBits {
		self.1.get()
	}
}

impl EventGroupFn for EventGroup {
	fn set(&self, bits: EventBits) -> EventBits {
		if self.is_null() {
			return 0;
		}

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
		}

		let new_bits = unsafe {
			*self.bits_ptr() |= bits;
			*self.bits_ptr()
		};

		unsafe {
			// Broadcast, not signal: any thread parked in `wait()`'s loop
			// could be the one whose mask is now satisfied, so all of them
			// are woken to re-check under the mutex; losers just go back to
			// waiting instead of missing the wake-up.
			pthread_cond_broadcast(self.cond_ptr());
			pthread_mutex_unlock(self.mutex_ptr());
		}

		new_bits
	}

	fn set_from_isr(&self, bits: EventBits) -> Result<()> {
		if self.is_null() {
			return Err(Error::NullPtr);
		}

		// pthreads has no ISR context of its own; `trylock` keeps this
		// non-blocking, as `_from_isr` callers expect (mirrors
		// `Semaphore::signal_from_isr`). If the mutex is contended, bail out
		// rather than blocking the "interrupt".
		if unsafe { pthread_mutex_trylock(self.mutex_ptr()) } != 0 {
			return Err(Error::QueueFull);
		}

		unsafe {
			*self.bits_ptr() |= bits;
			pthread_cond_broadcast(self.cond_ptr());
			pthread_mutex_unlock(self.mutex_ptr());
		}

		Ok(())
	}

	fn get(&self) -> EventBits {
		if self.is_null() {
			return 0;
		}

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
			let bits = *self.bits_ptr();
			pthread_mutex_unlock(self.mutex_ptr());
			bits
		}
	}

	fn get_from_isr(&self) -> EventBits {
		if self.is_null() {
			return 0;
		}

		if unsafe { pthread_mutex_trylock(self.mutex_ptr()) } != 0 {
			// Contended: fall back to a racy read rather than blocking the
			// "interrupt".
			return unsafe { *self.bits_ptr() };
		}

		unsafe {
			let bits = *self.bits_ptr();
			pthread_mutex_unlock(self.mutex_ptr());
			bits
		}
	}

	fn clear(&self, bits: EventBits) -> EventBits {
		if self.is_null() {
			return 0;
		}

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
		}

		let previous_bits = unsafe {
			let previous = *self.bits_ptr();
			*self.bits_ptr() &= !bits;
			previous
		};

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		previous_bits
	}

	fn clear_from_isr(&self, bits: EventBits) -> Result<()> {
		if self.is_null() {
			return Err(Error::NullPtr);
		}

		if unsafe { pthread_mutex_trylock(self.mutex_ptr()) } != 0 {
			return Err(Error::QueueFull);
		}

		unsafe {
			*self.bits_ptr() &= !bits;
			pthread_mutex_unlock(self.mutex_ptr());
		}

		Ok(())
	}

	fn wait(&self, mask: EventBits, timeout_ticks: TickType) -> EventBits {
		if self.is_null() {
			return 0;
		}

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
		}

		// The mask is re-checked in a loop after every wake-up: both
		// `pthread_cond_wait`/`pthread_cond_timedwait` may return spuriously,
		// and a wake caused by an unrelated `set()` may not satisfy this
		// waiter's mask yet.
		let result = if timeout_ticks == TickType::MAX {
			// TickType::MAX is the "wait forever" sentinel: no deadline,
			// block until every bit in `mask` is set.
			loop {
				let bits = unsafe { *self.bits_ptr() };
				if bits & mask == mask {
					break bits;
				}
				unsafe {
					pthread_cond_wait(self.cond_ptr(), self.mutex_ptr());
				}
			}
		} else {
			// Bounded wait: the deadline is computed once up front, then
			// every re-wake races against that same fixed point in time
			// (rather than restarting a fresh relative timeout each loop).
			let deadline = monotonic_deadline(Duration::from_millis((timeout_ticks as u64).saturating_mul(TICK_PERIOD_MS)));

			loop {
				let bits = unsafe { *self.bits_ptr() };
				if bits & mask == mask {
					break bits;
				}
				if unsafe { pthread_cond_timedwait(self.cond_ptr(), self.mutex_ptr(), &deadline) } == ETIMEDOUT {
					break unsafe { *self.bits_ptr() };
				}
			}
		};

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		result
	}

	fn delete(&mut self) {
		if self.is_null() {
			return;
		}

		unsafe {
			pthread_mutex_destroy(self.mutex_ptr());
			pthread_cond_destroy(self.cond_ptr());
		}

		// Reset to the "null" state so a second `delete()` call (e.g. from
		// `Drop` after an explicit `delete()`) is a no-op rather than
		// destroying the same pthread objects twice.
		*self.0.get_mut() = EventGroupHandle::default();
		*self.1.get_mut() = 0;
	}
}

impl Drop for EventGroup {
	fn drop(&mut self) {
		if self.is_null() {
			return;
		}
		// Safety net for callers that don't call `delete()` explicitly.
		self.delete();
	}
}

impl Deref for EventGroup {
	type Target = EventGroupHandle;

	fn deref(&self) -> &Self::Target {
		// Read-only escape hatch to the raw (mutex, condvar) handle.
		unsafe { &*self.0.get() }
	}
}

impl Debug for EventGroup {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("EventGroup")
			.field("handle", unsafe { &*self.0.get() })
			.field("bits", unsafe { &*self.1.get() })
			.finish()
	}
}

impl Display for EventGroup {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "EventGroup {{ handle: {:?}, bits: {:#X} }}", unsafe { &*self.0.get() }, unsafe { *self.1.get() })
	}
}
