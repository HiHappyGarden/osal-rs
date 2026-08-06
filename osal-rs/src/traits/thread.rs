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

use crate::os::types::{BaseType, StackType, ThreadHandle, TickType, UBaseType};
use crate::utils::{Bytes, DoublePtr, Result};

/// Maximum length (in bytes) of a thread name, shared by all backends.
pub(crate) const MAX_TASK_NAME_LEN: usize = 16;

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
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
///
/// // Create a parameter
/// let param: ThreadParam = Arc::new(42u32);
///
/// // In the thread callback, downcast to access it
/// assert_eq!(param.downcast_ref::<u32>(), Some(&42));
///
/// // A downcast to the wrong type simply reports `None`.
/// assert!(param.downcast_ref::<i8>().is_none());
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
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
///
/// let callback: Box<ThreadFnPtr> = Box::new(|_thread, param| {
///     if let Some(p) = param {
///         if let Some(count) = p.downcast_ref::<u32>() {
///             return Ok(Arc::new(*count + 1));
///         }
///     }
///     Ok(Arc::new(0u32))
/// });
///
/// // This is how `spawn` invokes it: with a handle to the thread itself and
/// // the parameter it was spawned with.
/// let thread = Thread::new("worker", 1024, 1);
/// let result = callback(Box::new(thread), Some(Arc::new(41u32))).unwrap();
///
/// assert_eq!(result.downcast_ref::<u32>(), Some(&42));
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
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicU32, Ordering};
///
/// static RUNS: AtomicU32 = AtomicU32::new(0);
///
/// let mut thread = Thread::new("simple", 1024, 1);
/// let worker = thread.spawn_simple(|| {
///     for _ in 0..3 {
///         RUNS.fetch_add(1, Ordering::SeqCst);
///         System::delay(5);
///     }
///     Ok(Arc::new(()))
/// }).unwrap();
///
/// worker.delete(); // waits for the thread to finish
/// assert_eq!(RUNS.load(Ordering::SeqCst), 3);
/// ```
pub type ThreadSimpleFnPtr = dyn Fn() -> Result<ThreadParam> + Send + Sync + 'static;

/// Thread notification actions.
///
/// Defines different ways to notify a thread, using a lightweight task-notification
/// mechanism modeled on FreeRTOS's (backends without native support, e.g. POSIX,
/// emulate the same one-slot semantics). Provides a lightweight alternative to
/// semaphores and queues for simple signaling between threads or from ISRs to threads.
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
/// ```
/// use osal_rs::os::*;
///
/// let thread = Thread::get_current();
///
/// // Increment notification counter
/// thread.notify(ThreadNotification::Increment).unwrap();
/// assert_eq!(thread.wait_notification(0, 0xFFFF_FFFF, 10).unwrap(), 1);
///
/// // Set specific bits (can combine multiple events)
/// thread.notify(ThreadNotification::SetBits(0b1010)).unwrap();
/// assert_eq!(thread.wait_notification(0, 0xFFFF_FFFF, 10).unwrap(), 0b1010);
///
/// // Set value, overwriting any existing value
/// thread.notify(ThreadNotification::SetValueWithOverwrite(42)).unwrap();
/// assert_eq!(thread.wait_notification(0, 0, 10).unwrap(), 42);
///
/// // Set value only if no pending notifications - the previous one was
/// // consumed by the wait above, so this one goes through.
/// thread.notify(ThreadNotification::SetValueWithoutOverwrite(100)).unwrap();
/// assert_eq!(thread.wait_notification(0, 0, 10).unwrap(), 100);
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

/// Represents the possible states of an RTOS task/thread.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
///
/// let thread = Thread::get_current();
/// let metadata = thread.get_metadata();
///
/// match metadata.state {
///     ThreadState::Running => (),   // currently executing
///     ThreadState::Ready => (),     // ready to run
///     ThreadState::Blocked => (),   // waiting for an event
///     ThreadState::Suspended => (), // explicitly suspended
///     _ => (),                      // deleted or unknown
/// }
///
/// // The thread asking is, by definition, the one running.
/// assert_eq!(metadata.state, ThreadState::Running);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ThreadState {
    /// Thread is currently executing on a CPU
    Running = 0,
    /// Thread is ready to run but not currently executing
    Ready = 1,
    /// Thread is blocked waiting for an event (e.g., semaphore, queue)
    Blocked = 2,
    /// Thread has been explicitly suspended
    Suspended = 3,
    /// Thread has been deleted
    Deleted = 4,
    /// Invalid or unknown state
    Invalid,
}

/// Metadata and runtime information about a thread.
///
/// Contains detailed information about a thread's state, priorities, stack usage,
/// and runtime statistics.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
///
/// let thread = Thread::get_current();
/// let metadata = thread.get_metadata();
///
/// // Name, priority and stack usage are all part of the snapshot.
/// let _ = (metadata.name.as_str(), metadata.priority, metadata.stack_high_water_mark);
///
/// assert_ne!(metadata.state, ThreadState::Invalid);
/// ```
#[derive(Clone, Debug)]
pub struct ThreadMetadata {
    /// OS-level thread/task handle
    pub thread: ThreadHandle,
    /// Thread name
    pub name: Bytes<MAX_TASK_NAME_LEN>,
    /// Original stack depth allocated for this thread
    pub stack_depth: StackType,
    /// Thread priority
    pub priority: UBaseType,
    /// Unique thread number assigned by OS
    pub thread_number: UBaseType,
    /// Current execution state
    pub state: ThreadState,
    /// Current priority (may differ from base priority due to priority inheritance)
    pub current_priority: UBaseType,
    /// Base priority without inheritance
    pub base_priority: UBaseType,
    /// Total runtime counter (requires configGENERATE_RUN_TIME_STATS)
    pub run_time_counter: UBaseType,
    /// Minimum remaining stack space ever recorded (lower values indicate higher stack usage)
    pub stack_high_water_mark: StackType,
}

unsafe impl Send for ThreadMetadata {}
unsafe impl Sync for ThreadMetadata {}

/// Provides default values for ThreadMetadata.
///
/// Creates a metadata instance with null/zero values, representing an
/// invalid or uninitialized thread.
impl Default for ThreadMetadata {
    fn default() -> Self {
        ThreadMetadata {
            thread: ThreadHandle::default(),
            name: Bytes::new(),
            stack_depth: 0,
            priority: 0,
            thread_number: 0,
            state: ThreadState::Invalid,
            current_priority: 0,
            base_priority: 0,
            run_time_counter: 0,
            stack_high_water_mark: 0,
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
/// ```
/// use osal_rs::os::*;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicU32, Ordering};
///
/// static WORK: AtomicU32 = AtomicU32::new(0);
///
/// // Create and spawn a simple thread
/// let mut thread = Thread::new("worker", 2048, 5);
/// let worker = thread.spawn_simple(|| {
///     for _ in 0..3 {
///         WORK.fetch_add(1, Ordering::SeqCst);
///         System::delay(5);
///     }
///     Ok(Arc::new(()))
/// }).unwrap();
///
/// // Create thread with parameter
/// let mut thread2 = Thread::new("counter", 1024, 5);
/// let counter: ThreadParam = Arc::new(AtomicU32::new(0));
/// let counting = thread2.spawn(Some(counter.clone()), |_thread, param| {
///     let param = param.unwrap();
///     if let Some(count) = param.downcast_ref::<AtomicU32>() {
///         count.fetch_add(1, Ordering::SeqCst);
///     }
///     Ok(param)
/// }).unwrap();
///
/// worker.delete();
/// counting.delete();
///
/// assert_eq!(WORK.load(Ordering::SeqCst), 3);
/// assert_eq!(counter.downcast_ref::<AtomicU32>().unwrap().load(Ordering::SeqCst), 1);
/// ```
pub trait Thread {

    /// Returns `true` if the underlying OS handle is null, i.e. the thread
    /// has not been spawned yet or has already been deleted.
    fn is_null(&self) -> bool;

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
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    ///
    /// static SEEN: AtomicU32 = AtomicU32::new(0);
    ///
    /// let mut thread = Thread::new("worker", 1024, 5);
    /// let counter: ThreadParam = Arc::new(100u32);
    ///
    /// let spawned = thread.spawn(Some(counter.clone()), |_thread, param| {
    ///     if let Some(p) = param {
    ///         if let Some(count) = p.downcast_ref::<u32>() {
    ///             SEEN.store(*count, Ordering::SeqCst);
    ///         }
    ///     }
    ///     Ok(Arc::new(200u32))
    /// }).unwrap();
    ///
    /// spawned.delete(); // waits for the thread to finish
    /// assert_eq!(SEEN.load(Ordering::SeqCst), 100);
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
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static LED: AtomicBool = AtomicBool::new(false);
    ///
    /// let mut thread = Thread::new("blinker", 512, 3);
    /// let blinker = thread.spawn_simple(|| {
    ///     for _ in 0..4 {
    ///         LED.fetch_xor(true, Ordering::SeqCst); // toggle the LED
    ///         System::delay(5);
    ///     }
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// blinker.delete();
    /// assert!(!LED.load(Ordering::SeqCst)); // toggled an even number of times
    /// ```
    fn spawn_simple<F>(&mut self, callback: F) -> Result<Self>
    where
        F: Fn() -> Result<ThreadParam> + Send + Sync + 'static,
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
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut thread = Thread::new("temp", 512, 1);
    /// let spawned = thread.spawn_simple(|| {
    ///     // Do some work
    ///     System::delay(5);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// // Later, from another thread
    /// spawned.delete();
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
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    ///
    /// static COUNTER: AtomicU32 = AtomicU32::new(0);
    ///
    /// let mut thread = Thread::new("counter", 1024, 1);
    /// let worker = thread.spawn_simple(|| {
    ///     loop {
    ///         COUNTER.fetch_add(1, Ordering::SeqCst);
    ///         System::delay(1);
    ///     }
    /// }).unwrap();
    ///
    /// System::delay(30);
    /// worker.suspend();  // Pauses the worker, not the caller
    ///
    /// let paused_at = COUNTER.load(Ordering::SeqCst);
    /// System::delay(50);
    ///
    /// // No progress while suspended.
    /// assert_eq!(COUNTER.load(Ordering::SeqCst), paused_at);
    ///
    /// worker.resume();
    /// ```
    fn suspend(&self);

    /// Resumes a suspended thread.
    ///
    /// Resumes execution of a thread that was previously suspended with `suspend()`.
    /// If the thread was not suspended, this has no effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    ///
    /// static COUNTER: AtomicU32 = AtomicU32::new(0);
    ///
    /// let mut thread = Thread::new("counter", 1024, 1);
    /// let worker_thread = thread.spawn_simple(|| {
    ///     loop {
    ///         COUNTER.fetch_add(1, Ordering::SeqCst);
    ///         System::delay(1);
    ///     }
    /// }).unwrap();
    ///
    /// System::delay(30);
    /// worker_thread.suspend();
    ///
    /// let paused_at = COUNTER.load(Ordering::SeqCst);
    /// System::delay(50);
    ///
    /// worker_thread.resume();  // Resume the worker
    /// System::delay(30);
    ///
    /// // Progress resumes.
    /// assert!(COUNTER.load(Ordering::SeqCst) > paused_at);
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
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut thread = Thread::new("worker", 1024, 1);
    /// let spawned = thread.spawn_simple(|| {
    ///     // Do work
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// // Pass a null pointer when the exit value is not needed.
    /// assert!(spawned.join(core::ptr::null_mut()).is_ok());
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
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let thread = Thread::new("worker", 1024, 3);
    /// let meta = thread.get_metadata();
    ///
    /// assert_eq!(meta.name.as_str(), "worker");
    /// assert_eq!(meta.priority, 3);
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
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    /// assert!(!current.is_null());
    ///
    /// let meta = current.get_metadata();
    /// assert_eq!(meta.state, ThreadState::Running);
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
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    ///
    /// static RECEIVED: AtomicU32 = AtomicU32::new(0);
    ///
    /// let mut thread = Thread::new("worker", 1024, 1);
    /// let worker = thread.spawn_simple(|| {
    ///     let current = Thread::get_current();
    ///     // Blocks until the notification below arrives.
    ///     let value = current.wait_notification(0, 0, 1000).unwrap();
    ///     RECEIVED.store(value, Ordering::SeqCst);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// // Send a value. `ThreadNotification::SetBits` would signal an event
    /// // instead, without disturbing the bits another event already set.
    /// worker.notify(ThreadNotification::SetValueWithOverwrite(42)).unwrap();
    ///
    /// worker.delete();
    /// assert_eq!(RECEIVED.load(Ordering::SeqCst), 42);
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
    /// ```
    /// use osal_rs::os::*;
    ///
    /// // In interrupt handler
    /// fn isr_handler(worker: &Thread) {
    ///     let mut task_woken = 0;
    ///
    ///     worker.notify_from_isr(
    ///         ThreadNotification::Increment,
    ///         &mut task_woken
    ///     ).ok();
    ///
    ///     System::yield_from_isr(task_woken);
    /// }
    ///
    /// let current = Thread::get_current();
    /// isr_handler(&current);
    ///
    /// // The notification is now pending on the notified thread.
    /// assert_eq!(current.wait_notification(0, 0xFFFF_FFFF, 10).unwrap(), 1);
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
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    ///
    /// // Nothing pending yet: this gives up once the timeout expires rather
    /// // than blocking forever.
    /// assert!(current.wait_notification(0, 0, 10).is_err());
    ///
    /// // Wait for notification, clear all bits on exit
    /// current.notify(ThreadNotification::SetValueWithOverwrite(7)).unwrap();
    /// match current.wait_notification(0, 0xFFFFFFFF, 1000) {
    ///     Ok(value) => assert_eq!(value, 7),
    ///     Err(_) => panic!("timeout waiting for notification"),
    /// }
    ///
    /// // Wait for specific bits
    /// let bits_of_interest = 0b0011;
    /// current.notify(ThreadNotification::SetBits(bits_of_interest)).unwrap();
    /// match current.wait_notification(0, bits_of_interest, 5000) {
    ///     Ok(value) => assert_ne!(value & bits_of_interest, 0),
    ///     Err(_) => panic!("timeout"),
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
/// ```
/// use osal_rs::os::*;
/// use osal_rs::os::types::UBaseType;
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
/// let thread = Thread::new_with_to_priority("worker", 1024, TaskPriority::High);
/// assert_eq!(thread.get_metadata().priority, 10);
/// ```
pub trait ToPriority {
    /// Converts this value to a priority.
    ///
    /// # Returns
    ///
    /// The priority value as `UBaseType`
    fn to_priority(&self) -> UBaseType;
}