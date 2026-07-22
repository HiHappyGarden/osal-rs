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

//! Mutex implementations for POSIX, with priority inheritance.
//!
//! [`Mutex<T>`] is the data-guarding, RAII-style mutex most application code
//! should use (see the example below). [`RawMutex`] is the lower-level
//! primitive it's built on - a bare, lock/unlock pair with no data attached
//! and no guard - for callers that need to manage locking by hand (e.g. to
//! pair a mutex with a condition variable, as [`crate::posix::thread`] does).
//!
//! Both are backed by a `pthread_mutex_t` configured with
//! `PTHREAD_PRIO_INHERIT` (a low-priority holder blocking a higher-priority
//! waiter is temporarily boosted, avoiding priority inversion) and
//! `PTHREAD_MUTEX_RECURSIVE` (the owning thread may lock it again without
//! deadlocking, as long as it unlocks the same number of times).
//!
//! # Examples
//!
//! ```
//! use osal_rs::os::*;
//!
//! let mutex = Mutex::new(0);
//! {
//!     let mut guard = mutex.lock().unwrap();
//!     *guard += 1;
//! } // Lock released here
//!
//! assert_eq!(*mutex.lock().unwrap(), 1);
//! ```

use core::cell::UnsafeCell;
use core::fmt::{Debug, Display, Formatter};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use alloc::sync::Arc;

use crate::posix::ffi::{
	PTHREAD_MUTEX_RECURSIVE, PTHREAD_PRIO_INHERIT, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_trylock, pthread_mutex_unlock,
	pthread_mutexattr_init, pthread_mutexattr_setprotocol, pthread_mutexattr_settype, pthread_mutexattr_t,
};
use crate::posix::types::MutexHandle;
use crate::traits::{MutexFn, MutexGuardFn, RawMutexFn};
use crate::utils::{Error, OsalRsBool, Result};

/// Low-level POSIX mutex: lock/unlock only, no guarded data and no RAII
/// guard. Most application code should use [`Mutex<T>`] instead; `RawMutex`
/// is for callers that need to manage the lock/unlock pairing themselves.
pub struct RawMutex(UnsafeCell<MutexHandle>);

unsafe impl Send for RawMutex {}
unsafe impl Sync for RawMutex {}

impl RawMutex {
	/// Creates a new, unlocked mutex with priority inheritance enabled.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	/// use osal_rs::utils::OsalRsBool;
	///
	/// let mutex = RawMutex::new().unwrap();
	/// assert_eq!(mutex.lock(), OsalRsBool::True);
	/// assert_eq!(mutex.unlock(), OsalRsBool::True);
	/// ```
	pub fn new() -> Result<Self> {
		let mut mutex_attr: pthread_mutexattr_t = Default::default();
		let mut mutex: MutexHandle = Default::default();

		unsafe {
			pthread_mutexattr_init(&mut mutex_attr);
			pthread_mutexattr_setprotocol(&mut mutex_attr, PTHREAD_PRIO_INHERIT);
			pthread_mutexattr_settype(&mut mutex_attr, PTHREAD_MUTEX_RECURSIVE);
		}

		let result = unsafe { pthread_mutex_init(&mut mutex, &mutex_attr) };

		if result != 0 {
			return Err(Error::ReturnWithCode(result));
		}

		Ok(Self(UnsafeCell::new(mutex)))
	}
}

impl RawMutexFn for RawMutex {

	/// Returns `true` if this mutex is never-initialized-or-already-deleted.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mut mutex = RawMutex::new().unwrap();
	/// assert!(!mutex.is_null());
	///
	/// mutex.delete();
	/// assert!(mutex.is_null());
	/// ```
	fn is_null(&self) -> bool {
		unsafe { (*self.0.get()).is_empty() }
	}

	/// Locks the mutex, blocking the calling thread until it becomes
	/// available. Returns [`OsalRsBool::True`] on success.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	/// use osal_rs::utils::OsalRsBool;
	///
	/// let mutex = RawMutex::new().unwrap();
	/// assert_eq!(mutex.lock(), OsalRsBool::True);
	/// mutex.unlock();
	/// ```
	fn lock(&self) -> OsalRsBool {
		if self.is_null() {
			return OsalRsBool::False;
		}

		match unsafe { pthread_mutex_lock(self.0.get()) } {
			0 => OsalRsBool::True,
			_ => OsalRsBool::False,
		}
	}

	/// ISR-safe variant of [`RawMutex::lock`]. POSIX has no interrupt
	/// context of its own, so this never blocks (`trylock` instead of
	/// `lock`); it returns [`OsalRsBool::False`] if the mutex is already
	/// held rather than waiting for it.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	/// use osal_rs::utils::OsalRsBool;
	///
	/// let mutex = RawMutex::new().unwrap();
	/// assert_eq!(mutex.lock_from_isr(), OsalRsBool::True);
	/// mutex.unlock_from_isr();
	/// ```
	fn lock_from_isr(&self) -> OsalRsBool {
		if self.is_null() {
			return OsalRsBool::False;
		}

		// pthreads has no ISR context/API of its own; `trylock` gives the
		// non-blocking behavior `lock_from_isr` callers expect instead.
		match unsafe { pthread_mutex_trylock(self.0.get()) } {
			0 => OsalRsBool::True,
			_ => OsalRsBool::False,
		}
	}

	/// Unlocks the mutex. Must be called by the thread that currently holds
	/// the lock.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	/// use osal_rs::utils::OsalRsBool;
	///
	/// let mutex = RawMutex::new().unwrap();
	/// mutex.lock();
	/// assert_eq!(mutex.unlock(), OsalRsBool::True);
	/// ```
	fn unlock(&self) -> OsalRsBool {
		if self.is_null() {
			return OsalRsBool::False;
		}

		match unsafe { pthread_mutex_unlock(self.0.get()) } {
			0 => OsalRsBool::True,
			_ => OsalRsBool::False,
		}
	}

	/// ISR-safe variant of [`RawMutex::unlock`]; identical on POSIX, since
	/// unlocking never blocks.
	fn unlock_from_isr(&self) -> OsalRsBool {
		self.unlock()
	}

	/// Destroys the underlying pthread mutex and resets it to its "null"
	/// state. Safe to call more than once, and called automatically on
	/// [`Drop`] if not called explicitly.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mut mutex = RawMutex::new().unwrap();
	/// mutex.delete();
	/// assert!(mutex.is_null());
	/// ```
	fn delete(&mut self) {
		if self.is_null() {
			return;
		}

		unsafe {
			pthread_mutex_destroy(self.0.get());
		}

		*self.0.get_mut() = MutexHandle::default();
	}
}

impl Drop for RawMutex {
	fn drop(&mut self) {
		self.delete();
	}
}

impl Deref for RawMutex {
	type Target = MutexHandle;

	fn deref(&self) -> &MutexHandle {
		unsafe { &*self.0.get() }
	}
}

impl Debug for RawMutex {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {

		f.debug_struct("RawMutex")
			.field("handle", &(&raw const self).addr())
			.finish()
	}
}

impl Display for RawMutex {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "RawMutex {{ handle: {:?} }}", &(&raw const self).addr())
	}
}

/// A mutex protecting a `T`, unlocked through RAII guards rather than
/// explicit lock/unlock calls. Built on [`RawMutex`], so it shares the same
/// priority-inheritance and recursive-locking behavior.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
///
/// let mutex = Mutex::new(0);
/// {
///     let mut guard = mutex.lock().unwrap();
///     *guard += 1;
/// } // Lock released here, when `guard` drops.
///
/// assert_eq!(*mutex.lock().unwrap(), 1);
/// ```
pub struct Mutex<T: ?Sized> {
	inner: RawMutex,
	data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T: ?Sized> Mutex<T> {
	/// Wraps `data` in a new, unlocked mutex.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mutex = Mutex::new(vec![1, 2, 3]);
	/// assert_eq!(mutex.lock().unwrap().len(), 3);
	/// ```
	pub fn new(data: T) -> Self
	where
		T: Sized,
	{
		Self {
			inner: RawMutex::new().unwrap(),
			data: UnsafeCell::new(data),
		}
	}

	#[inline]
	fn get_mut_ref(&mut self) -> &mut T {
		unsafe { &mut *self.data.get() }
	}
}

impl<T: ?Sized> MutexFn<T> for Mutex<T> {
	type Guard<'a> = MutexGuard<'a, T> where Self: 'a, T: 'a;
	type GuardFromIsr<'a> = MutexGuardFromIsr<'a, T> where Self: 'a, T: 'a;

	/// Blocks the calling thread until the mutex is available, then returns
	/// a RAII guard that unlocks it on [`Drop`].
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mutex = Mutex::new(10);
	/// let mut guard = mutex.lock().unwrap();
	/// *guard += 5;
	/// drop(guard);
	///
	/// assert_eq!(*mutex.lock().unwrap(), 15);
	/// ```
	fn lock(&self) -> Result<Self::Guard<'_>> {
		match self.inner.lock() {
			OsalRsBool::True => Ok(MutexGuard {
				mutex: self,
				_phantom: PhantomData,
			}),
			OsalRsBool::False => Err(Error::MutexLockFailed),
		}
	}

	/// ISR-safe variant of [`Mutex::lock`]: never blocks, failing with
	/// [`Error::MutexLockFailed`] instead of waiting if the mutex is already
	/// held (POSIX has no real interrupt context of its own, so this is the
	/// non-blocking equivalent expected of `_from_isr` methods).
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mutex = Mutex::new(0);
	/// let mut guard = mutex.lock_from_isr().unwrap();
	/// *guard = 42;
	/// drop(guard);
	///
	/// assert_eq!(*mutex.lock().unwrap(), 42);
	/// ```
	fn lock_from_isr(&self) -> Result<Self::GuardFromIsr<'_>> {
		match self.inner.lock_from_isr() {
			OsalRsBool::True => Ok(MutexGuardFromIsr {
				mutex: self,
				_phantom: PhantomData,
			}),
			OsalRsBool::False => Err(Error::MutexLockFailed),
		}
	}

	/// Consumes the mutex, returning the wrapped value without needing to
	/// lock it (the type system already guarantees exclusive access here).
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mutex = Mutex::new(String::from("hello"));
	/// let value = mutex.into_inner().unwrap();
	/// assert_eq!(value, "hello");
	/// ```
	fn into_inner(self) -> Result<T>
	where
		Self: Sized,
		T: Sized,
	{
		Ok(self.data.into_inner())
	}

	/// Returns a mutable reference to the wrapped value without locking (a
	/// `&mut Mutex<T>` already guarantees exclusive access).
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mut mutex = Mutex::new(1);
	/// *mutex.get_mut() += 1;
	/// assert_eq!(*mutex.lock().unwrap(), 2);
	/// ```
	fn get_mut(&mut self) -> &mut T {
		self.get_mut_ref()
	}
}

impl<T: ?Sized> Mutex<T> {
	/// Same as [`MutexFn::lock_from_isr`], exposed as an inherent method so
	/// it's callable without importing the [`MutexFn`] trait.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mutex = Mutex::new(0);
	/// let mut guard = mutex.lock_from_isr_explicit().unwrap();
	/// *guard = 7;
	/// ```
	pub fn lock_from_isr_explicit(&self) -> Result<MutexGuardFromIsr<'_, T>> {
		match self.inner.lock_from_isr() {
			OsalRsBool::True => Ok(MutexGuardFromIsr {
				mutex: self,
				_phantom: PhantomData,
			}),
			OsalRsBool::False => Err(Error::MutexLockFailed),
		}
	}
}

impl<T> Mutex<T> {
	/// Convenience constructor for the common case of sharing a mutex
	/// between threads: equivalent to `Arc::new(Mutex::new(data))`.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let shared = Mutex::new_arc(0);
	/// let clone_for_worker = shared.clone();
	///
	/// *clone_for_worker.lock().unwrap() += 1;
	/// assert_eq!(*shared.lock().unwrap(), 1);
	/// ```
	pub fn new_arc(data: T) -> Arc<Self> {
		Arc::new(Self::new(data))
	}
}

impl<T: ?Sized> Debug for Mutex<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Mutex")
			.field("inner", &self.inner)
			.finish()
	}
}

impl<T: ?Sized> Display for Mutex<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "Mutex {{ inner: {} }}", self.inner)
	}
}

/// RAII guard returned by [`Mutex::lock`]. Unlocks the mutex when dropped;
/// derefs to `&T`/`&mut T` in the meantime.
pub struct MutexGuard<'a, T: ?Sized + 'a> {
	mutex: &'a Mutex<T>,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
	/// Raw handle of the pthread mutex backing this guard.
	///
	/// Lets a condition variable (`pthread_cond_wait`/`pthread_cond_timedwait`)
	/// atomically unlock/re-lock the same OS mutex this guard represents,
	/// without going through [`RawMutexFn::unlock`]/`lock` — those calls are
	/// made internally by libc during the wait, so the guard's Rust-level
	/// "locked" state stays valid across it.
	pub(crate) fn raw_handle(&self) -> *mut MutexHandle {
		self.mutex.inner.0.get()
	}
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.mutex.data.get() }
	}
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.mutex.data.get() }
	}
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
	fn drop(&mut self) {
		let _ = self.mutex.inner.unlock();
	}
}

impl<'a, T: ?Sized> MutexGuardFn<'a, T> for MutexGuard<'a, T> {
	/// Replaces the guarded value with a clone of `t`, without needing a
	/// separate `*guard = t.clone()` statement.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mutex = Mutex::new(0);
	/// let mut guard = mutex.lock().unwrap();
	/// guard.update(&42);
	/// assert_eq!(*guard, 42);
	/// ```
	fn update(&mut self, t: &T)
	where
		T: Clone,
	{
		**self = t.clone();
	}
}

/// RAII guard returned by [`Mutex::lock_from_isr`]/[`Mutex::lock_from_isr_explicit`].
/// Unlocks the mutex (via the non-blocking `_from_isr` path) when dropped;
/// derefs to `&T`/`&mut T` in the meantime.
pub struct MutexGuardFromIsr<'a, T: ?Sized + 'a> {
	mutex: &'a Mutex<T>,
	_phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> Deref for MutexGuardFromIsr<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		unsafe { &*self.mutex.data.get() }
	}
}

impl<'a, T: ?Sized> DerefMut for MutexGuardFromIsr<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *self.mutex.data.get() }
	}
}

impl<'a, T: ?Sized> Drop for MutexGuardFromIsr<'a, T> {
	fn drop(&mut self) {
		let _ = self.mutex.inner.unlock_from_isr();
	}
}

impl<'a, T: ?Sized> MutexGuardFn<'a, T> for MutexGuardFromIsr<'a, T> {
	/// See [`MutexGuard::update`]; behaves identically here.
	///
	/// # Examples
	///
	/// ```
	/// use osal_rs::os::*;
	///
	/// let mutex = Mutex::new(0);
	/// let mut guard = mutex.lock_from_isr().unwrap();
	/// guard.update(&7);
	/// assert_eq!(*guard, 7);
	/// ```
	fn update(&mut self, t: &T)
	where
		T: Clone,
	{
		**self = t.clone();
	}
}