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
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::null;

#[cfg(not(feature = "serde"))]
use crate::traits::{Deserialize, Serialize};
use crate::traits::{BytesHasLen, QueueFn, QueueStreamedFn, ToTick};
use crate::utils::{Error, Result};
use crate::posix::types::{QueueHandle, TickType, UBaseType};

#[cfg(feature = "serde")]
use osal_rs_serde::{Deserialize, Serialize};

pub trait StructSerde: Serialize + BytesHasLen + Deserialize {}

impl<T> StructSerde for T where T: Serialize + BytesHasLen + Deserialize {}

pub struct Queue(QueueHandle);

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

impl Queue {
	pub fn new(size: UBaseType, message_size: UBaseType) -> Result<Self> {
		if size == 0 || message_size == 0 {
			Err(Error::InvalidQueueSize)
		} else {
			Ok(Self(null()))
		}
	}

	#[inline]
	pub fn fetch_with_to_tick(&self, buffer: &mut [u8], time: impl ToTick) -> Result<()> {
		self.fetch(buffer, time.to_ticks())
	}

	#[inline]
	pub fn post_with_to_tick(&self, item: &[u8], time: impl ToTick) -> Result<()> {
		self.post(item, time.to_ticks())
	}
}

impl QueueFn for Queue {
	fn fetch(&self, _buffer: &mut [u8], _time: TickType) -> Result<()> {
		Err(Error::Timeout)
	}

	fn fetch_from_isr(&self, _buffer: &mut [u8]) -> Result<()> {
		Err(Error::Timeout)
	}

	fn post(&self, _item: &[u8], _time: TickType) -> Result<()> {
		Ok(())
	}

	fn post_from_isr(&self, _item: &[u8]) -> Result<()> {
		Ok(())
	}

	fn delete(&mut self) {
		self.0 = null();
	}
}

impl Drop for Queue {
	fn drop(&mut self) {
		self.0 = null();
	}
}

impl Deref for Queue {
	type Target = QueueHandle;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Debug for Queue {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Queue")
			.field("handle", &self.0)
			.finish()
	}
}

impl Display for Queue {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "Queue {{ handle: {:?} }}", self.0)
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
	fn fetch(&self, _buffer: &mut T, _time: TickType) -> Result<()> {
		Err(Error::Timeout)
	}

	fn fetch_from_isr(&self, _buffer: &mut T) -> Result<()> {
		Err(Error::Timeout)
	}

	fn post(&self, _item: &T, _time: TickType) -> Result<()> {
		Ok(())
	}

	fn post_from_isr(&self, _item: &T) -> Result<()> {
		Ok(())
	}

	fn delete(&mut self) {
		self.0.delete();
	}
}

#[cfg(feature = "serde")]
impl<T> QueueStreamedFn<T> for QueueStreamed<T>
where
	T: StructSerde,
{
	fn fetch(&self, _buffer: &mut T, _time: TickType) -> Result<()> {
		Err(Error::Timeout)
	}

	fn fetch_from_isr(&self, _buffer: &mut T) -> Result<()> {
		Err(Error::Timeout)
	}

	fn post(&self, _item: &T, _time: TickType) -> Result<()> {
		Ok(())
	}

	fn post_from_isr(&self, _item: &T) -> Result<()> {
		Ok(())
	}

	fn delete(&mut self) {
		self.0.delete();
	}
}

impl<T> Deref for QueueStreamed<T>
where
	T: StructSerde,
{
	type Target = QueueHandle;

	fn deref(&self) -> &Self::Target {
		&self.0.0
	}
}

impl<T> Debug for QueueStreamed<T>
where
	T: StructSerde,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("QueueStreamed")
			.field("handle", &self.0.0)
			.finish()
	}
}

impl<T> Display for QueueStreamed<T>
where
	T: StructSerde,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "QueueStreamed {{ handle: {:?} }}", self.0.0)
	}
}