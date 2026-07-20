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
	pthread_condattr_init, pthread_condattr_setclock, pthread_condattr_t, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock, pthread_mutexattr_init,
	pthread_mutexattr_setprotocol, pthread_mutexattr_t, timespec,
};
use crate::posix::types::{SemaphoreHandle, TickType, UBaseType};
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
			pthread_condattr_init(&mut cond_attr);
			pthread_condattr_setclock (&mut cond_attr, CLOCK_MONOTONIC);
			pthread_cond_init (&mut cond, &cond_attr);
			pthread_mutexattr_init (&mut mutex_attr);
   			pthread_mutexattr_setprotocol (&mut mutex_attr, PTHREAD_PRIO_INHERIT);
   			pthread_mutex_init (&mut mutex, &mutex_attr);

		}

		Ok(Self(UnsafeCell::new(SemaphoreHandle(mutex, cond)), UnsafeCell::new(initial_count), max_count))
	}

	fn mutex_ptr(&self) -> *mut pthread_mutex_t {
		unsafe { &raw mut (*self.0.get()).0 }
	}

	fn cond_ptr(&self) -> *mut pthread_cond_t {
		unsafe { &raw mut (*self.0.get()).1 }
	}

	fn count_ptr(&self) -> *mut UBaseType {
		self.1.get()
	}
}

impl SemaphoreFn for Semaphore {

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
			loop {
				if unsafe { *self.count_ptr() } > 0 {
					break true;
				}
				unsafe {
					pthread_cond_wait(self.cond_ptr(), self.mutex_ptr());
				}
			}
		} else {
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

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
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

		let signalled = unsafe {
			if *self.count_ptr() < self.2 {
				*self.count_ptr() += 1;
				true
			} else {
				false
			}
		};

		if signalled {
			unsafe {
				pthread_cond_broadcast(self.cond_ptr());
			}
		}

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if signalled { OsalRsBool::True } else { OsalRsBool::False }
	}

	fn signal_from_isr(&self) -> OsalRsBool {
		self.signal()
	}

	fn delete(&mut self) {
		if self.is_null() {
			return;
		}

		unsafe {
			pthread_mutex_destroy(self.mutex_ptr());
			pthread_cond_destroy(self.cond_ptr());
		}

		*self.0.get_mut() = SemaphoreHandle::default();
		*self.1.get_mut() = 0;
	}
}

impl Drop for Semaphore {
	fn drop(&mut self) {
		if self.is_null() {
			return;
		}
		self.delete();
	}
}

impl Deref for Semaphore {
	type Target = SemaphoreHandle;

	fn deref(&self) -> &Self::Target {
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
