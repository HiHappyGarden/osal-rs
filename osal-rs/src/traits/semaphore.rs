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

//! Semaphore trait for resource management and signaling.
//!
//! Provides counting semaphores for controlling access to shared resources
//! and coordinating task execution.
//!
//! # Overview
//!
//! Semaphores are synchronization primitives that maintain an internal counter.
//! Tasks can wait (decrement) or signal (increment) the counter. When the
//! counter reaches zero, waiting tasks block until another task signals.
//!
//! # Semaphore Types
//!
//! - **Binary Semaphore**: Counter limited to 0 or 1, used for signaling between tasks
//! - **Counting Semaphore**: Counter can exceed 1, used for resource pools
//!
//! # Common Use Cases
//!
//! - **Task Synchronization**: Signal task completion or events
//! - **Resource Pools**: Manage multiple identical resources (e.g., buffer pool)
//! - **Producer-Consumer**: Control flow between producer and consumer tasks
//! - **ISR to Task Communication**: Signal events from interrupt handlers
//!
//! # Semaphore vs Mutex
//!
//! - **Semaphore**: Any task can signal, used for signaling and counting
//! - **Mutex**: Must be released by the owner, used for mutual exclusion
//!
//! # Thread Safety
//!
//! All operations are thread-safe. ISR-specific methods should only be called
//! from interrupt context.

use crate::utils::OsalRsBool;
use super::ToTick;

/// Counting semaphore for resource management.
///
/// Semaphores maintain a count that can be incremented (signal) and
/// decremented (wait), useful for:
/// - Protecting shared resources with multiple instances
/// - Task synchronization and signaling
/// - Implementing resource pools
///
/// # Counter Behavior
///
/// - **Wait**: Decrements counter if > 0, otherwise blocks
/// - **Signal**: Increments counter up to maximum value
/// - Tasks block when counter is 0 during wait
///
/// # Examples
///
/// ## Binary Semaphore (Signaling)
///
/// ```
/// use osal_rs::os::*;
/// use osal_rs::utils::OsalRsBool;
/// use core::time::Duration;
/// use std::sync::Arc;
///
/// // Max count 1, initial count 0: a binary semaphore for signaling
/// let sem = Arc::new(Semaphore::new(1, 0).unwrap());
/// let producer = sem.clone();
///
/// // Task 2: Send signal
/// let mut thread = Thread::new("producer", 1024, 1);
/// let worker = thread.spawn_simple(move || {
///     System::delay(10);
///     producer.signal();
///     Ok(Arc::new(()))
/// }).unwrap();
///
/// // Task 1: Wait for signal
/// assert_eq!(sem.wait(Duration::from_secs(1)), OsalRsBool::True);
///
/// worker.delete();
/// ```
///
/// ## Counting Semaphore (Resource Pool)
///
/// ```
/// use osal_rs::os::*;
/// use osal_rs::utils::OsalRsBool;
/// use core::time::Duration;
///
/// // Pool of 3 resources
/// let pool = Semaphore::new(3, 3).unwrap();
///
/// // Acquire resource
/// if pool.wait(Duration::from_millis(100)) == OsalRsBool::True {
///     // Use resource...
///
///     // Release resource
///     assert_eq!(pool.signal(), OsalRsBool::True);
/// }
/// ```
pub trait Semaphore {

    /// Returns `true` if the underlying OS handle is null, i.e. the thread
    /// has not been spawned yet or has already been deleted.
    fn is_null(&self) -> bool;

    /// Waits to acquire the semaphore (blocking).
    ///
    /// Decrements the semaphore count if greater than zero. If the count
    /// is zero, blocks the calling task until another task signals or
    /// the timeout expires.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Maximum time to wait (accepts `Duration` or ticks):
    ///   - `Duration::ZERO` or `0`: Return immediately if unavailable
    ///   - `Duration` or ticks: Wait up to specified time
    ///   - `Duration::MAX` or `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `True` - Semaphore acquired successfully (count decremented)
    /// * `False` - Timeout occurred, semaphore not acquired
    ///
    /// # Blocking Behavior
    ///
    /// This method blocks the calling task. Do not call from ISR context.
    /// Use `wait_from_isr()` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    /// use core::time::Duration;
    ///
    /// let sem = Semaphore::new(1, 1).unwrap();
    ///
    /// // Wait with timeout
    /// if sem.wait(Duration::from_millis(100)) == OsalRsBool::True {
    ///     // Semaphore acquired, do work...
    ///     sem.signal();
    /// } else {
    ///     panic!("timeout waiting for semaphore");
    /// }
    ///
    /// // Take the single unit and keep it: the next wait finds the count at
    /// // 0 and gives up once the timeout expires.
    /// assert_eq!(sem.wait(Duration::from_millis(100)), OsalRsBool::True);
    /// assert_eq!(sem.wait(Duration::from_millis(10)), OsalRsBool::False);
    /// ```
    fn wait(&self, ticks_to_wait: impl ToTick) -> OsalRsBool;

    /// Waits to acquire from ISR context (non-blocking).
    ///
    /// ISR-safe version of `wait()`. Attempts to decrement the semaphore
    /// count without blocking. Returns immediately whether successful or not.
    ///
    /// # Returns
    ///
    /// * `True` - Semaphore acquired (count was > 0 and is now decremented)
    /// * `False` - Semaphore not available (count was 0)
    ///
    /// # ISR Safety
    ///
    /// This method must only be called from interrupt context. It never blocks.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    ///
    /// let sem = Semaphore::new(1, 1).unwrap();
    /// let mut missed_event_count = 0;
    ///
    /// // In interrupt handler
    /// if sem.wait_from_isr() == OsalRsBool::True {
    ///     // Semaphore acquired, process event quickly
    /// } else {
    ///     // Semaphore unavailable, skip or set flag
    ///     missed_event_count += 1;
    /// }
    ///
    /// // The count is 0 now, so this attempt is the one that gives up.
    /// assert_eq!(sem.wait_from_isr(), OsalRsBool::False);
    /// assert_eq!(missed_event_count, 0);
    /// ```
    fn wait_from_isr(&self) -> OsalRsBool;

    /// Signals (releases) the semaphore.
    ///
    /// Increments the semaphore count, potentially unblocking the highest
    /// priority task waiting on this semaphore. Unlike mutexes, any task
    /// can signal a semaphore.
    ///
    /// # Returns
    ///
    /// * `True` - Signal successful (count incremented)
    /// * `False` - Signal failed (maximum count already reached)
    ///
    /// # Behavior
    ///
    /// - If tasks are waiting, the highest priority task is unblocked
    /// - If no tasks are waiting, the count is incremented (up to max)
    /// - For binary semaphores (max=1), signaling when count=1 has no effect
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    /// use core::time::Duration;
    ///
    /// // Binary semaphore for signaling
    /// let sem = Semaphore::new(1, 0).unwrap();
    ///
    /// // Task 2 signals to unblock Task 1, which is waiting in `wait()`
    /// assert_eq!(sem.signal(), OsalRsBool::True);
    ///
    /// // Already at max_count: signalling again fails.
    /// assert_eq!(sem.signal(), OsalRsBool::False);
    ///
    /// // Counting semaphore for resource pool
    /// let pool = Semaphore::new(3, 3).unwrap();
    /// pool.wait(Duration::ZERO);  // Count: 3 -> 2
    /// pool.signal();              // Count: 2 -> 3
    ///
    /// // Back at max_count, so no unit can be handed back.
    /// assert_eq!(pool.signal(), OsalRsBool::False);
    /// ```
    fn signal(&self) -> OsalRsBool;
    
    /// Signals the semaphore from ISR context.
    ///
    /// ISR-safe version of `signal()`. Increments the semaphore count
    /// without blocking. Must only be called from interrupt context.
    ///
    /// # Returns
    ///
    /// * `True` - Signal successful (count incremented or task unblocked)
    /// * `False` - Signal failed (maximum count reached)
    ///
    /// # ISR Safety
    ///
    /// This method must only be called from interrupt context.
    ///
    /// # Common Pattern
    ///
    /// ISRs typically signal semaphores to notify tasks of events,
    /// deferring processing to task context.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    /// use core::time::Duration;
    ///
    /// let sem = Semaphore::new(1, 0).unwrap();
    ///
    /// // In interrupt handler - signal event occurred
    /// if sem.signal_from_isr() == OsalRsBool::True {
    ///     // Signal sent successfully
    /// }
    ///
    /// // In task context - wait for events
    /// assert_eq!(sem.wait(Duration::from_millis(100)), OsalRsBool::True);
    /// ```
    fn signal_from_isr(&self) -> OsalRsBool;
    
    /// Deletes the semaphore and frees its resources.
    ///
    /// # Safety
    ///
    /// Ensure no tasks are blocked waiting on this semaphore before deletion.
    /// Calling this while tasks are waiting may cause undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use core::time::Duration;
    ///
    /// let mut sem = Semaphore::new(1, 1).unwrap();
    ///
    /// // Use semaphore
    /// sem.wait(Duration::from_millis(100));
    /// sem.signal();
    ///
    /// // Clean up when done
    /// sem.delete();
    /// assert!(sem.is_null());
    /// ```
    fn delete(&mut self);

}
