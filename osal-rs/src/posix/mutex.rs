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
use core::fmt::{Debug, Display, Formatter};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::null;

use alloc::sync::Arc;

use crate::posix::types::MutexHandle;
use crate::traits::{MutexFn, MutexGuardFn, RawMutexFn};
use crate::utils::{Error, OsalRsBool, Result};

#[derive(Clone)]
pub struct RawMutex(MutexHandle);

unsafe impl Send for RawMutex {}
unsafe impl Sync for RawMutex {}

impl RawMutex {
	pub fn new() -> Result<Self> {
		Ok(Self(null()))
	}
}

impl RawMutexFn for RawMutex {
	fn lock(&self) -> OsalRsBool {
		OsalRsBool::True
	}

	fn lock_from_isr(&self) -> OsalRsBool {
		OsalRsBool::True
	}

	fn unlock(&self) -> OsalRsBool {
		OsalRsBool::True
	}

	fn unlock_from_isr(&self) -> OsalRsBool {
		OsalRsBool::True
	}

	fn delete(&mut self) {
		self.0 = null();
	}
}

impl Drop for RawMutex {
	fn drop(&mut self) {
		self.0 = null();
	}
}

impl Deref for RawMutex {
	type Target = MutexHandle;

	fn deref(&self) -> &MutexHandle {
		&self.0
	}
}

impl Debug for RawMutex {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("RawMutex")
			.field("handle", &self.0)
			.finish()
	}
}

impl Display for RawMutex {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "RawMutex {{ handle: {:?} }}", self.0)
	}
}

pub struct Mutex<T: ?Sized> {
	inner: RawMutex,
	data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T: ?Sized> Mutex<T> {
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

	fn lock(&self) -> Result<Self::Guard<'_>> {
		match self.inner.lock() {
			OsalRsBool::True => Ok(MutexGuard {
				mutex: self,
				_phantom: PhantomData,
			}),
			OsalRsBool::False => Err(Error::MutexLockFailed),
		}
	}

	fn lock_from_isr(&self) -> Result<Self::GuardFromIsr<'_>> {
		match self.inner.lock_from_isr() {
			OsalRsBool::True => Ok(MutexGuardFromIsr {
				mutex: self,
				_phantom: PhantomData,
			}),
			OsalRsBool::False => Err(Error::MutexLockFailed),
		}
	}

	fn into_inner(self) -> Result<T>
	where
		Self: Sized,
		T: Sized,
	{
		Ok(self.data.into_inner())
	}

	fn get_mut(&mut self) -> &mut T {
		self.get_mut_ref()
	}
}

impl<T: ?Sized> Mutex<T> {
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

pub struct MutexGuard<'a, T: ?Sized + 'a> {
	mutex: &'a Mutex<T>,
	_phantom: PhantomData<&'a mut T>,
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
	fn update(&mut self, t: &T)
	where
		T: Clone,
	{
		**self = t.clone();
	}
}

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
	fn update(&mut self, t: &T)
	where
		T: Clone,
	{
		**self = t.clone();
	}
}