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
use core::fmt::{Debug, Display};
use core::ops::Deref;
use core::time::Duration;

use crate::posix::config::TICK_PERIOD_MS;
use crate::posix::ffi::{
	CLOCK_MONOTONIC, ETIMEDOUT, PTHREAD_PRIO_INHERIT, clock_gettime, pthread_cond_broadcast, pthread_cond_destroy, pthread_cond_init, pthread_cond_t, pthread_cond_timedwait, pthread_cond_wait,
	pthread_condattr_init, pthread_condattr_setclock, pthread_condattr_t, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_trylock, pthread_mutex_unlock, pthread_mutexattr_init,
	pthread_mutexattr_setprotocol, pthread_mutexattr_t, timespec,
};
use crate::posix::types::{ClockMonotonicHandle, SemaphoreHandle, TickType, UBaseType};
use crate::traits::{SemaphoreFn, ToTick};
use crate::utils::{OsalRsBool, Result};

/// Computes an absolute deadline `timeout` from now on the monotonic clock,
/// for `pthread_cond_timedwait` (this module's condition variables are all
/// created with `pthread_condattr_setclock(CLOCK_MONOTONIC)`, so its
/// `abstime` is measured against that same clock).
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

/// POSIX backend for [`SemaphoreFn`]. Built directly on a `pthread_mutex_t` +
/// `pthread_cond_t` pair rather than `sem_t`, so `signal()` can be bounded by
/// a user-supplied `max_count` and the mutex can use priority inheritance —
/// neither of which plain POSIX unnamed semaphores support.
///
/// Fields (unnamed, accessed as `self.0`/`self.1`/`self.2`): the pthread
/// handle, the current count, and the fixed maximum count.
pub struct Semaphore(UnsafeCell<SemaphoreHandle>, UnsafeCell<UBaseType>, UBaseType);

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
	pub fn new(max_count: UBaseType, initial_count: UBaseType) -> Result<Self> {

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

		Ok(Self(UnsafeCell::new(ClockMonotonicHandle(mutex, cond)), UnsafeCell::new(initial_count), max_count))
	}

	// Raw pointers into the `UnsafeCell`s, needed because the pthread FFI
	// takes `*mut`. `count_ptr()` must only be dereferenced while holding
	// `mutex_ptr()` locked, except for the racy peek in `is_null()`.
	fn mutex_ptr(&self) -> *mut pthread_mutex_t {
		unsafe { &raw mut (*self.0.get()).0 }
	}

	fn cond_ptr(&self) -> *mut pthread_cond_t {
		unsafe { &raw mut (*self.0.get()).1 }
	}

	fn count_ptr(&self) -> *mut UBaseType {
		self.1.get()
	}

	// Assumes `mutex_ptr()` is already locked (by the caller, either via
	// blocking `lock` or non-blocking `trylock`) and always unlocks it
	// before returning. Increments the count if below `max_count` and, if
	// so, broadcasts to wake any `wait()`ers.
	fn signal_locked(&self) -> OsalRsBool {
		let signalled = unsafe {
			if *self.count_ptr() < self.2 {
				*self.count_ptr() += 1;
				true
			} else {
				false
			}
		};

		if signalled {
			// Broadcast, not signal: any thread parked in `wait()`'s loop
			// could be the one to claim this unit, so all of them are woken
			// to re-check the count under the mutex; losers just go back to
			// waiting instead of missing the wake-up.
			unsafe {
				pthread_cond_broadcast(self.cond_ptr());
			}
		}

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if signalled { OsalRsBool::True } else { OsalRsBool::False }
	}
}

impl SemaphoreFn for Semaphore {

	// "Null" means never-initialized-or-already-deleted: both the pthread
	// handle and the count are still/again at their zero `Default` state.
	// Checking the count too avoids treating a live semaphore that just
	// happens to be momentarily empty (count == 0) as deleted.
	fn is_null(&self) -> bool {
		unsafe { (*self.0.get()).is_empty() && *self.1.get() == 0 }
	}

	fn wait(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
		if self.is_null() {
			return OsalRsBool::False
		}

		let ticks = ticks_to_wait.to_ticks();

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
		}

		// `count > 0` is re-checked in a loop after every wake-up: both
		// `pthread_cond_wait`/`pthread_cond_timedwait` may return spuriously,
		// and a woken thread isn't guaranteed to be the one that gets the
		// unit of the semaphore another thread's `wait()` grabbed first.
		let acquired = if ticks == TickType::MAX {
			// TickType::MAX is the "wait forever" sentinel: no deadline,
			// block until signalled.
			loop {
				if unsafe { *self.count_ptr() } > 0 {
					break true;
				}
				unsafe {
					pthread_cond_wait(self.cond_ptr(), self.mutex_ptr());
				}
			}
		} else {
			// Bounded wait: the deadline is computed once up front, then
			// every re-wake races against that same fixed point in time
			// (rather than restarting a fresh relative timeout each loop).
			let deadline = monotonic_deadline(Duration::from_millis((ticks as u64).saturating_mul(TICK_PERIOD_MS)));

			loop {
				if unsafe { *self.count_ptr() } > 0 {
					break true;
				}
				if unsafe { pthread_cond_timedwait(self.cond_ptr(), self.mutex_ptr(), &deadline) } == ETIMEDOUT {
					break false;
				}
			}
		};

		if acquired {
			unsafe {
				*self.count_ptr() -= 1;
			}
		}

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if acquired { OsalRsBool::True } else { OsalRsBool::False }
	}

	fn wait_from_isr(&self) -> OsalRsBool {
		if self.is_null() {
			return OsalRsBool::False;
		}

		// pthreads has no ISR context of its own; `trylock` keeps this
		// non-blocking, as `_from_isr` callers expect (mirrors
		// `RawMutex::lock_from_isr`). If the mutex is contended, bail out
		// rather than blocking the "interrupt".
		if unsafe { pthread_mutex_trylock(self.mutex_ptr()) } != 0 {
			return OsalRsBool::False;
		}

		let acquired = unsafe {
			if *self.count_ptr() > 0 {
				*self.count_ptr() -= 1;
				true
			} else {
				false
			}
		};

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if acquired { OsalRsBool::True } else { OsalRsBool::False }
	}

	fn signal(&self) -> OsalRsBool {
		if self.is_null() {
			return OsalRsBool::False;
		}

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
		}

		self.signal_locked()
	}

	fn signal_from_isr(&self) -> OsalRsBool {
		if self.is_null() {
			return OsalRsBool::False;
		}

		// Same non-blocking rationale as `wait_from_isr`: `trylock` instead
		// of `lock` so this never blocks the "interrupt".
		if unsafe { pthread_mutex_trylock(self.mutex_ptr()) } != 0 {
			return OsalRsBool::False;
		}

		self.signal_locked()
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
		*self.0.get_mut() = SemaphoreHandle::default();
		*self.1.get_mut() = 0;
	}
}

impl Drop for Semaphore {
	fn drop(&mut self) {
		if self.is_null() {
			return;
		}
		// Safety net for callers that don't call `delete()` explicitly.
		self.delete();
	}
}

impl Deref for Semaphore {
	type Target = SemaphoreHandle;

	fn deref(&self) -> &Self::Target {
		// Read-only escape hatch to the raw (mutex, condvar) handle.
		unsafe { &*self.0.get() }
	}
}

impl Debug for Semaphore {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Semaphore")
			.field("handle", unsafe { &*self.0.get() })
			.field("count", unsafe { &*self.1.get() })
			.field("max_count", &self.2)
			.finish()
	}
}

impl Display for Semaphore {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "Semaphore {{ handle: {:?}, count: {}, max_count: {} }}", unsafe { &*self.0.get() }, unsafe { *self.1.get() }, self.2)
	}
}
