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
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
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
//! ```ignore
//! use osal_rs::os::Timer;
//! use core::time::Duration;
//!
//! // One-shot timer
//! let once = Timer::new(
//!     "timeout",
//!     Duration::from_secs(5),
//!     false,  // Not auto-reload
//!     None,
//!     |_timer, _param| {
//!         println!("Timeout!");
//!         Ok(None)
//!     }
//! ).unwrap();
//! once.start(0);
//!
//! // Periodic timer
//! let periodic = Timer::new(
//!     "heartbeat",
//!     Duration::from_millis(500),
//!     true,  // Auto-reload
//!     None,
//!     |_timer, _param| {
//!         toggle_led();
//!         Ok(None)
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
/// ```ignore
/// use std::sync::Arc;
/// use osal_rs::traits::TimerParam;
///
/// // Create a parameter
/// let count: TimerParam = Arc::new(0u32);
///
/// // In timer callback, downcast to access
/// if let Some(value) = param.downcast_ref::<u32>() {
///     println!("Count: {}", value);
/// }
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
/// ```ignore
/// use osal_rs::traits::{Timer, TimerParam};
/// use std::sync::Arc;
///
/// let callback: Box<TimerFnPtr> = Box::new(|timer, param| {
///     if let Some(p) = param {
///         if let Some(count) = p.downcast_ref::<u32>() {
///             println!("Timer expired, count: {}", count);
///             return Ok(Arc::new(*count + 1));
///         }
///     }
///     Ok(Arc::new(0u32))
/// });
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
/// ```ignore
/// use osal_rs::os::Timer;
/// use core::time::Duration;
/// 
/// let timer = Timer::new(
///     "alarm",
///     Duration::from_secs(5),
///     false,  // One-shot
///     None,
///     |_timer, _param| {
///         println!("Alarm!");
///         trigger_alarm();
///         Ok(None)
///     }
/// ).unwrap();
/// 
/// timer.start(0);
/// // Expires once after 5 seconds
/// ```
///
/// ## Periodic Timer
///
/// ```ignore
/// use std::sync::Arc;
///
/// let counter = Arc::new(0u32);
/// let periodic = Timer::new(
///     "counter",
///     Duration::from_millis(100),
///     true,  // Auto-reload
///     Some(counter.clone()),
///     |_timer, param| {
///         if let Some(p) = param {
///             if let Some(count) = p.downcast_ref::<u32>() {
///                 println!("Count: {}", count);
///                 return Ok(Arc::new(*count + 1));
///             }
///         }
///         Ok(Arc::new(0u32))
///     }
/// ).unwrap();
/// 
/// periodic.start(0);
/// // Runs every 100ms until stopped
/// ```
pub trait Timer {
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
    /// ```ignore
    /// use osal_rs::os::Timer;
    ///
    /// // Start immediately, don't wait
    /// if timer.start(0).into() {
    ///     println!("Timer started");
    /// }
    ///
    /// // Wait up to 100 ticks for command queue
    /// timer.start(100);
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
    /// ```ignore
    /// use osal_rs::os::Timer;
    ///
    /// // Stop the timer, wait up to 100 ticks
    /// if timer.stop(100).into() {
    ///     println!("Timer stopped");
    /// }
    ///
    /// // Later, restart it
    /// timer.start(100);
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
    /// ```ignore
    /// use osal_rs::os::Timer;
    /// use core::time::Duration;
    ///
    /// // Watchdog timer pattern
    /// let watchdog = Timer::new(
    ///     "watchdog",
    ///     Duration::from_secs(10),
    ///     false,
    ///     None,
    ///     |_timer, _param| {
    ///         println!("WATCHDOG TIMEOUT!");
    ///         system_reset();
    ///         Ok(None)
    ///     }
    /// ).unwrap();
    ///
    /// watchdog.start(0);
    ///
    /// // In main loop: reset watchdog to prevent timeout
    /// loop {
    ///     do_work();
    ///     watchdog.reset(0);  // "Feed" the watchdog
    /// }
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
    /// ```ignore
    /// use osal_rs::os::Timer;
    /// use core::time::Duration;
    ///
    /// let timer = Timer::new(
    ///     "adaptive",
    ///     Duration::from_millis(100),
    ///     true,
    ///     None,
    ///     |_timer, _param| Ok(None)
    /// ).unwrap();
    ///
    /// timer.start(0);
    ///
    /// // Later, adjust the period based on system load
    /// if system_busy() {
    ///     // Slow down to 500ms
    ///     timer.change_period(500, 100);
    /// } else {
    ///     // Speed up to 100ms
    ///     timer.change_period(100, 100);
    /// }
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
    /// ```ignore
    /// timer.stop(100);
    /// timer.delete(100);
    /// ```
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Timer;
    /// use core::time::Duration;
    ///
    /// let mut timer = Timer::new(
    ///     "temporary",
    ///     Duration::from_secs(1),
    ///     false,
    ///     None,
    ///     |_timer, _param| Ok(None)
    /// ).unwrap();
    ///
    /// timer.start(0);
    /// // ... use timer ...
    ///
    /// // Clean shutdown
    /// timer.stop(100);
    /// if timer.delete(100).into() {
    ///     println!("Timer deleted");
    /// }
    /// ```
    fn delete(&mut self, ticks_to_wait: TickType) -> OsalRsBool;
}