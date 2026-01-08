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

//! Semaphore trait for resource management and signaling.
//!
//! Provides counting semaphores for controlling access to shared resources
//! and coordinating task execution.

use crate::os::types::UBaseType;
use crate::utils::{OsalRsBool, Result};
use super::ToTick;

/// Counting semaphore for resource management.
///
/// Semaphores maintain a count that can be incremented (signal) and
/// decremented (wait), useful for:
/// - Protecting shared resources with multiple instances
/// - Task synchronization and signaling
/// - Implementing resource pools
///
/// # Examples
///
/// ## Binary Semaphore (Signaling)
///
/// ```ignore
/// use osal_rs::os::{Semaphore, SemaphoreFn};
/// use core::time::Duration;
/// 
/// let sem = Semaphore::new_with_count(0).unwrap();
/// 
/// // Task 1: Wait for signal
/// sem.wait(Duration::from_secs(1));
/// 
/// // Task 2: Send signal
/// sem.signal();
/// ```
///
/// ## Counting Semaphore (Resource Pool)
///
/// ```ignore
/// // Pool of 3 resources
/// let pool = Semaphore::new(3, 3).unwrap();
/// 
/// // Acquire resource
/// if pool.wait(Duration::from_millis(100)).into() {
///     // Use resource
///     // ...
///     pool.signal();  // Release resource
/// }
/// ```
pub trait Semaphore {

    /// Waits to acquire the semaphore (blocking).
    ///
    /// Decrements the semaphore count if available, otherwise blocks
    /// the calling task until the semaphore becomes available or timeout.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Maximum time to wait (accepts `Duration` or ticks)
    ///
    /// # Returns
    ///
    /// * `True` - Semaphore acquired successfully
    /// * `False` - Timeout occurred
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Semaphore, SemaphoreFn};
    /// use core::time::Duration;
    /// 
    /// let sem = Semaphore::new_with_count(1).unwrap();
    /// 
    /// if sem.wait(Duration::from_millis(100)).into() {
    ///     // Semaphore acquired, critical section
    ///     // ...
    ///     sem.signal();
    /// }
    /// ```
    fn wait(&self, ticks_to_wait: impl ToTick) -> OsalRsBool;

    /// Waits to acquire from ISR context (non-blocking).
    ///
    /// ISR-safe version of `wait()`. Does not block.
    ///
    /// # Returns
    ///
    /// * `True` - Semaphore acquired
    /// * `False` - Semaphore not available
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In interrupt handler
    /// if sem.wait_from_isr().into() {
    ///     // Process event
    /// }
    /// ```
    fn wait_from_isr(&self) -> OsalRsBool;

    /// Signals (releases) the semaphore.
    ///
    /// Increments the semaphore count, potentially unblocking a waiting task.
    ///
    /// # Returns
    ///
    /// * `True` - Signal successful
    /// * `False` - Signal failed (max count reached)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Semaphore, SemaphoreFn};
    /// 
    /// let sem = Semaphore::new_with_count(0).unwrap();
    /// sem.signal();  // Notify waiting task
    /// ```
    fn signal(&self) -> OsalRsBool;
    
    /// Signals the semaphore from ISR context.
    ///
    /// ISR-safe version of `signal()`.
    ///
    /// # Returns
    ///
    /// * `True` - Signal successful
    /// * `False` - Signal failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In interrupt handler
    /// sem.signal_from_isr();
    /// ```
    fn signal_from_isr(&self) -> OsalRsBool;
    
    /// Deletes the semaphore and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are blocked on this semaphore before deletion.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut sem = Semaphore::new_with_count(1).unwrap();
    /// // ... use semaphore ...
    /// sem.delete();
    /// ```
    fn delete(&mut self);

}
