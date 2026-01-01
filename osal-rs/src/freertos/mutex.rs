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

use core::cell::UnsafeCell;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Deref, DerefMut};
use core::marker::PhantomData;

use alloc::sync::Arc;

use super::ffi::{MutexHandle, pdFALSE, pdTRUE};
use super::system::System;
use crate::traits::SystemFn;
use crate::traits::{MutexGuardFn, RawMutexFn, MutexFn, ToTick};
use crate::utils::{Result, Error, OsalRsBool, MAX_DELAY};
use crate::{vSemaphoreDelete, xSemaphoreCreateRecursiveMutex, xSemaphoreGiveFromISR, xSemaphoreGiveRecursive, xSemaphoreTake, xSemaphoreTakeFromISR, xSemaphoreTakeRecursive};


struct RawMutex(MutexHandle);

unsafe impl Send for RawMutex {}
unsafe impl Sync for RawMutex {}

impl RawMutexFn for RawMutex {
    fn new() -> Result<Self> {
        let handle = xSemaphoreCreateRecursiveMutex!();
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(RawMutex(handle))
        }
    }
    
    fn lock(&self) -> OsalRsBool {
        let res = xSemaphoreTakeRecursive!(self.0, MAX_DELAY.to_ticks());
        if res == pdTRUE {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn lock_from_isr(&self) -> OsalRsBool {
        let mut higher_priority_task_woken = pdFALSE;
        let res = xSemaphoreTakeFromISR!(self.0, &mut higher_priority_task_woken);
        if res == pdTRUE {

            System::yield_from_isr(higher_priority_task_woken);

            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn unlock(&self) -> OsalRsBool {
        let res = xSemaphoreGiveRecursive!(self.0);
        if res == pdTRUE {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }


    fn unlock_from_isr(&self) -> OsalRsBool {
        let mut higher_priority_task_woken = pdFALSE;
        let res = xSemaphoreGiveFromISR!(self.0, &mut higher_priority_task_woken);
        if res == pdTRUE {
            
            System::yield_from_isr(higher_priority_task_woken);
            
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn delete(&mut self) {
        vSemaphoreDelete!(self.0);
        self.0 = core::ptr::null();
    }
}

impl Drop for RawMutex {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
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
    data: UnsafeCell<T>
}


unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T: ?Sized> MutexFn<T> for Mutex<T> {
    type Guard<'a> = MutexGuard<'a, T> where Self: 'a, T: 'a;
    type GuardFromIsr<'a> = MutexGuardFromIsr<'a, T> where Self: 'a, T: 'a;

    fn new(data: T) -> Self
    where 
        T: Sized
    {
        Self {
            inner: RawMutex::new().unwrap(),
            data: UnsafeCell::new(data),
        }
    }

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
        T: Sized 
    {
        Ok(self.data.into_inner())
    }

    fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires the mutex from ISR context, returning a specific ISR guard
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
    /// Creates a new mutex wrapped in an Arc for easy sharing between threads.
    /// This is a convenience method that combines `Arc::new(Mutex::new(data))`.
    /// 
    /// # Example
    /// ```ignore
    /// let shared_data = Mutex::new_arc(0u32);
    /// let data_clone = Arc::clone(&shared_data);
    /// ```
    pub fn new_arc(data: T) -> Arc<Self> {
        Arc::new(Self::new(data))
    }
}

impl<T> Debug for Mutex<T> 
where 
    T: ?Sized {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mutex")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Display for Mutex<T> 
where 
    T: ?Sized {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Mutex {{ inner: {} }}", self.inner)
    }   
}

/// RAII guard returned by `Mutex::lock`
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
        self.mutex.inner.unlock();
    }
}

impl<'a, T: ?Sized> MutexGuardFn<'a, T> for MutexGuard<'a, T> {}

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
        self.mutex.inner.unlock_from_isr();
    }
}

impl<'a, T: ?Sized> MutexGuardFn<'a, T> for MutexGuardFromIsr<'a, T> {}