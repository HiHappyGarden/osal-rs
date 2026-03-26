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
//! - **Task-level**: `enter_critical()` / `exit_critical()` - For protecting shared data between tasks
//! - **ISR-level**: `enter_critical_from_isr()` / `exit_critical_from_isr()` - For ISR context
//!
//! Critical sections should be kept as short as possible to minimize interrupt latency.

use core::time::Duration;

use crate::os::types::{BaseType, TickType, UBaseType};
use crate::os::{ThreadState};
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
/// - **Timing**: `get_tick_count()`, `get_current_time_us()`, `delay()`, `delay_until()`
/// - **Critical Sections**: `enter_critical()`, `exit_critical()`, ISR variants
/// - **System Info**: `get_state()`, `count_threads()`, `get_all_thread()`, `get_free_heap_size()`
/// - **ISR Support**: `yield_from_isr()`, `end_switching_isr()`, ISR critical sections
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::System;
/// 
/// // Start the scheduler (does not return)
/// System::start();
/// 
/// // In a task:
/// System::delay(100);  // Delay for 100 ticks
/// 
/// // Critical section
/// System::enter_critical();
/// // Access shared data
/// System::exit_critical();
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
    /// ```ignore
    /// use osal_rs::os::{System, Thread};
    /// 
    /// // Create tasks
    /// let mut task = Thread::new("main_task", 1024, 1);
    /// task.spawn_simple(|| {
    ///     loop {
    ///         System::delay(100);
    ///         // Task work
    ///     }
    /// }).ok();
    /// 
    /// // Start scheduler - DOES NOT RETURN
    /// System::start();
    /// 
    /// // This line is never reached
    /// ```
    fn start();
    
    /// Gets the current scheduler state.
    ///
    /// Returns the current operational state of the RTOS scheduler.
    ///
    /// # Returns
    ///
    /// The current state of the scheduler (e.g., Running, Suspended, NotStarted)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, ThreadState};
    /// 
    /// let state = System::get_state();
    /// match state {
    ///     ThreadState::Running => println!("Scheduler running"),
    ///     ThreadState::Suspended => println!("Scheduler suspended"),
    ///     _ => {}
    /// }
    /// ```
    fn get_state() -> ThreadState;
    
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// System::suspend_all();
    /// // Critical operations where task switches must not occur
    /// // Interrupts still occur but won't cause task switches
    /// System::resume_all();
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
    /// ```ignore
    /// System::suspend_all();
    /// // Protected operations
    /// let nesting = System::resume_all();
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// let start = System::get_tick_count();
    /// // Perform work
    /// do_some_work();
    /// let elapsed = System::get_tick_count().wrapping_sub(start);
    /// println!("Work took {} ticks", elapsed);
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// let start = System::get_current_time_us();
    /// perform_operation();
    /// let elapsed = System::get_current_time_us() - start;
    /// println!("Operation took {} µs", elapsed.as_micros());
    /// ```
    fn get_current_time_us () -> Duration;
    
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
    /// ```ignore
    /// use core::time::Duration;
    /// use osal_rs::os::System;
    /// 
    /// let duration = Duration::from_millis(100);
    /// let ticks = System::get_us_from_tick(&duration);
    /// System::delay(ticks);
    /// ```
    fn get_us_from_tick(duration: &Duration) -> TickType;
    
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// let count = System::count_threads();
    /// println!("System has {} threads", count);
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// let sys_state = System::get_all_thread();
    /// for thread in &sys_state.threads {
    ///     println!("Thread: {} Priority: {} State: {:?}",
    ///         thread.name, thread.priority, thread.state);
    /// }
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// loop {
    ///     System::delay(100);  // Delay for 100 ticks
    ///     perform_periodic_task();
    /// }
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// let mut last_wake = System::get_tick_count();
    /// loop {
    ///     System::delay_until(&mut last_wake, 100);
    ///     // This runs exactly every 100 ticks regardless of execution time
    ///     perform_periodic_task();
    /// }
    /// ```
    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType);
    
    /// Enters a critical section.
    ///
    /// Disables interrupts or scheduler to create an atomic section.
    /// Must be paired with `critical_section_exit()`. Keep critical
    /// sections as short as possible.
    ///
    /// # Warning
    ///
    /// This is a legacy method. Prefer using `enter_critical()` /
    /// `exit_critical()` for task context.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// System::critical_section_enter();
    /// // Critical code - no interrupts/context switches
    /// unsafe_shared_operation();
    /// System::critical_section_exit();
    /// ```
    fn critical_section_enter();
    
    /// Exits a critical section.
    ///
    /// Re-enables interrupts/scheduling after `critical_section_enter()`.
    /// Must be called from the same context that called `critical_section_enter()`.
    fn critical_section_exit();
    
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
    /// ```ignore
    /// use core::time::Duration;
    /// use osal_rs::os::System;
    /// 
    /// let start = System::get_current_time_us();
    /// let timeout = Duration::from_millis(100);
    /// 
    /// loop {
    ///     if System::check_timer(&start, &timeout).into() {
    ///         println!("Timer expired!");
    ///         break;
    ///     }
    ///     // Do other work
    /// }
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
    /// ```ignore
    /// // In ISR handler
    /// let mut higher_priority_woken = 0;
    /// // ISR operations that may wake tasks...
    /// System::yield_from_isr(higher_priority_woken);
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
    /// Must be paired with [`exit_critical()`](Self::exit_critical).
    /// This is the task-level version; for ISR context use
    /// [`enter_critical_from_isr()`](Self::enter_critical_from_isr).
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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// System::enter_critical();
    /// // Access shared resource safely
    /// shared_counter += 1;
    /// System::exit_critical();
    /// ```
    fn enter_critical();

    /// Exits a critical section at task level.
    ///
    /// Re-enables scheduler and interrupts after [`enter_critical()`](Self::enter_critical).
    /// Must be called from the same task that called `enter_critical()`.
    ///
    /// # Nesting
    ///
    /// If critical sections are nested, interrupts are only re-enabled
    /// when the outermost `exit_critical()` is called.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// System::enter_critical();
    /// // Critical section code
    /// shared_data.update();
    /// System::exit_critical();
    /// ```
    fn exit_critical();

    /// Enters a critical section from an ISR context.
    ///
    /// ISR-safe version of critical section entry. Returns the interrupt mask state
    /// that must be passed to [`exit_critical_from_isr()`](Self::exit_critical_from_isr).
    /// Use this instead of [`enter_critical()`](Self::enter_critical) when in interrupt context.
    ///
    /// # Returns
    ///
    /// Saved interrupt status that must be passed to `exit_critical_from_isr()`
    ///
    /// # ISR Safety
    ///
    /// This method is specifically designed for ISR context and preserves
    /// the interrupt state more accurately than the task-level version.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// // In an interrupt handler
    /// let saved_status = System::enter_critical_from_isr();
    /// // Critical ISR code - access shared data
    /// shared_isr_data.update();
    /// System::exit_critical_from_isr(saved_status);
    /// ```
    fn enter_critical_from_isr() -> UBaseType;

    /// Exits a critical section from an ISR context.
    ///
    /// Restores the interrupt mask to the state saved by
    /// [`enter_critical_from_isr()`](Self::enter_critical_from_isr).
    ///
    /// # Parameters
    ///
    /// * `saved_interrupt_status` - Interrupt status returned by `enter_critical_from_isr()`
    ///
    /// # Important
    ///
    /// Always pass the exact value returned by the matching `enter_critical_from_isr()`
    /// call. Using an incorrect value can lead to undefined behavior.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// let saved = System::enter_critical_from_isr();
    /// // Protected ISR operations
    /// update_shared_buffer();
    /// System::exit_critical_from_isr(saved);
    /// ```
    fn exit_critical_from_isr(saved_interrupt_status: UBaseType);

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
    /// ```ignore
    /// use osal_rs::os::System;
    /// 
    /// let free = System::get_free_heap_size();
    /// println!("Free heap: {} bytes", free);
    /// 
    /// if free < 1024 {
    ///     println!("Warning: Low memory!");
    /// }
    /// ```
    fn get_free_heap_size() -> usize;
    
}
