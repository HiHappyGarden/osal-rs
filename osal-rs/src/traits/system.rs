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

//! System-level RTOS control trait.
//!
//! Provides functions for scheduler control, timing, and system-wide operations.

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
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// 
/// // Start the scheduler
/// System::start();
/// 
/// // In a task:
/// System::delay(100);  // Delay for 100 ticks
/// ```
pub trait System {
    /// Starts the RTOS scheduler.
    ///
    /// This function does not return - it transfers control to the RTOS
    /// scheduler which then begins executing tasks.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn, Thread, ThreadFn};
    /// 
    /// // Create tasks
    /// let mut task = Thread::new("task", 1024, 1);
    /// task.spawn_simple(|| {
    ///     loop {
    ///         // Task code
    ///     }
    /// }).ok();
    /// 
    /// // Start scheduler (does not return)
    /// System::start();
    /// ```
    fn start();
    
    /// Gets the current scheduler state.
    ///
    /// # Returns
    ///
    /// The current state of the scheduler
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let state = System::get_state();
    /// ```
    fn get_state() -> ThreadState;
    
    /// Suspends all tasks.
    ///
    /// Pauses the scheduler, preventing any task switches.
    /// Must be paired with `resume_all()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// System::suspend_all();
    /// // Critical operations where task switches must not occur
    /// System::resume_all();
    /// ```
    fn suspend_all();
    
    /// Resumes all tasks.
    ///
    /// Re-enables the scheduler after `suspend_all()`.
    ///
    /// # Returns
    ///
    /// Number of nested suspensions that were active
    fn resume_all() -> BaseType;
    
    /// Stops the scheduler.
    ///
    /// Halts task scheduling. Behavior is implementation-specific.
    fn stop();
    
    /// Gets the current system tick count.
    ///
    /// Returns the number of ticks since the scheduler started.
    ///
    /// # Returns
    ///
    /// Current tick count
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let start = System::get_tick_count();
    /// // ... do work ...
    /// let elapsed = System::get_tick_count() - start;
    /// ```
    fn get_tick_count() -> TickType;
    
    /// Gets current system time in microseconds.
    ///
    /// # Returns
    ///
    /// Current time as `Duration`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let now = System::get_current_time_us();
    /// ```
    fn get_current_time_us () -> Duration;
    
    /// Converts duration to tick count.
    ///
    /// # Parameters
    ///
    /// * `duration` - The duration to convert
    ///
    /// # Returns
    ///
    /// Number of ticks equivalent to the duration
    fn get_us_from_tick(duration: &Duration) -> TickType;
    
    /// Gets the number of threads in the system.
    ///
    /// # Returns
    ///
    /// Count of all threads/tasks
    fn count_threads() -> usize;
    
    /// Gets information about all threads.
    ///
    /// # Returns
    ///
    /// System state containing thread metadata
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let sys_state = System::get_all_thread();
    /// for thread in &sys_state.threads {
    ///     println!("Thread: {}", thread.name);
    /// }
    /// ```
    fn get_all_thread() -> SystemState;
    
    /// Delays the calling task for specified ticks.
    ///
    /// Blocks the calling task for at least the specified number of ticks.
    ///
    /// # Parameters
    ///
    /// * `ticks` - Number of ticks to delay
    ///
    /// # Examples
    ///
    /// ```ignore
    /// System::delay(100);  // Delay for 100 ticks
    /// ```
    fn delay(ticks: TickType);
    
    /// Delays until an absolute time.
    ///
    /// Used for periodic execution with precise timing.
    ///
    /// # Parameters
    ///
    /// * `previous_wake_time` - Last wake time (updated by function)
    /// * `time_increment` - Period between wake times
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut last_wake = System::get_tick_count();
    /// loop {
    ///     System::delay_until(&mut last_wake, 100);
    ///     // Runs every 100 ticks
    /// }
    /// ```
    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType);
    
    /// Enters a critical section.
    ///
    /// Disables interrupts/scheduling. Must be paired with `critical_section_exit()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// System::critical_section_enter();
    /// // Critical code - no interrupts/context switches
    /// System::critical_section_exit();
    /// ```
    fn critical_section_enter();
    
    /// Exits a critical section.
    ///
    /// Re-enables interrupts/scheduling after `critical_section_enter()`.
    fn critical_section_exit();
    
    /// Checks if a timer has expired.
    ///
    /// # Parameters
    ///
    /// * `timestamp` - The time to check
    /// * `time` - The timeout duration
    ///
    /// # Returns
    ///
    /// `True` if time has expired, `False` otherwise
    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool;
    
    /// Yields to scheduler from ISR if needed.
    ///
    /// # Parameters
    ///
    /// * `higher_priority_task_woken` - Flag indicating context switch needed
    fn yield_from_isr(higher_priority_task_woken: BaseType);
    
    /// Ends ISR with potential context switch.
    ///
    /// # Parameters
    ///
    /// * `switch_required` - Flag indicating if context switch is required
    fn end_switching_isr( switch_required: BaseType );
    
    /// Enters a critical section at task level.
    ///
    /// Disables scheduler and interrupts to protect shared resources.
    /// Must be paired with [`exit_critical()`](Self::exit_critical).
    /// This is the task-level version; for ISR context use
    /// [`enter_critical_from_isr()`](Self::enter_critical_from_isr).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// System::enter_critical();
    /// // Access shared resource safely
    /// System::exit_critical();
    /// ```
    fn enter_critical();

    /// Exits a critical section at task level.
    ///
    /// Re-enables scheduler and interrupts after [`enter_critical()`](Self::enter_critical).
    /// Must be called from the same task that called `enter_critical()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// System::enter_critical();
    /// // Critical section code
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
    /// Saved interrupt status to be restored on exit
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// // In an interrupt handler
    /// let saved_status = System::enter_critical_from_isr();
    /// // Critical ISR code
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
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// let saved = System::enter_critical_from_isr();
    /// // Protected ISR operations
    /// System::exit_critical_from_isr(saved);
    /// ```
    fn exit_critical_from_isr(saved_interrupt_status: UBaseType);

    /// Gets the amount of free heap memory.
    ///
    /// # Returns
    ///
    /// Number of free bytes in the heap
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let free = System::get_free_heap_size();
    /// println!("Free heap: {} bytes", free);
    /// ```
    fn get_free_heap_size() -> usize;
    
}
