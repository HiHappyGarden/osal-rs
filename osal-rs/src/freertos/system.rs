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

//! System-level functions and utilities for FreeRTOS.
//!
//! This module provides access to system-wide functionality including:
//! - Scheduler control (start, stop, suspend, resume)
//! - System time and delay functions
//! - Thread enumeration and state inspection
//! - Critical sections
//! - Heap memory information

use core::fmt::Debug;
use core::ops::Deref;
use core::time::Duration;

use alloc::vec::Vec;

use super::ffi::{
    BLOCKED, DELETED, READY, RUNNING, SUSPENDED, TaskStatus, eTaskGetState, osal_rs_critical_section_enter, osal_rs_critical_section_exit, osal_rs_port_end_switching_isr, osal_rs_port_yield_from_isr, uxTaskGetNumberOfTasks, uxTaskGetSystemState, vTaskDelay, vTaskEndScheduler, vTaskStartScheduler, vTaskSuspendAll, xPortGetFreeHeapSize, xTaskDelayUntil, xTaskGetCurrentTaskHandle, xTaskGetTickCount, xTaskResumeAll, osal_rs_task_enter_critical, osal_rs_task_enter_critical_from_isr, osal_rs_task_exit_critical, osal_rs_task_exit_critical_from_isr
};
use super::thread::{ThreadState, ThreadMetadata};
use super::types::{BaseType, TickType, UBaseType};
use crate::tick_period_ms;
use crate::traits::{SystemFn, ToTick};
use crate::utils::{CpuRegisterSize::*, register_bit_size, OsalRsBool};

/// Represents a snapshot of the system state including all threads.
///
/// Contains metadata for all threads in the system and total runtime statistics.
/// This is useful for monitoring, debugging, and profiling.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// 
/// let state = System::get_all_thread();
/// 
/// println!("Total threads: {}", state.tasks.len());
/// println!("Total runtime: {}", state.total_run_time);
/// 
/// for thread in &state.tasks {
///     println!("Thread: {} - Priority: {} - State: {:?}",
///         thread.name,
///         thread.priority,
///         thread.state
///     );
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SystemState {
    /// List of all thread metadata in the system
    pub tasks: Vec<ThreadMetadata>,
    /// Total runtime counter across all threads (if enabled)
    pub total_run_time: u32
}

/// Provides access to the task list as a slice.
impl Deref for SystemState {
    type Target = [ThreadMetadata];

    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

/// System-level operations and utilities.
///
/// Provides a collection of static methods for controlling the FreeRTOS scheduler
/// and accessing system-wide information. All methods are static.
///
/// # Examples
///
/// ## Starting the scheduler
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// 
/// // Create threads, queues, etc.
/// // ...
/// 
/// // Start the scheduler (never returns in normal operation)
/// System::start();
/// ```
///
/// ## Delays and timing
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// use core::time::Duration;
/// 
/// // Simple delay
/// System::delay_with_to_tick(Duration::from_millis(500));
/// 
/// // Get current system time
/// let now = System::get_current_time_us();
/// println!("Uptime: {:?}", now);
/// 
/// // Periodic execution using delay_until
/// let mut last_wake = System::get_tick_count();
/// loop {
///     System::delay_until_with_to_tick(&mut last_wake, Duration::from_millis(100));
///     println!("Periodic task");
/// }
/// ```
///
/// ## Critical sections
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// 
/// // Protect shared data
/// System::critical_section_enter();
/// // Access shared data here
/// // ...
/// System::critical_section_exit();
/// ```
///
/// ## Thread enumeration
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// 
/// let count = System::count_threads();
/// println!("Active threads: {}", count);
/// 
/// let state = System::get_all_thread();
/// for thread in &state.tasks {
///     println!("Thread: {} - Stack high water: {}",
///         thread.name,
///         thread.stack_high_water_mark
///     );
/// }
/// ```
///
/// ## Heap monitoring
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// 
/// let free_heap = System::get_free_heap_size();
/// println!("Free heap: {} bytes", free_heap);
/// ```
///
/// ## Scheduler suspend/resume
///
/// ```ignore
/// use osal_rs::os::{System, SystemFn};
/// 
/// // Suspend scheduler for atomic operations
/// System::suspend_all();
/// // Perform atomic operations
/// // ...
/// System::resume_all();
/// ```
/// System-level operations and scheduler control.
///
/// Provides static methods for controlling the RTOS scheduler, timing,
/// and system-wide operations.
pub struct System;

impl System {
    /// Delays execution using a type that implements `ToTick`.
    ///
    /// Convenience method that accepts `Duration` or other tick-convertible types.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// use core::time::Duration;
    /// 
    /// System::delay_with_to_tick(Duration::from_millis(100));
    /// ```
    #[inline]
    pub fn delay_with_to_tick(ticks: impl ToTick){
        Self::delay(ticks.to_ticks());
    }

    /// Delays until an absolute time point with tick conversion.
    ///
    /// Used for precise periodic timing.
    ///
    /// # Parameters
    ///
    /// * `previous_wake_time` - Previous wake time (updated by this function)
    /// * `time_increment` - Time increment for next wake
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// use core::time::Duration;
    /// 
    /// let mut last_wake = System::get_tick_count();
    /// loop {
    ///     // Do work...
    ///     System::delay_until_with_to_tick(&mut last_wake, Duration::from_millis(100));
    /// }
    /// ```
    #[inline]
    pub fn delay_until_with_to_tick(previous_wake_time: &mut TickType, time_increment: impl ToTick) { 
        Self::delay_until(previous_wake_time, time_increment.to_ticks());
    }
}

impl SystemFn for System {
    /// Starts the RTOS scheduler.
    ///
    /// This function never returns if successful. All created threads will
    /// begin execution according to their priorities.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn, Thread};
    /// 
    /// // Create threads...
    /// let thread = Thread::new("worker", 2048, 5, || {
    ///     loop { /* work */ }
    /// }).unwrap();
    /// 
    /// thread.start().unwrap();
    /// 
    /// // Start scheduler (does not return)
    /// System::start();
    /// ```
    fn start() {
        unsafe {
            vTaskStartScheduler();
        }
    }

    /// Gets the state of the currently executing thread.
    ///
    /// # Returns
    ///
    /// Current thread state enum value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn, ThreadState};
    /// 
    /// let state = System::get_state();
    /// match state {
    ///     ThreadState::Running => println!("Currently running"),
    ///     _ => println!("Other state"),
    /// }
    /// ```
    fn get_state() -> ThreadState {
        use super::thread::ThreadState::*;
        let state = unsafe { eTaskGetState(xTaskGetCurrentTaskHandle()) };
        match state {
            RUNNING => Running,
            READY => Ready,
            BLOCKED => Blocked,
            SUSPENDED => Suspended,
            DELETED => Deleted,
            _ => Invalid, // INVALID or unknown state
        }
    }

    /// Suspends all tasks in the scheduler.
    ///
    /// No context switches will occur until `resume_all()` is called.
    /// Use this to create atomic sections spanning multiple operations.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// System::suspend_all();
    /// // Perform critical operations
    /// System::resume_all();
    /// ```
    fn suspend_all() {
        unsafe {
            vTaskSuspendAll();
        }
    }
    
    /// Resumes all suspended tasks.
    ///
    /// # Returns
    ///
    /// Non-zero if a context switch should occur
    ///
    /// # Examples
    ///
    /// ```ignore
    /// System::resume_all();
    /// ```
    fn resume_all() -> BaseType {
        unsafe { xTaskResumeAll() }
    }

    /// Stops the RTOS scheduler.
    ///
    /// All threads will stop executing. Rarely used in embedded systems.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// System::stop();
    /// ```
    fn stop() {
        unsafe {
            vTaskEndScheduler();
        }
    }

    /// Returns the current tick count.
    ///
    /// The tick count increments with each RTOS tick interrupt.
    ///
    /// # Returns
    ///
    /// Current tick count value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// let ticks = System::get_tick_count();
    /// println!("Current ticks: {}", ticks);
    /// ```
    fn get_tick_count() -> TickType {
        unsafe { xTaskGetTickCount() }
    }

    /// Returns the current system time as a `Duration`.
    ///
    /// Converts the current tick count to microseconds and returns it as
    /// a standard `Duration` type.
    ///
    /// # Returns
    ///
    /// Current system uptime as `Duration`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// let uptime = System::get_current_time_us();
    /// println!("System uptime: {:?}", uptime);
    /// ```
    fn get_current_time_us () -> Duration {
        let ticks = Self::get_tick_count();
        Duration::from_millis( 1_000 * ticks as u64 / tick_period_ms!() as u64 )
    }

    /// Converts a `Duration` to microsecond ticks.
    ///
    /// Helper function for converting duration values to system tick counts
    /// in microsecond resolution.
    ///
    /// # Parameters
    ///
    /// * `duration` - Duration to convert
    ///
    /// # Returns
    ///
    /// Equivalent tick count in microseconds
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// use core::time::Duration;
    /// 
    /// let duration = Duration::from_millis(100);
    /// let us_ticks = System::get_us_from_tick(&duration);
    /// ```
    fn get_us_from_tick(duration: &Duration) -> TickType {
        let millis = duration.as_millis() as TickType;
        millis / (1_000 * tick_period_ms!() as TickType) 
    }

    /// Returns the number of threads currently in the system.
    ///
    /// Includes threads in all states (running, ready, blocked, suspended).
    ///
    /// # Returns
    ///
    /// Total number of threads
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// let count = System::count_threads();
    /// println!("Total threads: {}", count);
    /// ```
    fn count_threads() -> usize {
        unsafe { uxTaskGetNumberOfTasks() as usize }
    }

    /// Retrieves a snapshot of all threads in the system.
    ///
    /// Returns detailed metadata for every thread including state, priority,
    /// stack usage, and runtime statistics.
    ///
    /// # Returns
    ///
    /// `SystemState` containing all thread information
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// let state = System::get_all_thread();
    /// 
    /// for thread in &state.tasks {
    ///     println!("Thread: {} - Stack remaining: {}",
    ///         thread.name,
    ///         thread.stack_high_water_mark
    ///     );
    /// }
    /// ```
    fn get_all_thread() -> SystemState {
        let threads_count = Self::count_threads();
        let mut threads: Vec<TaskStatus> = Vec::with_capacity(threads_count);
        let mut total_run_time: u32 = 0;

        unsafe {
            let count = uxTaskGetSystemState(
                threads.as_mut_ptr(),
                threads_count as UBaseType,
                &raw mut total_run_time,
            ) as usize;
            
            // Set the length only after data has been written by FreeRTOS
            threads.set_len(count);
        }

        let tasks = threads.into_iter()
            .map(|task_status| {
                ThreadMetadata::from((
                    task_status.xHandle, 
                    task_status
                ))
            }).collect();

        SystemState {
            tasks,
            total_run_time
        }
    }


    /// Delays the current thread for the specified number of ticks.
    ///
    /// The thread will enter the Blocked state for the delay period.
    ///
    /// # Parameters
    ///
    /// * `ticks` - Number of ticks to delay
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// System::delay(100);  // Delay 100 ticks
    /// ```
    fn delay(ticks: TickType){
        unsafe {
            vTaskDelay(ticks);
        }
    }

    /// Delays until an absolute time point.
    ///
    /// Used for creating precise periodic timing. The `previous_wake_time`
    /// is updated automatically for the next period.
    ///
    /// # Parameters
    ///
    /// * `previous_wake_time` - Pointer to last wake time (will be updated)
    /// * `time_increment` - Period in ticks
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// let mut last_wake = System::get_tick_count();
    /// loop {
    ///     // Periodic task code...
    ///     System::delay_until(&mut last_wake, 100);  // 100 tick period
    /// }
    /// ```
    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType) {
        unsafe {
            xTaskDelayUntil(
                previous_wake_time,
                time_increment,
            );
        }
    }

    /// Enters a critical section.
    ///
    /// Disables interrupts or increments the scheduler lock nesting count.
    /// Must be paired with `critical_section_exit()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// System::critical_section_enter();
    /// // Critical code - no task switches or interrupts
    /// System::critical_section_exit();
    /// ```
    fn critical_section_enter() {
        unsafe {
            osal_rs_critical_section_enter();
        }
    }
    
    /// Exits a critical section.
    ///
    /// Re-enables interrupts or decrements the scheduler lock nesting count.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// System::critical_section_exit();
    /// ```
    fn critical_section_exit() {
        unsafe {
            osal_rs_critical_section_exit();
        }   
    }
    
    /// Checks if a timer has elapsed.
    ///
    /// Compares the elapsed time since a timestamp against a target duration,
    /// handling tick counter overflow correctly for both 32-bit and 64-bit systems.
    ///
    /// # Parameters
    ///
    /// * `timestamp` - Starting time reference
    /// * `time` - Target duration to wait for
    ///
    /// # Returns
    ///
    /// * `OsalRsBool::True` - Timer has elapsed
    /// * `OsalRsBool::False` - Timer has not yet elapsed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// use core::time::Duration;
    /// 
    /// let start = System::get_current_time_us();
    /// let timeout = Duration::from_secs(1);
    /// 
    /// // Later...
    /// if System::check_timer(&start, &timeout).into() {
    ///     println!("Timeout occurred");
    /// }
    /// ```
    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool {
        let temp_tick_time = Self::get_current_time_us();
        
        let time_passing = if temp_tick_time >= *timestamp {
            temp_tick_time - *timestamp
        } else {
            if register_bit_size() == Bit32 {
                // Handle tick count overflow for 32-bit TickType
                let overflow_correction = Duration::from_micros(0xff_ff_ff_ff_u64);
                overflow_correction - *timestamp + temp_tick_time
            } else {
                // Handle tick count overflow for 64-bit TickType
                let overflow_correction = Duration::from_micros(0xff_ff_ff_ff_ff_ff_ff_ff_u64);
                overflow_correction - *timestamp + temp_tick_time
            }
        };

        if time_passing >= *time {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    /// Yields to a higher priority task from ISR context.
    ///
    /// Should be called when an ISR operation wakes a higher priority task.
    ///
    /// # Parameters
    ///
    /// * `higher_priority_task_woken` - pdTRUE if higher priority task was woken
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In ISR:
    /// let mut woken = pdFALSE;
    /// // ... ISR operations that might wake a task ...
    /// System::yield_from_isr(woken);
    /// ```
    fn yield_from_isr(higher_priority_task_woken: BaseType) {
        unsafe {
            osal_rs_port_yield_from_isr(higher_priority_task_woken);
        }
    }

    /// Ends ISR and performs context switch if needed.
    ///
    /// This function should be called at the end of an interrupt service routine
    /// to trigger a context switch if a higher priority task was woken during
    /// the ISR.
    ///
    /// # Parameters
    ///
    /// * `switch_required` - `pdTRUE` if context switch is required, `pdFALSE` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// use osal_rs::os::ffi::pdTRUE;
    /// 
    /// // In ISR:
    /// let mut switch_required = pdFALSE;
    /// // ... ISR operations that might require context switch ...
    /// System::end_switching_isr(switch_required);
    /// ```
    fn end_switching_isr( switch_required: BaseType ) {
        unsafe {
            osal_rs_port_end_switching_isr( switch_required );
        }
    }

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
    fn enter_critical() {
        unsafe {
            osal_rs_task_enter_critical();
        }
    }

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
    fn exit_critical() {
        unsafe {
            osal_rs_task_exit_critical();
        }
    }

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
    fn enter_critical_from_isr() -> UBaseType {
        unsafe {
            osal_rs_task_enter_critical_from_isr()
        }
    }

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
    fn exit_critical_from_isr(saved_interrupt_status: UBaseType) {
        unsafe {
            osal_rs_task_exit_critical_from_isr(saved_interrupt_status);
        }
    }


    /// Returns the amount of free heap space.
    ///
    /// Useful for monitoring memory usage and detecting leaks.
    ///
    /// # Returns
    ///
    /// Number of free heap bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{System, SystemFn};
    /// 
    /// let free = System::get_free_heap_size();
    /// println!("Free heap: {} bytes", free);
    /// ```
    fn get_free_heap_size() -> usize {
        unsafe {
            xPortGetFreeHeapSize()
        }
    }

}

