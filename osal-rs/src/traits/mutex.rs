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

use core::ops::{Deref, DerefMut};

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

/// Static mutex trait for compile-time allocated mutexes.
///
/// This trait provides mutex operations for statically allocated mutexes,
/// which can be used in const contexts or as static variables. Unlike the
/// `Mutex` trait, this doesn't require dynamic allocation and can be
/// initialized at compile time.
///
/// # Type Parameters
///
/// * `T` - The type of data protected by the mutex
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::StaticMutex;
/// 
/// static COUNTER: StaticMutex<u32> = StaticMutex::new(0);
/// 
/// fn increment() {
///     let mut guard = COUNTER.lock().unwrap();
///     *guard += 1;
/// }
/// ```
pub trait StaticMutex<T> {
    /// Guard type returned by `lock()`, providing RAII-style mutex access.
    ///
    /// Automatically releases the mutex when dropped. Implements `Deref` and
    /// `DerefMut` for transparent access to the protected data.
    type Guard<'a>: Deref<Target = T> + DerefMut
    where
        Self: 'a;
    
    /// Guard type returned by `lock_from_isr()`, for ISR-context mutex access.
    ///
    /// Similar to `Guard` but designed for use in interrupt service routines.
    /// Automatically releases the mutex when dropped.
    type GuardFromIsr<'a>: Deref<Target = T> + DerefMut
    where
        Self: 'a;

    /// Acquires the mutex, blocking until it becomes available.
    ///
    /// Returns a guard that provides access to the protected data and
    /// automatically releases the mutex when dropped.
    ///
    /// # Returns
    ///
    /// * `Ok(Guard)` - Successfully acquired the mutex
    /// * `Err(_)` - Failed to acquire (e.g., mutex deleted or invalid)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let guard = mutex.lock()?;
    /// *guard = 42;  // Mutex automatically released when guard drops
    /// ```
    fn lock(&self) -> Result<Self::Guard<'_>>;

    /// Acquires the mutex from an interrupt service routine (ISR).
    ///
    /// Non-blocking version for use in ISR context. Returns immediately
    /// if the mutex cannot be acquired.
    ///
    /// # Returns
    ///
    /// * `Ok(GuardFromIsr)` - Successfully acquired the mutex
    /// * `Err(_)` - Could not acquire (already locked or invalid)
    ///
    /// # Safety
    ///
    /// Must only be called from ISR context. Calling from normal task
    /// context may lead to priority inversion or deadlock.
    fn lock_from_isr(&self) -> Result<Self::GuardFromIsr<'_>>;

    /// Returns a mutable reference to the underlying data.
    ///
    /// This method bypasses the locking mechanism and provides direct access
    /// to the protected data. Safe because it requires exclusive mutable
    /// access to the mutex itself.
    ///
    /// # Parameters
    ///
    /// * `&mut self` - Exclusive mutable reference ensuring no other access
    ///
    /// # Returns
    ///
    /// Mutable reference to the protected data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut mutex = StaticMutex::new(0);
    /// *mutex.get_mut() = 42;  // No locking needed
    /// ```
    fn get_mut(&mut self) -> &mut T;
}
