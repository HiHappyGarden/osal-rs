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

//! Mutex trait definitions.
//!
//! This module provides traits for mutual exclusion (mutex) synchronization
//! primitives, enabling safe shared access to data across multiple tasks.
//!
//! # Overview
//!
//! Mutexes prevent race conditions by ensuring only one task can access
//! protected data at a time. This module provides both low-level raw mutex
//! operations and high-level RAII-style interfaces.
//!
//! # Concepts
//!
//! - **RAII Guards**: Locks are automatically released when guard goes out of scope
//! - **Priority Inheritance**: Some implementations support priority inheritance to prevent priority inversion
//! - **ISR Safety**: Special methods for use in interrupt service routines
//!
//! # Deadlock Prevention
//!
//! - Always acquire mutexes in the same order
//! - Don't hold locks longer than necessary
//! - Avoid calling blocking operations while holding a lock
//!
//! # Examples
//!
//! ```ignore
//! use osal_rs::os::Mutex;
//!
//! let mutex = Mutex::new(0);
//!
//! // Lock automatically released when guard goes out of scope
//! {
//!     let mut guard = mutex.lock().unwrap();
//!     *guard += 1;
//! } // Lock released here
//! ```

use crate::utils::{OsalRsBool, Result};

/// Low-level raw mutex operations.
///
/// This trait defines the basic mutex primitives that interface directly
/// with the underlying RTOS mutex implementation.
///
/// # Implementation Notes
///
/// Implementations should support priority inheritance where available to
/// prevent priority inversion problems in real-time systems.
///
/// # Safety
///
/// - `lock()` must only be called from task context (not ISR)
/// - `lock_from_isr()` must only be called from ISR context
/// - `unlock()` must be called by the same task that acquired the lock
/// - Deadlocks can occur if locks are not acquired in consistent order
///
/// # Examples
///
/// ```ignore
/// use osal_rs::traits::RawMutex;
///
/// // Acquire and release lock
/// if raw_mutex.lock() {
///     // Critical section
///     raw_mutex.unlock();
/// }
/// ```
pub trait RawMutex
where
    Self: Sized,
{
    /// Locks the mutex (blocking).
    ///
    /// Blocks the calling task until the mutex becomes available.
    /// Must only be called from task context, not from ISR.
    ///
    /// # Returns
    ///
    /// * `True` - Lock was successfully acquired
    /// * `False` - Lock acquisition failed (should be rare)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if raw_mutex.lock() {
    ///     // Protected code here
    ///     raw_mutex.unlock();
    /// }
    /// ```
    fn lock(&self) -> OsalRsBool;

    /// Locks the mutex from ISR context (non-blocking).
    ///
    /// Attempts to acquire the lock without blocking. Must only be
    /// called from interrupt service routine context.
    ///
    /// # Returns
    ///
    /// * `True` - Lock was successfully acquired
    /// * `False` - Lock is currently held by another task
    ///
    /// # Note
    ///
    /// This is a try-lock operation that returns immediately.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In ISR handler
    /// if raw_mutex.lock_from_isr() {
    ///     // Quick critical operation
    ///     raw_mutex.unlock_from_isr();
    /// }
    /// ```
    fn lock_from_isr(&self) -> OsalRsBool;

    /// Unlocks the mutex.
    ///
    /// Releases the mutex that was previously acquired by `lock()`.
    /// Must be called by the same task that acquired the lock.
    ///
    /// # Returns
    ///
    /// * `True` - Unlock succeeded
    /// * `False` - Unlock failed (mutex not owned by caller)
    ///
    /// # Safety
    ///
    /// Calling unlock on a mutex not owned by the current task
    /// may cause undefined behavior.
    fn unlock(&self) -> OsalRsBool;

    /// Unlocks the mutex from ISR context.
    ///
    /// Releases the mutex that was previously acquired by `lock_from_isr()`.
    /// Must only be called from interrupt context.
    ///
    /// # Returns
    ///
    /// * `True` - Unlock succeeded
    /// * `False` - Unlock failed
    fn unlock_from_isr(&self) -> OsalRsBool;

    /// Deletes the mutex and frees its resources.
    ///
    /// # Safety
    ///
    /// The mutex must not be locked by any task when this is called.
    /// Ensure no tasks are waiting for this mutex before deletion.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut raw_mutex = create_raw_mutex();
    /// // Use mutex...
    /// raw_mutex.delete();
    /// ```
    fn delete(&mut self);
}

/// Marker trait for mutex guard types.
///
/// Implemented by types that represent active mutex locks. Guards
/// automatically release the mutex when dropped (RAII pattern).
///
/// # Lifetime
///
/// The `'a` lifetime ensures the guard cannot outlive the mutex it guards.
///
/// # Auto-Unlock
///
/// The mutex is automatically unlocked when the guard goes out of scope,
/// ensuring locks are always properly released even if a panic occurs.
pub trait MutexGuard<'a, T: ?Sized + 'a> {
    /// Updates the value protected by the mutex guard.
    ///
    /// Clones the provided value and replaces the current value
    /// protected by the mutex.
    ///
    /// # Parameters
    ///
    /// * `t` - Reference to the new value to assign
    ///
    /// # Type Requirements
    ///
    /// The type `T` must implement `Clone`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Mutex;
    /// use osal_rs::traits::MutexGuard;
    /// 
    /// let mutex = Mutex::new(0);
    /// let mut guard = mutex.lock().unwrap();
    /// 
    /// // Update with new value
    /// guard.update(&42);
    /// assert_eq!(*guard, 42);
    /// 
    /// // Lock is automatically released when guard drops
    /// ```
    fn update(&mut self, t: &T)
    where
        T: Clone;

}

/// High-level mutex trait with type-safe data protection.
///
/// This trait provides RAII-style mutex operations with automatic lock
/// management through guard types. The mutex owns the data it protects,
/// ensuring data can only be accessed through a locked guard.
///
/// # Type Safety
///
/// The data type `T` is protected at compile time - you cannot access
/// the data without holding the lock.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::Mutex;
///
/// let counter = Mutex::new(0);
///
/// // Task 1
/// {
///     let mut guard = counter.lock().unwrap();
///     *guard += 1;
/// } // Lock released here
///
/// // Task 2
/// {
///     let guard = counter.lock().unwrap();
///     println!("Counter: {}", *guard);
/// }
/// ```
pub trait Mutex<T: ?Sized> {
    /// The guard type for normal mutex locks
    type Guard<'a>: MutexGuard<'a, T> where Self: 'a, T: 'a;
    /// The guard type for ISR-context mutex locks
    type GuardFromIsr<'a>: MutexGuard<'a, T> where Self: 'a, T: 'a;

    /// Acquires the mutex, blocking the current task until it is able to do so.
    ///
    /// This method will block until the lock can be acquired. When the lock
    /// is acquired, a guard is returned that provides access to the protected
    /// data and automatically releases the lock when dropped.
    ///
    /// # Returns
    ///
    /// * `Ok(Guard)` - Lock acquired successfully
    /// * `Err(Error)` - Lock acquisition failed (rare)
    ///
    /// # Panics
    ///
    /// May panic if called from ISR context. Use `lock_from_isr()` instead.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mutex = Mutex::new(vec![1, 2, 3]);
    ///
    /// let mut guard = mutex.lock().unwrap();
    /// guard.push(4);
    /// // Lock automatically released when guard goes out of scope
    /// ```
    fn lock(&self) -> Result<Self::Guard<'_>>;
    
    /// Acquires the mutex from ISR context.
    ///
    /// This is a non-blocking attempt to acquire the mutex, suitable for
    /// use in interrupt service routines. Returns immediately whether or
    /// not the lock was acquired.
    ///
    /// # Returns
    ///
    /// * `Ok(GuardFromIsr)` - Lock acquired successfully
    /// * `Err(Error)` - Lock is currently held, try again later
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In interrupt handler
    /// match mutex.lock_from_isr() {
    ///     Ok(mut guard) => {
    ///         *guard += 1;
    ///         // Lock released when guard drops
    ///     },
    ///     Err(_) => {
    ///         // Lock unavailable, skip or retry later
    ///     }
    /// }
    /// ```
    fn lock_from_isr(&self) -> Result<Self::GuardFromIsr<'_>>;

    /// Attempts to consume this mutex, returning the underlying data.
    ///
    /// This method consumes the mutex and returns the protected data.
    /// Since the mutex is consumed, no locking is required.
    ///
    /// # Returns
    ///
    /// * `Ok(T)` - The data that was protected by the mutex
    /// * `Err(Error)` - Failed to consume mutex (e.g., still locked)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mutex = Mutex::new(vec![1, 2, 3]);
    ///
    /// let data = mutex.into_inner().unwrap();
    /// assert_eq!(data, vec![1, 2, 3]);
    /// ```
    fn into_inner(self) -> Result<T> 
    where 
        Self: Sized, 
        T: Sized;

    /// Returns a mutable reference to the underlying data.
    ///
    /// This method does not require locking since it takes a mutable
    /// reference to the mutex itself, which guarantees exclusive access
    /// at compile time.
    ///
    /// # Returns
    ///
    /// A mutable reference to the protected data.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut mutex = Mutex::new(0);
    ///
    /// // No lock needed - we have exclusive access
    /// *mutex.get_mut() = 42;
    /// ```
    fn get_mut(&mut self) -> &mut T;
}
