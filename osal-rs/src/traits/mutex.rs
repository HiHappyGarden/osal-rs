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

//! Mutex trait definitions.

use crate::utils::{OsalRsBool, Result};

/// Low-level raw mutex operations.
///
/// This trait defines the basic mutex primitives that interface directly
/// with the underlying RTOS mutex implementation.
pub trait RawMutex
where
    Self: Sized,
{
    /// Locks the mutex (blocking).
    ///
    /// # Returns
    ///
    /// `True` if lock was acquired, `False` otherwise
    fn lock(&self) -> OsalRsBool;

    /// Locks the mutex from ISR context (non-blocking).
    ///
    /// # Returns
    ///
    /// `True` if lock was acquired, `False` otherwise
    fn lock_from_isr(&self) -> OsalRsBool;

    /// Unlocks the mutex.
    ///
    /// # Returns
    ///
    /// `True` if unlock succeeded, `False` otherwise
    fn unlock(&self) -> OsalRsBool;

    /// Unlocks the mutex from ISR context.
    ///
    /// # Returns
    ///
    /// `True` if unlock succeeded, `False` otherwise
    fn unlock_from_isr(&self) -> OsalRsBool;

    /// Deletes the mutex and frees its resources.
    fn delete(&mut self);
}

/// Marker trait for mutex guard types.
///
/// Implemented by types that represent active mutex locks.
pub trait MutexGuard<'a, T: ?Sized + 'a> {
    /// Updates the value protected by the mutex guard.
    ///
    /// # Parameters
    ///
    /// * `t` - Reference to the new value to assign
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Mutex, MutexFn};
    /// use osal_rs::traits::MutexGuard;
    /// 
    /// let mutex = Mutex::new(0);
    /// let mut guard = mutex.lock().unwrap();
    /// guard.update(&42);
    /// assert_eq!(*guard, 42);
    /// ```
    fn update(&mut self, t: &T)
    where
        T: Clone;

}

/// High-level mutex trait with type-safe data protection.
///
/// This trait provides RAII-style mutex operations with automatic lock
/// management through guard types.
pub trait Mutex<T: ?Sized> {
    /// The guard type for normal mutex locks
    type Guard<'a>: MutexGuard<'a, T> where Self: 'a, T: 'a;
    /// The guard type for ISR-context mutex locks
    type GuardFromIsr<'a>: MutexGuard<'a, T> where Self: 'a, T: 'a;

    /// Acquires the mutex, blocking the current thread until it is able to do so
    fn lock(&self) -> Result<Self::Guard<'_>>;
    
    /// Acquires the mutex from ISR context
    fn lock_from_isr(&self) -> Result<Self::GuardFromIsr<'_>>;

    /// Attempts to consume this mutex, returning the underlying data
    fn into_inner(self) -> Result<T> 
    where 
        Self: Sized, 
        T: Sized;

    /// Returns a mutable reference to the underlying data
    fn get_mut(&mut self) -> &mut T;
}
