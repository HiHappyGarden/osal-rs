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
use core::marker::PhantomData;
use core::ops::Deref;
use core::time::Duration;

use crate::os::types::ClockMonotonicHandle;
use crate::posix::config::TICK_PERIOD_MS;
use crate::posix::ffi::{
	CLOCK_MONOTONIC, ETIMEDOUT, PTHREAD_PRIO_INHERIT, clock_gettime, pthread_cond_broadcast, pthread_cond_destroy, pthread_cond_init, pthread_cond_t, pthread_cond_timedwait, pthread_cond_wait,
	pthread_condattr_init, pthread_condattr_setclock, pthread_condattr_t, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_trylock, pthread_mutex_unlock,
	pthread_mutexattr_init, pthread_mutexattr_setprotocol, pthread_mutexattr_t, timespec,
};
#[cfg(not(feature = "serde"))]
use crate::traits::{Deserialize, Serialize};
use crate::traits::{BytesHasLen, QueueFn, QueueStreamedFn, ToTick};
use crate::utils::{Error, Result};
use crate::posix::types::{QueueHandle, TickType, UBaseType};

#[cfg(feature = "serde")]
use osal_rs_serde::{Deserialize, Serialize, from_bytes, to_bytes};

pub trait StructSerde: Serialize + BytesHasLen + Deserialize {}

impl<T> StructSerde for T where T: Serialize + BytesHasLen + Deserialize {}

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

pub struct Queue{
	handle: UnsafeCell<QueueHandle>,
	r: UnsafeCell<usize>,
   	w: UnsafeCell<usize>,
   	count: UnsafeCell<usize>,
   	size: usize,
   	message_size: usize,
   	msg: UnsafeCell<Vec<u8>>

}

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

impl Queue {
	pub fn new(size: UBaseType, message_size: UBaseType) -> Result<Self> {
		if size == 0 || message_size == 0 {
			return Err(Error::InvalidQueueSize)
		}

		let size = size as usize;
		let message_size = message_size as usize;

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

		Ok(Self {
			handle: UnsafeCell::new(ClockMonotonicHandle(mutex, cond)),
			r: UnsafeCell::new(0),
			w: UnsafeCell::new(0),
			count: UnsafeCell::new(0),
			size,
			message_size,
			msg: UnsafeCell::new(vec![0u8; size * message_size])
		})
	}

	#[inline]
	pub fn fetch_with_to_tick(&self, buffer: &mut [u8], time: impl ToTick) -> Result<()> {
		self.fetch(buffer, time.to_ticks())
	}

	#[inline]
	pub fn post_with_to_tick(&self, item: &[u8], time: impl ToTick) -> Result<()> {
		self.post(item, time.to_ticks())
	}

	// Raw pointers into the `UnsafeCell`s, needed because the pthread FFI
	// takes `*mut`. Must only be dereferenced while holding `mutex_ptr()`
	// locked.
	fn mutex_ptr(&self) -> *mut pthread_mutex_t {
		unsafe { &raw mut (*self.handle.get()).0 }
	}

	fn cond_ptr(&self) -> *mut pthread_cond_t {
		unsafe { &raw mut (*self.handle.get()).1 }
	}
}

impl QueueFn for Queue {
	// "Null" means never-initialized-or-already-deleted: the pthread handle
	// is still/again at its zero `Default` state.
	fn is_null(&self) -> bool {
		unsafe { (*self.handle.get()).is_empty() }
	}

	fn fetch(&self, buffer: &mut [u8], time: TickType) -> Result<()> {
		if self.is_null() {
			return Err(Error::NullPtr);
		}

		if buffer.len() < self.message_size {
			return Err(Error::InvalidQueueSize);
		}

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
		}

		// `count > 0` is re-checked in a loop after every wake-up: both
		// `pthread_cond_wait`/`pthread_cond_timedwait` may return spuriously.
		let received = if time == TickType::MAX {
			loop {
				if unsafe { *self.count.get() } > 0 {
					break true;
				}
				unsafe {
					pthread_cond_wait(self.cond_ptr(), self.mutex_ptr());
				}
			}
		} else {
			let deadline = monotonic_deadline(Duration::from_millis((time as u64).saturating_mul(TICK_PERIOD_MS)));

			loop {
				if unsafe { *self.count.get() } > 0 {
					break true;
				}
				if unsafe { pthread_cond_timedwait(self.cond_ptr(), self.mutex_ptr(), &deadline) } == ETIMEDOUT {
					break false;
				}
			}
		};

		if received {
			unsafe {
				let r = *self.r.get();
				let offset = r * self.message_size;
				let msg = &*self.msg.get();
				buffer[..self.message_size].copy_from_slice(&msg[offset..offset + self.message_size]);

				*self.r.get() = (r + 1) % self.size;
				*self.count.get() -= 1;

				// Wakes any `post()`er blocked on a full queue: fetching
				// just freed a slot.
				pthread_cond_broadcast(self.cond_ptr());
			}
		}

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if received { Ok(()) } else { Err(Error::Timeout) }
	}

	fn fetch_from_isr(&self, buffer: &mut [u8]) -> Result<()> {
		if self.is_null() {
			return Err(Error::NullPtr);
		}

		if buffer.len() < self.message_size {
			return Err(Error::InvalidQueueSize);
		}

		// pthreads has no ISR context of its own; `trylock` keeps this
		// non-blocking, as `_from_isr` callers expect. If the mutex is
		// contended, bail out rather than blocking the "interrupt".
		if unsafe { pthread_mutex_trylock(self.mutex_ptr()) } != 0 {
			return Err(Error::QueueFull);
		}

		let received = unsafe { *self.count.get() } > 0;

		if received {
			unsafe {
				let r = *self.r.get();
				let offset = r * self.message_size;
				let msg = &*self.msg.get();
				buffer[..self.message_size].copy_from_slice(&msg[offset..offset + self.message_size]);

				*self.r.get() = (r + 1) % self.size;
				*self.count.get() -= 1;

				pthread_cond_broadcast(self.cond_ptr());
			}
		}

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if received { Ok(()) } else { Err(Error::Timeout) }
	}

	fn post(&self, item: &[u8], time: TickType) -> Result<()> {
		if self.is_null() {
			return Err(Error::NullPtr);
		}

		if item.len() < self.message_size {
			return Err(Error::InvalidQueueSize);
		}

		unsafe {
			pthread_mutex_lock(self.mutex_ptr());
		}

		let sent = if time == TickType::MAX {
			loop {
				if unsafe { *self.count.get() } < self.size {
					break true;
				}
				unsafe {
					pthread_cond_wait(self.cond_ptr(), self.mutex_ptr());
				}
			}
		} else {
			let deadline = monotonic_deadline(Duration::from_millis((time as u64).saturating_mul(TICK_PERIOD_MS)));

			loop {
				if unsafe { *self.count.get() } < self.size {
					break true;
				}
				if unsafe { pthread_cond_timedwait(self.cond_ptr(), self.mutex_ptr(), &deadline) } == ETIMEDOUT {
					break false;
				}
			}
		};

		if sent {
			unsafe {
				let w = *self.w.get();
				let offset = w * self.message_size;
				let msg = &mut *self.msg.get();
				msg[offset..offset + self.message_size].copy_from_slice(&item[..self.message_size]);

				*self.w.get() = (w + 1) % self.size;
				*self.count.get() += 1;

				// Wakes any `fetch()`er blocked on an empty queue: posting
				// just produced a message.
				pthread_cond_broadcast(self.cond_ptr());
			}
		}

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if sent { Ok(()) } else { Err(Error::Timeout) }
	}

	fn post_from_isr(&self, item: &[u8]) -> Result<()> {
		if self.is_null() {
			return Err(Error::NullPtr);
		}

		if item.len() < self.message_size {
			return Err(Error::InvalidQueueSize);
		}

		if unsafe { pthread_mutex_trylock(self.mutex_ptr()) } != 0 {
			return Err(Error::QueueFull);
		}

		let sent = unsafe { *self.count.get() } < self.size;

		if sent {
			unsafe {
				let w = *self.w.get();
				let offset = w * self.message_size;
				let msg = &mut *self.msg.get();
				msg[offset..offset + self.message_size].copy_from_slice(&item[..self.message_size]);

				*self.w.get() = (w + 1) % self.size;
				*self.count.get() += 1;

				pthread_cond_broadcast(self.cond_ptr());
			}
		}

		unsafe {
			pthread_mutex_unlock(self.mutex_ptr());
		}

		if sent { Ok(()) } else { Err(Error::QueueFull) }
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
		*self.handle.get_mut() = QueueHandle::default();
		*self.r.get_mut() = 0;
		*self.w.get_mut() = 0;
		*self.count.get_mut() = 0;
		self.msg.get_mut().clear();
	}
}

impl Drop for Queue {
	fn drop(&mut self) {
		if self.is_null() {
			return;
		}
		// Safety net for callers that don't call `delete()` explicitly.
		self.delete();
	}
}

impl Deref for Queue {
	type Target = QueueHandle;

	fn deref(&self) -> &Self::Target {
		// Read-only escape hatch to the raw (mutex, condvar) handle.
		unsafe { &*self.handle.get() }
	}
}

impl Debug for Queue {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Queue")
			.field("handle", unsafe { &*self.handle.get() })
			.field("count", unsafe { &*self.count.get() })
			.field("size", &self.size)
			.field("message_size", &self.message_size)
			.finish()
	}
}

impl Display for Queue {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(
			f,
			"Queue {{ handle: {:?}, count: {}, size: {}, message_size: {} }}",
			unsafe { &*self.handle.get() },
			unsafe { *self.count.get() },
			self.size,
			self.message_size
		)
	}
}

pub struct QueueStreamed<T: StructSerde>(Queue, PhantomData<T>);

unsafe impl<T: StructSerde> Send for QueueStreamed<T> {}
unsafe impl<T: StructSerde> Sync for QueueStreamed<T> {}

impl<T> QueueStreamed<T>
where
	T: StructSerde,
{
	#[inline]
	pub fn new(size: UBaseType, message_size: UBaseType) -> Result<Self> {
		Ok(Self(Queue::new(size, message_size)?, PhantomData))
	}

	#[allow(dead_code)]
	#[inline]
	fn fetch_with_to_tick(&self, buffer: &mut T, time: impl ToTick) -> Result<()> {
		self.fetch(buffer, time.to_ticks())
	}

	#[allow(dead_code)]
	#[inline]
	fn post_with_to_tick(&self, item: &T, time: impl ToTick) -> Result<()> {
		self.post(item, time.to_ticks())
	}
}

#[cfg(not(feature = "serde"))]
impl<T> QueueStreamedFn<T> for QueueStreamed<T>
where
	T: StructSerde,
{
	fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()> {
		let mut buf_bytes = vec![0u8; buffer.len()];

		self.0.fetch(&mut buf_bytes, time)?;
		*buffer = T::from_bytes(&buf_bytes)?;

		Ok(())
	}

	fn fetch_from_isr(&self, buffer: &mut T) -> Result<()> {
		let mut buf_bytes = vec![0u8; buffer.len()];

		self.0.fetch_from_isr(&mut buf_bytes)?;
		*buffer = T::from_bytes(&buf_bytes)?;

		Ok(())
	}

	#[inline]
	fn post(&self, item: &T, time: TickType) -> Result<()> {
		self.0.post(&item.to_bytes(), time)
	}

	#[inline]
	fn post_from_isr(&self, item: &T) -> Result<()> {
		self.0.post_from_isr(&item.to_bytes())
	}

	#[inline]
	fn delete(&mut self) {
		self.0.delete()
	}
}

#[cfg(feature = "serde")]
impl<T> QueueStreamedFn<T> for QueueStreamed<T>
where
	T: StructSerde,
{
	fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()> {
		let mut buf_bytes = vec![0u8; buffer.len()];

		self.0.fetch(&mut buf_bytes, time)?;
		*buffer = from_bytes(&buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

		Ok(())
	}

	fn fetch_from_isr(&self, buffer: &mut T) -> Result<()> {
		let mut buf_bytes = vec![0u8; buffer.len()];

		self.0.fetch_from_isr(&mut buf_bytes)?;
		*buffer = from_bytes(&buf_bytes).map_err(|_| Error::Unhandled("Deserializiation error"))?;

		Ok(())
	}

	fn post(&self, item: &T, time: TickType) -> Result<()> {
		let mut buf_bytes = vec![0u8; item.len()];

		to_bytes(item, &mut buf_bytes).map_err(|_| Error::Unhandled("Serialization error"))?;

		self.0.post(&buf_bytes, time)
	}

	fn post_from_isr(&self, item: &T) -> Result<()> {
		let mut buf_bytes = vec![0u8; item.len()];

		to_bytes(item, &mut buf_bytes).map_err(|_| Error::Unhandled("Serialization error"))?;

		self.0.post_from_isr(&buf_bytes)
	}

	#[inline]
	fn delete(&mut self) {
		self.0.delete()
	}
}

impl<T> Deref for QueueStreamed<T>
where
	T: StructSerde,
{
	type Target = QueueHandle;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.0.handle.get() }
	}
}

impl<T> Debug for QueueStreamed<T>
where
	T: StructSerde,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("QueueStreamed")
			.field("handle", unsafe { &*self.0.handle.get() })
			.finish()
	}
}

impl<T> Display for QueueStreamed<T>
where
	T: StructSerde,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "QueueStreamed {{ handle: {:?} }}", unsafe { &*self.0.handle.get() })
	}
}
