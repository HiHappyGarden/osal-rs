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

//! Thread-related traits and type definitions.
//!
//! This module provides the core abstractions for creating and managing RTOS tasks/threads,
//! including thread lifecycle, notifications, and priority management.
//!
//! # Overview
//!
//! In RTOS terminology, tasks and threads are often used interchangeably. This module
//! uses "Thread" for consistency with Rust conventions, but these map directly to
//! RTOS tasks.
//!
//! # Thread Lifecycle
//!
//! 1. **Creation**: Use `Thread::new()` with name, stack size, and priority
//! 2. **Spawning**: Call `spawn()` or `spawn_simple()` with the thread function
//! 3. **Execution**: Thread runs until function returns or `delete()` is called
//! 4. **Cleanup**: Call `delete()` to free resources
//!
//! # Thread Notifications
//!
//! Threads support lightweight task notifications as an alternative to semaphores
//! and queues for simple signaling. See `ThreadNotification` for available actions.
//!
//! # Priority Management
//!
//! Higher priority threads preempt lower priority ones. Priority 0 is typically
//! reserved for the idle task. Use `ToPriority` trait for flexible priority specification.

use core::any::Any;
use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::os::{ThreadMetadata};
use crate::os::types::{BaseType, TickType, UBaseType};
use crate::utils::{Result, DoublePtr};

/// Type-erased parameter that can be passed to thread callbacks.
///
/// Allows passing arbitrary data to thread functions in a thread-safe manner.
/// The parameter is wrapped in an `Arc` for safe sharing across thread boundaries
/// and can be downcast to its original type using `downcast_ref()`.
///
/// # Thread Safety
///
/// The inner type must implement `Any + Send + Sync` to ensure it can be
/// safely shared between threads.
///
/// # Examples
///
/// ```ignore
/// use std::sync::Arc;
/// use osal_rs::traits::ThreadParam;
///
/// // Create a parameter
/// let param: ThreadParam = Arc::new(42u32);
///
/// // In the thread callback, downcast to access
/// if let Some(value) = param.downcast_ref::<u32>() {
///     println!("Received: {}", value);
/// }
/// ```
pub type ThreadParam = Arc<dyn Any + Send + Sync>;

/// Thread callback function pointer type.
///
/// Thread callbacks receive a boxed thread handle and optional parameter,
/// and can return an updated parameter value.
///
/// # Parameters
///
/// - `Box<dyn Thread>` - Handle to the thread itself (for self-reference)
/// - `Option<ThreadParam>` - Optional type-erased parameter passed at spawn time
///
/// # Returns
///
/// `Result<ThreadParam>` - Updated parameter or error
///
/// # Trait Bounds
///
/// The function must be `Send + Sync + 'static` to be safely used across
/// thread boundaries and to live for the duration of the thread.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Thread, ThreadParam};
/// use std::sync::Arc;
///
/// let callback: Box<ThreadFnPtr> = Box::new(|thread, param| {
///     if let Some(p) = param {
///         if let Some(count) = p.downcast_ref::<u32>() {
///             println!("Count: {}", count);
///         }
///     }
///     Ok(Arc::new(0u32))
/// });
/// ```
pub type ThreadFnPtr = dyn Fn(Box<dyn Thread>, Option<ThreadParam>) -> Result<ThreadParam> + Send + Sync + 'static;

/// Simple thread function pointer type without parameters.
///
/// Used for basic thread functions that don't need access to the thread handle
/// or parameters. This is the simplest form of thread callback.
///
/// # Trait Bounds
///
/// The function must be `Send + Sync + 'static` to be safely used in a
/// multi-threaded environment.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::Thread;
///
/// let mut thread = Thread::new("simple", 1024, 1);
/// thread.spawn_simple(|| {
///     loop {
///         println!("Hello from thread!");
///         System::delay(1000);
///     }
/// }).unwrap();
/// ```
pub type ThreadSimpleFnPtr = dyn Fn() + Send + Sync + 'static;

/// Thread notification actions.
///
/// Defines different ways to notify a thread using the FreeRTOS task notification mechanism.
/// Task notifications provide a lightweight alternative to semaphores and queues for
/// simple signaling between threads or from ISRs to threads.
///
/// # Performance
///
/// Task notifications are faster and use less memory than semaphores or queues,
/// but each thread has only one notification value (32 bits).
///
/// # Common Patterns
///
/// - **Event Signaling**: Use `Increment` or `SetBits` to signal events
/// - **Value Passing**: Use `SetValueWithOverwrite` to pass a value
/// - **Non-Blocking Updates**: Use `SetValueWithoutOverwrite` to avoid data races
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Thread, ThreadNotification};
/// 
/// let thread = Thread::current();
/// 
/// // Increment notification counter
/// thread.notify(ThreadNotification::Increment);
/// 
/// // Set specific bits (can combine multiple events)
/// thread.notify(ThreadNotification::SetBits(0b1010));
/// 
/// // Set value, overwriting any existing value
/// thread.notify(ThreadNotification::SetValueWithOverwrite(42));
/// 
/// // Set value only if no pending notifications
/// thread.notify(ThreadNotification::SetValueWithoutOverwrite(100));
/// ```
#[derive(Debug, Copy, Clone)]
pub enum ThreadNotification {
    /// Don't update the notification value.
    ///
    /// Can be used to just query whether a task has been notified.
    NoAction,
    /// Bitwise OR the notification value with the specified bits.
    ///
    /// Useful for setting multiple event flags that accumulate.
    SetBits(u32),
    /// Increment the notification value by one.
    ///
    /// Useful for counting events or implementing a lightweight counting semaphore.
    Increment,
    /// Set the notification value, overwriting any existing value.
    ///
    /// Use when you want to send a value and don't care if it overwrites
    /// a previous unread value.
    SetValueWithOverwrite(u32),
    /// Set the notification value only if the receiving thread has no pending notifications.
    ///
    /// Use when you want to avoid overwriting an unread value. Returns an error
    /// if a notification is already pending.
    SetValueWithoutOverwrite(u32),
}

impl Into<(u32, u32)> for ThreadNotification {
    fn into(self) -> (u32, u32) {
        use ThreadNotification::*;
        match self {
            NoAction => (0, 0),
            SetBits(bits) => (1, bits),
            Increment => (2, 0),
            SetValueWithOverwrite(value) => (3, value),
            SetValueWithoutOverwrite(value) => (4, value),
        }
    }
}

/// Core thread/task trait.
///
/// Provides methods for thread lifecycle management, synchronization,
/// and communication through task notifications.
///
/// # Thread Creation
///
/// Threads are typically created with `Thread::new()` specifying name,
/// stack size, and priority, then started with `spawn()` or `spawn_simple()`.
///
/// # Thread Safety
///
/// All methods are thread-safe. ISR-specific methods (suffixed with `_from_isr`)
/// should only be called from interrupt context.
///
/// # Resource Management
///
/// Threads should be properly deleted with `delete()` when no longer needed
/// to free stack memory and control structures.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Thread, System};
/// use std::sync::Arc;
///
/// // Create and spawn a simple thread
/// let mut thread = Thread::new("worker", 2048, 5);
/// thread.spawn_simple(|| {
///     loop {
///         println!("Working...");
///         System::delay(1000);
///     }
/// }).unwrap();
///
/// // Create thread with parameter
/// let mut thread2 = Thread::new("counter", 1024, 5);
/// let counter = Arc::new(0u32);
/// thread2.spawn(Some(counter.clone()), |_thread, param| {
///     // Use param here
///     Ok(param.unwrap())
/// }).unwrap();
/// ```
pub trait Thread {
    /// Spawns a thread with a callback function and optional parameter.
    ///
    /// Creates and starts a new thread that executes the provided callback function.
    /// The callback receives a handle to itself and an optional parameter.
    ///
    /// # Parameters
    ///
    /// * `param` - Optional type-erased parameter passed to the callback
    /// * `callback` - Function to execute in the thread context
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Thread spawned successfully
    /// * `Err(Error)` - Failed to create or start thread
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    /// use std::sync::Arc;
    ///
    /// let mut thread = Thread::new("worker", 1024, 5);
    /// let counter = Arc::new(100u32);
    ///
    /// thread.spawn(Some(counter.clone()), |thread, param| {
    ///     if let Some(p) = param {
    ///         if let Some(count) = p.downcast_ref::<u32>() {
    ///             println!("Starting with count: {}", count);
    ///         }
    ///     }
    ///     Ok(Arc::new(200u32))
    /// }).unwrap();
    /// ```
    fn spawn<F>(&mut self, param: Option<ThreadParam>, callback: F) -> Result<Self>
    where 
        F: Fn(Box<dyn Thread>, Option<ThreadParam>) -> Result<ThreadParam>,
        F: Send + Sync + 'static,
        Self: Sized;

    /// Spawns a simple thread with a callback function (no parameters).
    ///
    /// Creates and starts a new thread that executes the provided callback.
    /// This is a simpler version of `spawn()` for threads that don't need
    /// parameters or self-reference.
    ///
    /// # Parameters
    ///
    /// * `callback` - Function to execute in the thread context
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Thread spawned successfully
    /// * `Err(Error)` - Failed to create or start thread
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, System};
    ///
    /// let mut thread = Thread::new("blinker", 512, 3);
    /// thread.spawn_simple(|| {
    ///     loop {
    ///         toggle_led();
    ///         System::delay(500);
    ///     }
    /// }).unwrap();
    /// ```
    fn spawn_simple<F>(&mut self, callback: F) -> Result<Self>
    where
        F: Fn() + Send + Sync + 'static,
        Self: Sized;

    /// Deletes the thread and frees its resources.
    ///
    /// Terminates the thread and releases its stack and control structures.
    /// After calling this, the thread handle becomes invalid.
    ///
    /// # Safety
    ///
    /// - The thread should not be holding any resources (mutexes, etc.)
    /// - Other threads should not be waiting on this thread
    /// - Cannot delete the currently running thread (use from another thread)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    ///
    /// let mut thread = Thread::new("temp", 512, 1);
    /// thread.spawn_simple(|| {
    ///     // Do some work
    /// }).unwrap();
    ///
    /// // Later, from another thread
    /// thread.delete();
    /// ```
    fn delete(&self);

    /// Suspends the thread.
    ///
    /// Prevents the thread from executing until `resume()` is called.
    /// The thread state is preserved and can be resumed later.
    ///
    /// # Use Cases
    ///
    /// - Temporarily pause a thread
    /// - Debugging and development
    /// - Dynamic task management
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    ///
    /// let thread = Thread::current();
    /// thread.suspend();  // Pauses this thread
    /// ```
    fn suspend(&self);

    /// Resumes a suspended thread.
    ///
    /// Resumes execution of a thread that was previously suspended with `suspend()`.
    /// If the thread was not suspended, this has no effect.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Thread 1
    /// worker_thread.suspend();
    ///
    /// // Thread 2
    /// worker_thread.resume();  // Resume Thread 1
    /// ```
    fn resume(&self);

    /// Waits for the thread to complete and retrieves its return value.
    ///
    /// Blocks the calling thread until this thread terminates. The thread's
    /// return value is stored in the provided pointer.
    ///
    /// # Parameters
    ///
    /// * `retval` - Pointer to store the thread's return value
    ///
    /// # Returns
    ///
    /// * `Ok(exit_code)` - Thread completed successfully
    /// * `Err(Error)` - Join operation failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    ///
    /// let mut thread = Thread::new("worker", 1024, 1);
    /// thread.spawn_simple(|| {
    ///     // Do work
    /// }).unwrap();
    ///
    /// let mut retval = core::ptr::null_mut();
    /// thread.join(&mut retval).unwrap();
    /// ```
    fn join(&self, retval: DoublePtr) -> Result<i32>;

    /// Gets metadata about the thread.
    ///
    /// Returns information such as thread name, priority, stack usage,
    /// and current state.
    ///
    /// # Returns
    ///
    /// `ThreadMetadata` structure containing thread information
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    ///
    /// let thread = Thread::current();
    /// let meta = thread.get_metadata();
    /// println!("Thread: {} Priority: {}", meta.name, meta.priority);
    /// ```
    fn get_metadata(&self) -> ThreadMetadata;

    /// Gets a handle to the currently executing thread.
    ///
    /// Returns a handle to the thread that is currently running.
    /// Useful for self-referential operations.
    ///
    /// # Returns
    ///
    /// Handle to the current thread
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    ///
    /// let current = Thread::get_current();
    /// let meta = current.get_metadata();
    /// println!("Running in thread: {}", meta.name);
    /// ```
    fn get_current() -> Self
    where 
        Self: Sized;

    /// Sends a notification to the thread.
    ///
    /// Notifies the thread using the specified notification action.
    /// Task notifications are a lightweight signaling mechanism.
    ///
    /// # Parameters
    ///
    /// * `notification` - The notification action to perform
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Notification sent successfully
    /// * `Err(Error)` - Failed to send notification
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadNotification};
    ///
    /// let worker = get_worker_thread();
    /// 
    /// // Signal an event
    /// worker.notify(ThreadNotification::SetBits(0b0001)).unwrap();
    /// 
    /// // Send a value
    /// worker.notify(ThreadNotification::SetValueWithOverwrite(42)).unwrap();
    /// ```
    fn notify(&self, notification: ThreadNotification) -> Result<()>;

    /// Sends a notification to the thread from ISR context.
    ///
    /// ISR-safe version of `notify()`. Must only be called from interrupt context.
    ///
    /// # Parameters
    ///
    /// * `notification` - The notification action to perform
    /// * `higher_priority_task_woken` - Set to non-zero if a context switch should occur
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Notification sent successfully
    /// * `Err(Error)` - Failed to send notification
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadNotification, System};
    ///
    /// // In interrupt handler
    /// fn isr_handler() {
    ///     let worker = get_worker_thread();
    ///     let mut task_woken = 0;
    ///     
    ///     worker.notify_from_isr(
    ///         ThreadNotification::Increment,
    ///         &mut task_woken
    ///     ).ok();
    ///     
    ///     System::yield_from_isr(task_woken);
    /// }
    /// ```
    fn notify_from_isr(&self, notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()>;

    /// Waits for a notification.
    ///
    /// Blocks the calling thread until a notification is received or timeout occurs.
    /// Allows clearing specific bits on entry and/or exit.
    ///
    /// # Parameters
    ///
    /// * `bits_to_clear_on_entry` - Bits to clear before waiting
    /// * `bits_to_clear_on_exit` - Bits to clear after receiving notification
    /// * `timeout_ticks` - Maximum ticks to wait (0 = no wait, MAX = wait forever)
    ///
    /// # Returns
    ///
    /// * `Ok(notification_value)` - Notification received, returns the notification value
    /// * `Err(Error::Timeout)` - No notification received within timeout
    /// * `Err(Error)` - Other error occurred
    ///
    /// # Note
    ///
    /// This method does not use `ToTick` trait to maintain dynamic dispatch compatibility.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    ///
    /// let current = Thread::get_current();
    ///
    /// // Wait for notification, clear all bits on exit
    /// match current.wait_notification(0, 0xFFFFFFFF, 1000) {
    ///     Ok(value) => println!("Notified with value: {}", value),
    ///     Err(_) => println!("Timeout waiting for notification"),
    /// }
    ///
    /// // Wait for specific bits
    /// let bits_of_interest = 0b0011;
    /// match current.wait_notification(0, bits_of_interest, 5000) {
    ///     Ok(value) => {
    ///         if value & bits_of_interest != 0 {
    ///             println!("Received expected bits");
    ///         }
    ///     },
    ///     Err(_) => println!("Timeout"),
    /// }
    /// ```
    fn wait_notification(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32 , timeout_ticks: TickType) -> Result<u32>;


}

/// Trait for converting types to thread priority values.
///
/// Allows flexible specification of thread priorities using different types
/// (e.g., integers, enums) that can be converted to the underlying RTOS
/// priority representation.
///
/// # Priority Ranges
///
/// Priority 0 is typically reserved for the idle task. Higher numbers
/// indicate higher priority (preemptive scheduling).
///
/// # Examples
///
/// ```ignore
/// use osal_rs::traits::ToPriority;
///
/// // Implement for a custom priority enum
/// enum TaskPriority {
///     Low,
///     Medium,
///     High,
/// }
///
/// impl ToPriority for TaskPriority {
///     fn to_priority(&self) -> UBaseType {
///         match self {
///             TaskPriority::Low => 1,
///             TaskPriority::Medium => 5,
///             TaskPriority::High => 10,
///         }
///     }
/// }
///
/// let thread = Thread::new("worker", 1024, TaskPriority::High);
/// ```
pub trait ToPriority {
    /// Converts this value to a priority.
    ///
    /// # Returns
    ///
    /// The priority value as `UBaseType`
    fn to_priority(&self) -> UBaseType;
}