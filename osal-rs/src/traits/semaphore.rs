/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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
/// ```ignore
/// use osal_rs::os::Semaphore;
/// use core::time::Duration;
/// 
/// // Create with count 0 (binary semaphore for signaling)
/// let sem = Semaphore::new_with_count(0).unwrap();
/// 
/// // Task 1: Wait for signal
/// if sem.wait(Duration::from_secs(1)).into() {
///     println!("Signal received!");
/// }
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
///     process_with_resource();
///     
///     // Release resource
///     pool.signal();
/// }
/// ```
pub trait Semaphore {

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
    /// ```ignore
    /// use osal_rs::os::Semaphore;
    /// use core::time::Duration;
    /// 
    /// let sem = Semaphore::new_with_count(1).unwrap();
    /// 
    /// // Wait with timeout
    /// if sem.wait(Duration::from_millis(100)).into() {
    ///     // Semaphore acquired, do work
    ///     process_critical_section();
    ///     sem.signal();
    /// } else {
    ///     println!("Timeout waiting for semaphore");
    /// }
    /// 
    /// // Wait forever
    /// sem.wait(Duration::MAX);
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
    /// ```ignore
    /// // In interrupt handler
    /// if sem.wait_from_isr().into() {
    ///     // Semaphore acquired, process event quickly
    ///     handle_event();
    /// } else {
    ///     // Semaphore unavailable, skip or set flag
    ///     missed_event_count += 1;
    /// }
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
    /// ```ignore
    /// use osal_rs::os::Semaphore;
    /// 
    /// // Binary semaphore for signaling
    /// let sem = Semaphore::new_with_count(0).unwrap();
    /// 
    /// // Task 1 is waiting on sem.wait()
    /// // Task 2 signals to unblock Task 1
    /// sem.signal();  // Unblocks Task 1
    /// 
    /// // Counting semaphore for resource pool
    /// let pool = Semaphore::new(3, 3).unwrap();
    /// pool.wait(Duration::ZERO);  // Count: 3 -> 2
    /// pool.signal();              // Count: 2 -> 3
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
    /// ```ignore
    /// // In interrupt handler - signal event occurred
    /// if sem.signal_from_isr().into() {
    ///     // Signal sent successfully
    /// }
    /// 
    /// // In task context - wait for events
    /// loop {
    ///     if sem.wait(Duration::MAX).into() {
    ///         // Event received from ISR, process it
    ///         handle_isr_event();
    ///     }
    /// }
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
    /// ```ignore
    /// let mut sem = Semaphore::new_with_count(1).unwrap();
    /// 
    /// // Use semaphore
    /// sem.wait(Duration::from_millis(100));
    /// sem.signal();
    /// 
    /// // Clean up when done
    /// sem.delete();
    /// ```
    fn delete(&mut self);

}
