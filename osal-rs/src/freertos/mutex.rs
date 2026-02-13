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

//! Mutex synchronization primitives for FreeRTOS.
//!
//! This module provides safe mutual exclusion primitives built on top of FreeRTOS
//! recursive mutexes. It supports RAII-style lock guards for automatic lock management
//! and ISR-safe variants for interrupt contexts.

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
use crate::{vSemaphoreDelete, xSemaphoreCreateRecursiveMutex, xSemaphoreGiveFromISR, xSemaphoreGiveRecursive, xSemaphoreTakeFromISR, xSemaphoreTakeRecursive};

//// RawMutex ////

/// Low-level recursive mutex wrapper for FreeRTOS.
///
/// This is the underlying implementation of the mutex that directly interfaces
/// with FreeRTOS semaphore APIs. It's recursive, meaning the same thread can
/// lock it multiple times.
///
/// # Note
///
/// Users should typically use [`Mutex<T>`] instead, which provides type-safe
/// data protection. This type is exposed for advanced use cases.
pub struct RawMutex(MutexHandle);

unsafe impl Send for RawMutex {}
unsafe impl Sync for RawMutex {}

impl RawMutex {
    /// Creates a new raw recursive mutex.
    ///
    /// # Returns
    /// * `Ok(RawMutex)` - Successfully created
    /// * `Err(Error::OutOf Memory)` - Failed to allocate mutex resources
    pub fn new() -> Result<Self> {
        let handle = xSemaphoreCreateRecursiveMutex!();
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(RawMutex(handle))
        }
    }
}

impl RawMutexFn for RawMutex {

    
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

//// Mutex ////

/// A mutual exclusion primitive useful for protecting shared data.
///
/// This mutex will block threads waiting for the lock to become available.
/// The mutex is implemented using FreeRTOS recursive mutexes, supporting
/// priority inheritance to prevent priority inversion.
///
/// # Type Parameters
///
/// * `T` - The type of data protected by the mutex
///
/// # Examples
///
/// ## Basic usage
///
/// ```ignore
/// use osal_rs::os::Mutex;
/// 
/// let mutex = Mutex::new(0);
/// 
/// // Acquire the lock and modify the data
/// {
///     let mut guard = mutex.lock().unwrap();
///     *guard += 1;
/// }  // Lock is automatically released here
/// ```
///
/// ## Sharing between threads
///
/// ```ignore
/// use osal_rs::os::{Mutex, Thread};
/// use alloc::sync::Arc;
/// 
/// let counter = Arc::new(Mutex::new(0));
/// let counter_clone = counter.clone();
/// 
/// let thread = Thread::new("worker", 2048, 5, move || {
///     let mut guard = counter_clone.lock().unwrap();
///     *guard += 1;
/// }).unwrap();
/// 
/// thread.start().unwrap();
/// ```
///
/// ## Using from ISR context
///
/// ```ignore
/// use osal_rs::os::Mutex;
/// 
/// let mutex = Mutex::new(0);
/// 
/// // In an interrupt handler:
/// if let Ok(mut guard) = mutex.lock_from_isr() {
///     *guard = 42;
/// }
/// ```
pub struct Mutex<T: ?Sized> {
    inner: RawMutex,
    data: UnsafeCell<T>
}


unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T: ?Sized>  Mutex<T> {
        
    /// Creates a new mutex wrapping the supplied data.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Mutex, MutexFn};
    /// 
    /// let mutex = Mutex::new(0);
    pub fn new(data: T) -> Self
    where 
        T: Sized
    {
        Self {
            inner: RawMutex::new().unwrap(),
            data: UnsafeCell::new(data),
        }
    }

    fn get_mut(&mut self) -> &mut T {
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

    /// Consumes the mutex and returns the inner data.
    ///
    /// This is safe because we have unique ownership of the mutex.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Mutex, MutexFn};
    /// 
    /// let mutex = Mutex::new(5);
    /// let value = mutex.into_inner().unwrap();
    /// assert_eq!(value, 5);
    /// ```
    fn into_inner(self) -> Result<T> 
    where 
        Self: Sized, 
        T: Sized 
    {
        Ok(self.data.into_inner())
    }

    /// Returns a mutable reference to the inner data.
    ///
    /// Since this takes `&mut self`, we know there are no other references
    /// to the data, so we can safely return a mutable reference.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Mutex, MutexFn};
    /// 
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.get_mut(), 10);
    /// ```
    fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires the mutex from ISR context, returning a specific ISR guard.
    ///
    /// This is an explicit version of `lock_from_isr` that returns the ISR-specific guard type.
    ///
    /// # Returns
    ///
    /// * `Ok(MutexGuardFromIsr)` - Lock acquired
    /// * `Err(Error::MutexLockFailed)` - Failed to acquire lock
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In ISR context:
    /// if let Ok(guard) = mutex.lock_from_isr_explicit() {
    ///     *guard = new_value;
    /// }
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
    /// Creates a new mutex wrapped in an Arc for easy sharing between threads.
    ///
    /// This is a convenience method equivalent to `Arc::new(Mutex::new(data))`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Mutex;
    /// use alloc::sync::Arc;
    /// 
    /// let shared_data = Mutex::new_arc(0u32);
    /// let data_clone = Arc::clone(&shared_data);
    /// 
    /// // Use in thread...
    /// let thread = Thread::new("worker", 2048, 5, move || {
    ///     let mut guard = data_clone.lock().unwrap();
    ///     *guard += 1;
    /// });
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

//// MutexGuard ////

/// RAII guard returned by `Mutex::lock()`.
///
/// When this guard goes out of scope, the mutex is automatically unlocked.
/// Provides access to the protected data through `Deref` and `DerefMut`.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Mutex, MutexFn};
/// 
/// let mutex = Mutex::new(0);
/// 
/// {
///     let mut guard = mutex.lock().unwrap();
///     *guard += 1;  // Access protected data
/// }  // Mutex automatically unlocked here
/// ```
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

impl<'a, T: ?Sized> MutexGuardFn<'a, T> for MutexGuard<'a, T> {
    /// Updates the protected value with a new value.
    ///
    /// # Note
    ///
    /// This requires `T` to implement `Clone` to copy the value.
    /// Use the dereference operator directly for types that implement `Copy`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut guard = mutex.lock().unwrap();
    /// let new_value = 42;
    /// guard.update(&new_value);
    /// ```
    fn update(&mut self, t: &T) 
    where
        T: Clone
    {
        // Dereference twice: first to get &mut T from MutexGuard,
        // then assign the cloned value
        **self = t.clone();
    }
}

/// RAII guard returned by `Mutex::lock_from_isr()`.
///
/// Similar to `MutexGuard` but specifically for ISR context.
/// Automatically unlocks the mutex when dropped using ISR-safe unlock.
///
/// # Examples
///
/// ```ignore
/// // In ISR context:
/// if let Ok(mut guard) = mutex.lock_from_isr() {
///     *guard = new_value;
/// }  // Automatically unlocked with ISR-safe method
/// ```
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

impl<'a, T: ?Sized> MutexGuardFn<'a, T> for MutexGuardFromIsr<'a, T> {
    /// Updates the protected value from ISR context.
    ///
    /// # Note
    ///
    /// This requires `T` to implement `Clone` to copy the value.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In ISR context:
    /// if let Ok(mut guard) = mutex.lock_from_isr() {
    ///     guard.update(&new_value);
    /// }
    /// ```
    fn update(&mut self, t: &T) 
    where
        T: Clone
    {
        **self = t.clone();
    }
}
