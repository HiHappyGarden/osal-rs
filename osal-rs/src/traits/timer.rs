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

//! Software timer trait for delayed and periodic callbacks.
//!
//! Timers execute callback functions in the context of a timer service task,
//! enabling delayed operations and periodic tasks without dedicated threads.
//!
//! # Overview
//!
//! Software timers provide a way to execute callback functions at specified
//! intervals without creating dedicated tasks. All timer callbacks run in
//! the context of a single timer service daemon task.
//!
//! # Timer Types
//!
//! - **One-shot**: Expires once after the period elapses
//! - **Auto-reload (Periodic)**: Automatically restarts after expiring
//!
//! # Timer Service Task
//!
//! All timer callbacks execute in a dedicated timer service task that:
//! - Has a configurable priority
//! - Processes timer commands from a queue
//! - Executes callbacks sequentially (not in parallel)
//!
//! # Important Constraints
//!
//! - Timer callbacks should be short and non-blocking
//! - Callbacks should not call blocking RTOS APIs (may cause deadlock)
//! - Long callbacks delay other timer expirations
//! - Use task notifications or queues to defer work to other tasks
//!
//! # Accuracy
//!
//! Timer accuracy depends on:
//! - System tick rate (e.g., 1ms for 1000 Hz)
//! - Timer service task priority
//! - Duration of other timer callbacks
//! - System load
//!
//! # Examples
//!
//! ```
//! use osal_rs::os::*;
//! use std::sync::Arc;
//! use core::time::Duration;
//!
//! // One-shot timer
//! let once = Timer::new_with_to_tick(
//!     "timeout",
//!     Duration::from_millis(50),
//!     false,  // Not auto-reload
//!     None,
//!     |_timer, _param| {
//!         println!("Timeout!");
//!         Ok(Arc::new(()))
//!     }
//! ).unwrap();
//! once.start(0);
//!
//! // Periodic timer
//! let periodic = Timer::new_with_to_tick(
//!     "heartbeat",
//!     Duration::from_millis(500),
//!     true,  // Auto-reload
//!     None,
//!     |_timer, _param| {
//!         println!("Blink!");
//!         Ok(Arc::new(()))
//!     }
//! ).unwrap();
//! periodic.start(0);
//! ```

use core::any::Any;

use alloc::{boxed::Box, sync::Arc};

use crate::os::types::TickType;
use crate::utils::{OsalRsBool, Result};

/// Type-erased parameter for timer callbacks.
///
/// Allows passing arbitrary data to timer callback functions in a type-safe
/// manner. The parameter is wrapped in an `Arc` for safe sharing and can be
/// downcast to its original type.
///
/// # Thread Safety
///
/// The inner type must implement `Any + Send + Sync` since timer callbacks
/// execute in the timer service task context.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
///
/// // Create a parameter
/// let param: TimerParam = Arc::new(7u32);
///
/// // In the timer callback, downcast to access it
/// let count = param.downcast_ref::<u32>().copied();
/// assert_eq!(count, Some(7));
///
/// // A downcast to the wrong type simply reports `None`.
/// assert!(param.downcast_ref::<i8>().is_none());
/// ```
pub type TimerParam = Arc<dyn Any + Send + Sync>;

/// Timer callback function pointer type.
///
/// Callbacks receive the timer handle and optional parameter,
/// and can return an updated parameter value.
///
/// # Parameters
///
/// - `Box<dyn Timer>` - Handle to the timer that expired
/// - `Option<TimerParam>` - Optional parameter passed at creation
///
/// # Returns
///
/// `Result<TimerParam>` - Updated parameter or error
///
/// # Execution Context
///
/// Callbacks execute in the timer service task, not ISR context.
/// They should be short and avoid blocking operations.
///
/// # Trait Bounds
///
/// The function must be `Send + Sync + 'static` to safely execute
/// in the timer service task.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
///
/// let callback: Box<TimerFnPtr> = Box::new(|_timer, param| {
///     if let Some(p) = param {
///         if let Some(count) = p.downcast_ref::<u32>() {
///             // Timer expired: hand the next invocation an updated count.
///             return Ok(Arc::new(*count + 1));
///         }
///     }
///     Ok(Arc::new(0u32))
/// });
///
/// // This is what the timer service does on every expiration: it passes the
/// // expired timer and the current parameter, and keeps whatever comes back.
/// let timer = Timer::new("counter", 50, true, None, |_t, _p| Ok(Arc::new(()))).unwrap();
/// let updated = callback(Box::new(timer), Some(Arc::new(41u32))).unwrap();
///
/// assert_eq!(updated.downcast_ref::<u32>(), Some(&42));
/// ```
pub type TimerFnPtr = dyn Fn(Box<dyn Timer>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + 'static;

/// Software timer for delayed and periodic callbacks.
///
/// Timers run callbacks in the timer service task context, not ISR context.
/// They can be one-shot or auto-reloading (periodic).
///
/// # Timer Lifecycle
///
/// 1. **Creation**: `Timer::new()` with name, period, auto-reload flag, and callback
/// 2. **Start**: `start()` begins the timer countdown
/// 3. **Expiration**: Callback executes when period elapses
/// 4. **Auto-reload**: If enabled, timer automatically restarts
/// 5. **Management**: Use `stop()`, `reset()`, `change_period()` to control
/// 6. **Cleanup**: `delete()` frees resources
///
/// # Command Queue
///
/// Timer operations (start, stop, etc.) send commands to a queue processed
/// by the timer service task. The `ticks_to_wait` parameter controls how
/// long to wait if the queue is full.
///
/// # Callback Constraints
///
/// - Keep callbacks short (< 1ms ideally)
/// - Avoid blocking operations (delays, mutex waits, etc.)
/// - Don't call APIs that might block indefinitely
/// - Use task notifications or queues to defer work to tasks
///
/// # Examples
///
/// ## One-shot Timer
///
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicU32, Ordering};
/// use core::time::Duration;
///
/// static ALARMS: AtomicU32 = AtomicU32::new(0);
///
/// let timer = Timer::new_with_to_tick(
///     "alarm",
///     Duration::from_millis(20),
///     false,  // One-shot
///     None,
///     |_timer, _param| {
///         ALARMS.fetch_add(1, Ordering::SeqCst);
///         Ok(Arc::new(()))
///     }
/// ).unwrap();
///
/// timer.start(0);
///
/// // Expires once, then stays quiet however long we wait.
/// System::delay(120);
/// assert_eq!(ALARMS.load(Ordering::SeqCst), 1);
/// ```
///
/// ## Periodic Timer
///
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicU32, Ordering};
/// use core::time::Duration;
///
/// // The parameter is handed to each invocation and replaced by whatever
/// // that invocation returns, so the count carries across expirations.
/// let counter: TimerParam = Arc::new(AtomicU32::new(0));
///
/// let periodic = Timer::new_with_to_tick(
///     "counter",
///     Duration::from_millis(10),
///     true,  // Auto-reload
///     Some(counter.clone()),
///     |_timer, param| {
///         let param = param.unwrap();
///         if let Some(count) = param.downcast_ref::<AtomicU32>() {
///             count.fetch_add(1, Ordering::SeqCst);
///         }
///         Ok(param)
///     }
/// ).unwrap();
///
/// periodic.start(0);
///
/// // Runs every 10ms until stopped
/// System::delay(120);
/// periodic.stop(0);
///
/// assert!(counter.downcast_ref::<AtomicU32>().unwrap().load(Ordering::SeqCst) > 1);
/// ```
pub trait Timer {

    /// Returns `true` if the underlying OS handle is null, i.e. the mutex
    /// has not been created yet or has already been deleted.
    fn is_null(&self) -> bool;


    /// Starts or restarts the timer.
    ///
    /// If the timer is already running, this command resets it to its full
    /// period (equivalent to calling `reset()`). If stopped, the timer begins
    /// counting down from its period.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Maximum ticks to wait if command queue is full:
    ///   - `0`: Return immediately if queue full
    ///   - `n`: Wait up to n ticks
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully to timer service
    /// * `False` - Failed to send command (queue full, timeout)
    ///
    /// # Timing
    ///
    /// The timer begins counting after the command is processed by the
    /// timer service task, not immediately when this function returns.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static FIRED: AtomicBool = AtomicBool::new(false);
    ///
    /// let timer = Timer::new("alarm", 20, false, None, |_timer, _param| {
    ///     FIRED.store(true, Ordering::SeqCst);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// // Start immediately, don't wait
    /// assert_eq!(timer.start(0), OsalRsBool::True);
    ///
    /// // Restarting it resets the countdown to the full period
    /// timer.start(100); // wait up to 100 ticks for the command queue
    ///
    /// System::delay(120);
    /// assert!(FIRED.load(Ordering::SeqCst));
    /// ```
    fn start(&self, ticks_to_wait: TickType) -> OsalRsBool;
    
    /// Stops the timer.
    ///
    /// The timer will not expire until started again with `start()` or `reset()`.
    /// For periodic timers, this stops the automatic reloading.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Maximum ticks to wait if command queue is full:
    ///   - `0`: Return immediately if queue full
    ///   - `n`: Wait up to n ticks
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully to timer service
    /// * `False` - Failed to send command (queue full, timeout)
    ///
    /// # State
    ///
    /// If the timer is already stopped, this command has no effect but
    /// still returns `True`.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    ///
    /// static FIRINGS: AtomicU32 = AtomicU32::new(0);
    ///
    /// let timer = Timer::new("heartbeat", 20, true, None, |_timer, _param| {
    ///     FIRINGS.fetch_add(1, Ordering::SeqCst);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// timer.start(0);
    ///
    /// // Stop the timer, wait up to 100 ticks
    /// assert_eq!(timer.stop(100), OsalRsBool::True);
    ///
    /// // Nothing fires while it is stopped.
    /// let stopped_at = FIRINGS.load(Ordering::SeqCst);
    /// System::delay(60);
    /// assert_eq!(FIRINGS.load(Ordering::SeqCst), stopped_at);
    ///
    /// // Later, restart it
    /// timer.start(100);
    /// System::delay(60);
    /// assert!(FIRINGS.load(Ordering::SeqCst) > stopped_at);
    /// ```
    fn stop(&self, ticks_to_wait: TickType)  -> OsalRsBool;
    
    /// Resets the timer to its full period.
    ///
    /// If the timer is running, this restarts it from the beginning of its
    /// period. If the timer is stopped, this starts it. This is useful for
    /// implementing watchdog-style timers that must be periodically reset.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Maximum ticks to wait if command queue is full:
    ///   - `0`: Return immediately if queue full
    ///   - `n`: Wait up to n ticks
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully to timer service
    /// * `False` - Failed to send command (queue full, timeout)
    ///
    /// # Use Cases
    ///
    /// - Watchdog timer: Reset timer to prevent timeout
    /// - Activity timer: Reset when activity detected
    /// - Timeout extension: Give more time before expiration
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    /// use core::time::Duration;
    ///
    /// static TIMED_OUT: AtomicBool = AtomicBool::new(false);
    ///
    /// // Watchdog timer pattern
    /// let watchdog = Timer::new_with_to_tick(
    ///     "watchdog",
    ///     Duration::from_millis(50),
    ///     false,
    ///     None,
    ///     |_timer, _param| {
    ///         TIMED_OUT.store(true, Ordering::SeqCst);
    ///         Ok(Arc::new(()))
    ///     }
    /// ).unwrap();
    ///
    /// watchdog.start(0);
    ///
    /// // In main loop: reset watchdog to prevent timeout
    /// for _ in 0..5 {
    ///     System::delay(20); // do work
    ///     watchdog.reset(0); // "Feed" the watchdog
    /// }
    ///
    /// // Fed often enough, it never expired.
    /// assert!(!TIMED_OUT.load(Ordering::SeqCst));
    ///
    /// // Stop feeding it and it fires.
    /// System::delay(120);
    /// assert!(TIMED_OUT.load(Ordering::SeqCst));
    /// ```
    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool;
    
    /// Changes the timer period.
    ///
    /// Updates the timer period. The new period takes effect immediately:
    /// - If the timer is running, it continues with the new period
    /// - The remaining time is adjusted proportionally
    /// - For periodic timers, future expirations use the new period
    ///
    /// # Parameters
    ///
    /// * `new_period_in_ticks` - New timer period in ticks
    /// * `ticks_to_wait` - Maximum ticks to wait if command queue is full:
    ///   - `0`: Return immediately if queue full
    ///   - `n`: Wait up to n ticks
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully to timer service
    /// * `False` - Failed to send command (queue full, timeout)
    ///
    /// # Behavior
    ///
    /// - If timer has already expired and is auto-reload, the new period
    ///   applies to the next expiration
    /// - If timer is stopped, the new period will be used when started
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    /// use core::time::Duration;
    ///
    /// static FIRED: AtomicBool = AtomicBool::new(false);
    ///
    /// let timer = Timer::new_with_to_tick(
    ///     "adaptive",
    ///     Duration::from_secs(10),
    ///     true,
    ///     None,
    ///     |_timer, _param| {
    ///         FIRED.store(true, Ordering::SeqCst);
    ///         Ok(Arc::new(()))
    ///     }
    /// ).unwrap();
    ///
    /// timer.start(0);
    ///
    /// // Later, adjust the period based on system load. The new period takes
    /// // effect immediately: the original 10s would never elapse in time here.
    /// let system_busy = false;
    /// if system_busy {
    ///     timer.change_period(500, 100); // slow down to 500ms
    /// } else {
    ///     timer.change_period(10, 100);  // speed up to 10ms
    /// }
    ///
    /// System::delay(60);
    /// assert!(FIRED.load(Ordering::SeqCst));
    /// ```
    fn change_period(&self, new_period_in_ticks: TickType, ticks_to_wait: TickType) -> OsalRsBool;
    
    /// Deletes the timer and frees its resources.
    ///
    /// Terminates the timer and releases its resources. After deletion,
    /// the timer handle becomes invalid and should not be used.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Maximum ticks to wait if command queue is full:
    ///   - `0`: Return immediately if queue full
    ///   - `n`: Wait up to n ticks
    ///   - `TickType::MAX`: Wait forever
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully to timer service
    /// * `False` - Failed to send command (queue full, timeout)
    ///
    /// # Safety
    ///
    /// - The timer should be stopped before deletion (recommended)
    /// - Do not use the timer handle after calling this
    /// - The timer is deleted asynchronously by the timer service task
    ///
    /// # Best Practice
    ///
    /// Stop the timer before deleting it to ensure clean shutdown:
    ///
    /// ```
    /// # use osal_rs::os::*;
    /// # use std::sync::Arc;
    /// # let mut timer = Timer::new("temporary", 50, false, None, |_t, _p| Ok(Arc::new(()))).unwrap();
    /// timer.stop(100);
    /// timer.delete(100);
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    /// use std::sync::Arc;
    /// use core::time::Duration;
    ///
    /// let mut timer = Timer::new_with_to_tick(
    ///     "temporary",
    ///     Duration::from_secs(1),
    ///     false,
    ///     None,
    ///     |_timer, _param| Ok(Arc::new(()))
    /// ).unwrap();
    ///
    /// timer.start(0);
    /// // ... use timer ...
    ///
    /// // Clean shutdown
    /// timer.stop(100);
    /// assert_eq!(timer.delete(100), OsalRsBool::True);
    /// assert!(timer.is_null());
    /// ```
    fn delete(&mut self, ticks_to_wait: TickType) -> OsalRsBool;
}