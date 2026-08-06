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

//! System-level RTOS control trait.
//!
//! Provides functions for scheduler control, timing, and system-wide operations.
//!
//! # Overview
//!
//! This module defines the `System` trait for RTOS-level operations including:
//! - Scheduler lifecycle management (start, stop, suspend/resume)
//! - Time and tick management
//! - Task delays and periodic execution
//! - Critical sections (task and ISR context)
//! - System state introspection
//! - Heap memory monitoring
//!
//! # Scheduler Control
//!
//! The scheduler must be started with `start()` after creating all initial tasks.
//! Once started, the scheduler runs indefinitely and `start()` does not return.
//!
//! # Timing
//!
//! The RTOS uses a tick-based timing system. The tick rate (typically 100Hz - 1000Hz)
//! determines the resolution of delays and timeouts.
//!
//! # Critical Sections
//!
//! Two types of critical sections are provided:
//! - **Task-level**: `critical_section_enter()` / `critical_section_exit()` - For protecting shared data between tasks
//! - **ISR-level**: `critical_section_enter_from_isr()` / `critical_section_exit_from_isr()` - For ISR context
//!
//! Critical sections should be kept as short as possible to minimize interrupt latency.

use core::time::Duration;

use crate::os::types::{BaseType, TickType, UBaseType};
use crate::os::SystemState;
use crate::utils::OsalRsBool;

/// System-level RTOS operations.
///
/// This trait provides static methods for controlling the RTOS scheduler,
/// managing system time, and performing system-wide operations.
///
/// # Method Categories
///
/// - **Scheduler**: `start()`, `stop()`, `suspend_all()`, `resume_all()`
/// - **Timing**: `get_tick_count()`, `get_current_time()`, `delay()`, `delay_until()`
/// - **Critical Sections**: `critical_section_enter()`, `critical_section_exit()`, ISR variants
/// - **System Info**: `count_threads()`, `get_all_thread()`, `get_free_heap_size()`
/// - **ISR Support**: `yield_from_isr()`, `end_switching_isr()`, ISR critical sections
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
///
/// // In a task:
/// System::delay(10);  // Delay for 10 ticks
///
/// // Critical section
/// System::critical_section_enter();
/// // Access shared data
/// System::critical_section_exit();
///
/// // Hand control over to the scheduler. On FreeRTOS this never returns; the
/// // POSIX backend returns once some task calls `System::stop()`.
/// let mut stopper = Thread::new("stopper", 1024, 1);
/// let worker = stopper.spawn_simple(|| {
///     System::delay(10);
///     System::stop();
///     Ok(Arc::new(()))
/// }).unwrap();
///
/// System::start();
///
/// worker.delete();
/// ```
pub trait System {
    /// Starts the RTOS scheduler.
    ///
    /// This function transfers control to the RTOS scheduler and does not return.
    /// After calling this, the scheduler begins executing the highest priority
    /// ready task.
    ///
    /// # Behavior
    ///
    /// - Enables interrupts and starts the system tick timer
    /// - Begins executing the highest priority task
    /// - Never returns to the caller
    ///
    /// # Prerequisites
    ///
    /// Before calling `start()`, you must:
    /// - Create at least one task with `Thread::new()` and `spawn()`
    /// - Initialize any required peripherals or resources
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// // Create tasks
    /// let mut task = Thread::new("main_task", 1024, 1);
    /// let worker = task.spawn_simple(|| {
    ///     System::delay(10);
    ///     // Task work
    ///
    ///     // Only the POSIX backend can be asked to hand control back.
    ///     System::stop();
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// // Start scheduler - on FreeRTOS this DOES NOT RETURN
    /// System::start();
    ///
    /// worker.delete();
    /// ```
    fn start();

    /// Suspends all tasks.
    ///
    /// Pauses the scheduler, preventing any task switches. The current task
    /// continues to execute but no context switches will occur. Calls can be
    /// nested; each `suspend_all()` must be paired with a `resume_all()`.
    ///
    /// # Use Cases
    ///
    /// - Performing time-critical operations without interruption
    /// - Accessing shared resources without locks (use sparingly)
    /// - Debugging scenarios
    ///
    /// # Warning
    ///
    /// Keep suspension periods as short as possible. Long suspensions
    /// can affect real-time behavior and task responsiveness.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut worker = Thread::new("worker", 1024, 1);
    /// worker.spawn_simple(|| {
    ///     System::delay(200);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// System::delay(10); // give it a moment to start running
    ///
    /// System::suspend_all();
    /// // Critical operations where task switches must not occur
    /// // Interrupts still occur but won't cause task switches
    /// System::delay(10);
    /// assert!(System::resume_all() >= 1);
    /// ```
    fn suspend_all();
    
    /// Resumes all tasks.
    ///
    /// Re-enables the scheduler after `suspend_all()`. If there were nested
    /// calls to `suspend_all()`, the scheduler resumes only when the nesting
    /// level returns to zero.
    ///
    /// # Returns
    ///
    /// Number of nested suspensions that were active before this call
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut worker = Thread::new("worker", 1024, 1);
    /// worker.spawn_simple(|| {
    ///     System::delay(200);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// System::delay(10); // give it a moment to start running
    ///
    /// System::suspend_all();
    /// // Protected operations
    /// System::delay(10);
    /// let nesting = System::resume_all();
    ///
    /// assert!(nesting >= 1);
    /// ```
    fn resume_all() -> BaseType;
    
    /// Stops the scheduler.
    ///
    /// Halts task scheduling permanently. Behavior is implementation-specific.
    /// Typically used for error handling or system shutdown.
    ///
    /// # Warning
    ///
    /// This may not be supported on all RTOS implementations. After calling
    /// this, the system may need to be reset to resume normal operation.
    fn stop();
    
    /// Gets the current system tick count.
    ///
    /// Returns the number of ticks since the scheduler started. The tick
    /// rate is configured at compile time (typically 100-1000 Hz).
    ///
    /// # Returns
    ///
    /// Current tick count (wraps around at `TickType::MAX`)
    ///
    /// # Overflow
    ///
    /// The tick count will eventually overflow. Use tick-count arithmetic
    /// that handles wrapping when calculating elapsed time.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let start = System::get_tick_count();
    ///
    /// // Perform work
    /// System::delay(20);
    ///
    /// let elapsed = System::get_tick_count().wrapping_sub(start);
    /// assert!(elapsed >= 20);
    /// ```
    fn get_tick_count() -> TickType;
    
    /// Gets current system time in microseconds.
    ///
    /// Returns a high-resolution timestamp based on the system tick count
    /// and any hardware timer available.
    ///
    /// # Returns
    ///
    /// Current time as `Duration` in microseconds
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let start = System::get_current_time();
    ///
    /// // Perform the operation being measured
    /// System::delay(20);
    ///
    /// let elapsed = System::get_current_time() - start;
    /// assert!(elapsed.as_micros() >= 20_000);
    /// ```
    fn get_current_time () -> Duration;
    
    /// Deprecated alias for [`System::get_current_time`]; kept for source
    /// compatibility with code written before the rename.
    #[deprecated(since = "1.0.4", note = "use `get_current_time` instead")]
    fn get_current_time_ms() -> Duration;

    /// Converts duration to tick count.
    ///
    /// Converts a `Duration` into the equivalent number of RTOS ticks.
    /// Useful when you need to work with tick-based APIs but have
    /// time expressed as a `Duration`.
    ///
    /// # Parameters
    ///
    /// * `duration` - The duration to convert
    ///
    /// # Returns
    ///
    /// Number of ticks equivalent to the duration (rounded)
    ///
    /// # Examples
    ///
    /// ```
    /// use core::time::Duration;
    /// use osal_rs::os::*;
    ///
    /// let duration = Duration::from_millis(20);
    /// let ticks = System::get_from_tick(&duration);
    /// assert!(ticks > 0);
    ///
    /// let before = System::get_tick_count();
    /// System::delay(ticks);
    /// assert!(System::get_tick_count() - before >= ticks);
    /// ```
    fn get_from_tick(duration: &Duration) -> TickType;

    /// Deprecated alias for [`System::get_from_tick`]; kept for source
    /// compatibility with code written before the rename.
    #[deprecated(since = "1.0.4", note = "use `get_from_tick` instead")]
    fn get_ms_from_tick(duration: &Duration) -> TickType;

    /// Gets the number of threads in the system.
    ///
    /// Returns the total count of all tasks/threads currently registered
    /// with the scheduler, including idle and system tasks.
    ///
    /// # Returns
    ///
    /// Count of all threads/tasks in the system
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let count = System::count_threads();
    ///
    /// // The calling thread is always part of the count.
    /// assert!(count >= 1);
    /// ```
    fn count_threads() -> usize;
    
    /// Gets information about all threads.
    ///
    /// Returns detailed information about all tasks in the system including
    /// names, priorities, states, and resource usage.
    ///
    /// # Returns
    ///
    /// System state containing thread metadata and statistics
    ///
    /// # Performance
    ///
    /// This operation may be expensive, especially with many tasks.
    /// Use sparingly in production code.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let sys_state = System::get_all_thread();
    ///
    /// for thread in &sys_state.tasks {
    ///     // Thread name, priority and state are all available here.
    ///     let _ = (thread.name.as_str(), thread.priority, thread.state);
    /// }
    ///
    /// // The calling thread is always in there.
    /// assert!(!sys_state.tasks.is_empty());
    /// ```
    fn get_all_thread() -> SystemState;
    
    /// Delays the calling task for specified ticks.
    ///
    /// Blocks the calling task for at least the specified number of ticks,
    /// allowing other tasks to run. The actual delay may be slightly longer
    /// due to scheduling granularity.
    ///
    /// # Parameters
    ///
    /// * `ticks` - Number of ticks to delay (minimum delay)
    ///
    /// # Blocking
    ///
    /// This function blocks the calling task. Do not call from ISR context.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let before = System::get_tick_count();
    ///
    /// for _ in 0..3 {
    ///     System::delay(10);  // Delay for 10 ticks
    ///     // perform the periodic task here
    /// }
    ///
    /// assert!(System::get_tick_count() - before >= 30);
    /// ```
    fn delay(ticks: TickType);
    
    /// Delays until an absolute time.
    ///
    /// Used for implementing periodic tasks with precise timing. Unlike `delay()`,
    /// which delays for a relative duration, this delays until a specific absolute
    /// tick count. This compensates for execution time and provides more accurate
    /// periodic execution.
    ///
    /// # Parameters
    ///
    /// * `previous_wake_time` - Last wake time (updated by function to next wake time)
    /// * `time_increment` - Period between wake times in ticks
    ///
    /// # Behavior
    ///
    /// The function calculates the next wake time as `previous_wake_time + time_increment`
    /// and delays until that time. This ensures consistent period even if task
    /// execution time varies.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let start = System::get_tick_count();
    /// let mut last_wake = start;
    ///
    /// for _ in 0..3 {
    ///     System::delay_until(&mut last_wake, 10);
    ///     // This runs exactly every 10 ticks regardless of execution time
    /// }
    ///
    /// // The wake-up times are anchored to the original tick, so they do not
    /// // drift by however long the work in the loop took.
    /// assert_eq!(last_wake, start + 30);
    /// ```
    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType);
    
    /// Checks if a timer has expired.
    ///
    /// Utility function to check if a specified duration has elapsed
    /// since a timestamp.
    ///
    /// # Parameters
    ///
    /// * `timestamp` - The starting time to check from
    /// * `time` - The timeout duration
    ///
    /// # Returns
    ///
    /// * `True` - The time period has expired
    /// * `False` - The time period has not yet expired
    ///
    /// # Examples
    ///
    /// ```
    /// use core::time::Duration;
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    ///
    /// let start = System::get_current_time();
    /// let timeout = Duration::from_millis(20);
    ///
    /// loop {
    ///     if System::check_timer(&start, &timeout) == OsalRsBool::True {
    ///         break; // Timer expired
    ///     }
    ///     // Do other work
    ///     System::delay(5);
    /// }
    ///
    /// assert!(System::get_current_time() - start >= timeout);
    /// ```
    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool;
    
    /// Yields to scheduler from ISR if needed.
    ///
    /// Requests a context switch from ISR context if a higher priority
    /// task has been woken by the ISR.
    ///
    /// # Parameters
    ///
    /// * `higher_priority_task_woken` - Flag indicating if context switch is needed
    ///   (non-zero value triggers yield)
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// // In ISR handler: set by the operations that may have woken a task.
    /// let higher_priority_woken = 1;
    ///
    /// // On the way out of the ISR, request the context switch.
    /// System::yield_from_isr(higher_priority_woken);
    ///
    /// // A zero flag means no task was woken, so this one is a no-op.
    /// System::yield_from_isr(0);
    /// ```
    fn yield_from_isr(higher_priority_task_woken: BaseType);
    
    /// Ends ISR with potential context switch.
    ///
    /// Marks the end of an ISR and performs a context switch if required.
    /// Some RTOS implementations require this to be called at the end of
    /// every ISR that interacts with RTOS primitives.
    ///
    /// # Parameters
    ///
    /// * `switch_required` - Flag indicating if context switch is required
    ///   (non-zero value triggers switch)
    fn end_switching_isr( switch_required: BaseType );
    
    /// Enters a critical section at task level.
    ///
    /// Disables scheduler and interrupts to protect shared resources.
    /// Must be paired with [`critical_section_exit()`](Self::critical_section_exit).
    /// This is the task-level version; for ISR context use
    /// [`critical_section_enter_from_isr()`](Self::critical_section_enter_from_isr).
    ///
    /// # Critical Section Behavior
    ///
    /// - Disables interrupts up to a configurable priority level
    /// - Prevents task switches
    /// - Can be nested (maintains nesting counter)
    ///
    /// # Performance Impact
    ///
    /// Critical sections increase interrupt latency. Keep them as short
    /// as possible - only a few microseconds ideally.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let mut shared_counter = 0;
    ///
    /// System::critical_section_enter();
    /// // Access shared resource safely
    /// shared_counter += 1;
    /// System::critical_section_exit();
    ///
    /// assert_eq!(shared_counter, 1);
    /// ```
    fn critical_section_enter();

    /// Exits a critical section at task level.
    ///
    /// Re-enables scheduler and interrupts after [`critical_section_enter()`](Self::critical_section_enter).
    /// Must be called from the same task that called `critical_section_enter()`.
    ///
    /// # Nesting
    ///
    /// If critical sections are nested, interrupts are only re-enabled
    /// when the outermost `critical_section_exit()` is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let mut shared_data = vec![1, 2, 3];
    ///
    /// System::critical_section_enter();
    /// // Critical section code
    /// shared_data.push(4);
    /// System::critical_section_exit();
    ///
    /// assert_eq!(shared_data.len(), 4);
    /// ```
    fn critical_section_exit();

    /// Enters a critical section from an ISR context.
    ///
    /// ISR-safe version of critical section entry. Returns the interrupt mask state
    /// that must be passed to [`critical_section_exit_from_isr()`](Self::critical_section_exit_from_isr).
    /// Use this instead of [`critical_section_enter()`](Self::critical_section_enter) when in interrupt context.
    ///
    /// # Returns
    ///
    /// Saved interrupt status that must be passed to `critical_section_exit_from_isr()`
    ///
    /// # ISR Safety
    ///
    /// This method is specifically designed for ISR context and preserves
    /// the interrupt state more accurately than the task-level version.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let mut shared_isr_data = 0;
    ///
    /// // In an interrupt handler
    /// let saved_status = System::critical_section_enter_from_isr();
    /// // Critical ISR code - access shared data
    /// shared_isr_data += 1;
    /// System::critical_section_exit_from_isr(saved_status);
    ///
    /// assert_eq!(shared_isr_data, 1);
    /// ```
    fn critical_section_enter_from_isr() -> UBaseType;

    /// Exits a critical section from an ISR context.
    ///
    /// Restores the interrupt mask to the state saved by
    /// [`critical_section_enter_from_isr()`](Self::critical_section_enter_from_isr).
    ///
    /// # Parameters
    ///
    /// * `saved_interrupt_status` - Interrupt status returned by `critical_section_enter_from_isr()`
    ///
    /// # Important
    ///
    /// Always pass the exact value returned by the matching `critical_section_enter_from_isr()`
    /// call. Using an incorrect value can lead to undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let mut shared_buffer = [0u8; 4];
    ///
    /// let saved = System::critical_section_enter_from_isr();
    /// // Protected ISR operations
    /// shared_buffer[0] = 0x42;
    /// System::critical_section_exit_from_isr(saved);
    ///
    /// assert_eq!(shared_buffer[0], 0x42);
    /// ```
    fn critical_section_exit_from_isr(saved_interrupt_status: UBaseType);

    /// Gets the amount of free heap memory.
    ///
    /// Returns the number of free bytes in the RTOS heap. Useful for
    /// monitoring memory usage and detecting memory leaks.
    ///
    /// # Returns
    ///
    /// Number of free bytes in the heap
    ///
    /// # Usage
    ///
    /// - Monitor memory usage during development
    /// - Implement low-memory handling strategies
    /// - Detect memory leaks by tracking over time
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let free = System::get_free_heap_size();
    /// assert!(free > 0);
    ///
    /// if free < 1024 {
    ///     // Warning: low memory - back off on allocations here.
    /// }
    /// ```
    fn get_free_heap_size() -> usize;
    
}
