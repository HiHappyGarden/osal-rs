use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::marker::PhantomData;

use crate::freertos::ffi::{MutexHandle, osal_rs_port_yield_from_isr, pdFALSE, pdTRUE};
use crate::traits::{MutexGuardFn, RawMutexFn, MutexFn, ToTick};
use crate::utils::{Result, Error, OsalRsBool, MAX_DELAY};
use crate::{vSemaphoreDelete, xSemaphoreCreateRecursiveMutex, xSemaphoreGiveFromISR, xSemaphoreGiveRecursive, xSemaphoreTake, xSemaphoreTakeFromISR};

pub struct Mutex<T: ?Sized> {
    inner: RawMutex,
    data: UnsafeCell<T>,
}

struct RawMutex(MutexHandle);

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
        let res = xSemaphoreTake!(self.0, MAX_DELAY.to_tick());
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
            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);
            }
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
            unsafe {
                osal_rs_port_yield_from_isr(higher_priority_task_woken);
            }
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

unsafe impl Send for RawMutex {}
unsafe impl Sync for RawMutex {}

impl<T: ?Sized> MutexFn<T> for Mutex<T> {
    type Guard<'a> = MutexGuard<'a, T> where Self: 'a, T: 'a;
    type GuardFromIsr<'a> = MutexGuardFromIsr<'a, T> where Self: 'a, T: 'a;

    fn new(data: T) -> Result<Self> 
    where 
        Self: Sized,
        T: Sized 
    {
        Ok(Mutex {
            inner: RawMutex::new()?,
            data: UnsafeCell::new(data),
        })
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

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

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