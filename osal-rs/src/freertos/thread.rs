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

//! Thread management and synchronization for FreeRTOS.
//!
//! This module provides a safe Rust interface for creating and managing FreeRTOS tasks.
//! It supports thread creation with callbacks, priority management, and thread notifications.

use core::ffi::c_void;
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use alloc::boxed::Box;
use alloc::sync::Arc;

use super::ffi::task::INVALID;
use super::ffi::{ TaskStatus, ThreadHandle, pdPASS, pdTRUE, vTaskDelete, vTaskGetInfo, vTaskResume, vTaskSuspend, xTaskCreate, xTaskGetCurrentTaskHandle};
use super::types::{StackType, UBaseType, BaseType, TickType};
use crate::traits::ThreadState::*;
use crate::os::ThreadSimpleFnPtr;
use crate::traits::{ThreadFn, ThreadParam, ThreadFnPtr, ThreadNotification, ThreadMetadata, ToTick, ToPriority};
use crate::traits::{MAX_TASK_NAME_LEN, SemaphoreFn};
use crate::freertos::semaphore::Semaphore;
use crate::utils::{Bytes, DoublePtr, Error, MAX_DELAY, Result};

/// Exit latch shared between a task spawned through this crate and whoever
/// joins it.
///
/// FreeRTOS has no `pthread_join`: a task function simply returns and deletes
/// itself, with no way for another task to wait on that or to collect a
/// result. This supplies both, so [`ThreadFn::join`] means the same thing on
/// either backend - block until the callback has returned, then hand back its
/// value.
///
/// One binary semaphore per spawned thread is the cost. It is allocated in
/// `spawn`/`spawn_simple` and released once the last `Arc` to it drops, i.e.
/// when both the task wrapper and every `Thread` handle are gone.
struct JoinState {
    /// Signalled exactly once by the task wrapper, after `retval` and
    /// `finished` have been published.
    done: Semaphore,
    /// Set just before `done` is signalled. Read by `delete()` to tell a task
    /// that is still running from one that has already deleted itself, and by
    /// `join()` to skip the wait when the task finished first.
    finished: AtomicBool,
    /// Set by the first `join()`, so a second one reports failure instead of
    /// blocking forever on an already-consumed `done`.
    joined: AtomicBool,
    /// The callback's `Result<ThreadParam>`, boxed and leaked exactly the way
    /// `posix`'s task wrapper leaks it, so `join`'s `DoublePtr` carries the
    /// same thing on both backends.
    retval: AtomicPtr<c_void>,
}

impl JoinState {
    fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            done: Semaphore::new(1, 0)?,
            finished: AtomicBool::new(false),
            joined: AtomicBool::new(false),
            retval: AtomicPtr::new(null_mut()),
        }))
    }

    /// Publishes the callback's result and releases anyone blocked in
    /// `join()`. Called from the task wrapper just before it self-deletes.
    fn publish(&self, ret: Result<ThreadParam>) {
        self.retval
            .store(Box::into_raw(Box::new(ret)) as *mut c_void, Ordering::Release);
        self.finished.store(true, Ordering::Release);
        self.done.signal();
    }

    /// Drops a still-unclaimed exit value, so a thread nobody joins does not
    /// leak its boxed result.
    fn discard_retval(&self) {
        let raw = self.retval.swap(null_mut(), Ordering::Acquire);
        if !raw.is_null() {
            // SAFETY: `raw` came from `Box::into_raw` in `publish`, and the
            // swap guarantees only one caller observes it non-null.
            drop(unsafe { Box::from_raw(raw as *mut Result<ThreadParam>) });
        }
    }
}

impl Drop for JoinState {
    fn drop(&mut self) {
        self.discard_retval();
    }
}

/// Converts a FreeRTOS TaskStatus into ThreadMetadata.
///
/// This conversion extracts all relevant task information from the FreeRTOS
/// TaskStatus structure and creates a safe Rust representation.
impl From<(ThreadHandle,TaskStatus)> for ThreadMetadata {
    fn from(status: (ThreadHandle, TaskStatus)) -> Self {
        let state = match status.1.eCurrentState {
            0 => Running,
            1 => Ready,
            2 => Blocked,
            3 => Suspended,
            4 => Deleted,
            _ => Invalid,
        };

        ThreadMetadata {
            thread: status.0,
            name: Bytes::from_char_ptr(status.1.pcTaskName),
            // Avoid dereferencing pxStackBase, which may be null or otherwise invalid.
            // Use 0 as a safe default for unknown stack depth.
            stack_depth: 0,
            priority: status.1.uxBasePriority,
            thread_number: status.1.xTaskNumber,
            state,
            current_priority: status.1.uxCurrentPriority,
            base_priority: status.1.uxBasePriority,
            run_time_counter: status.1.ulRunTimeCounter,
            stack_high_water_mark: status.1.usStackHighWaterMark,
        }
    }
}

/// A FreeRTOS task/thread wrapper.
///
/// Provides a safe Rust interface for creating and managing FreeRTOS tasks.
/// Threads can be created with closures or function pointers and support
/// various synchronization primitives.
///
/// # Examples
///
/// ## Creating a simple thread
///
/// ```ignore
/// use osal_rs::os::{Thread, ThreadPriority};
/// use core::time::Duration;
/// 
/// let thread = Thread::new(
///     "worker",
///     2048,  // stack size in words
///     ThreadPriority::Normal,
///     || {
///         loop {
///             println!("Working...");
///             Duration::from_secs(1).sleep();
///         }
///     }
/// ).unwrap();
/// 
/// thread.start().unwrap();
/// ```
///
/// ## Using thread notifications
///
/// ```ignore
/// use osal_rs::os::{Thread, ThreadNotification};
/// use core::time::Duration;
/// 
/// let thread = Thread::new("notified", 2048, 5, || {
///     loop {
///         if let Some(value) = Thread::current().wait_notification(Duration::from_secs(1)) {
///             println!("Received notification: {}", value);
///         }
///     }
/// }).unwrap();
/// 
/// thread.start().unwrap();
/// thread.notify(42).unwrap();  // Send notification
/// ```
#[derive(Clone)]
pub struct Thread {
    handle: ThreadHandle,
    name: Bytes<MAX_TASK_NAME_LEN>,
    stack_depth: StackType,
    priority: UBaseType,
    callback: Option<Arc<ThreadFnPtr>>,
    param: Option<ThreadParam>,
    /// `Some` only for threads this crate spawned; `None` for a handle-less
    /// `new()` and for foreign tasks wrapped with `new_with_handle`, neither
    /// of which has anything to wait on.
    join_state: Option<Arc<JoinState>>,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {

    /// Creates a new uninitialized thread.
    ///
    /// The thread must be started with `spawn()` or `spawn_simple()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn};
    /// 
    /// let thread = Thread::new("worker", 4096, 5);
    /// ```
    pub fn new(name: &str, stack_depth: StackType, priority: UBaseType) -> Self 
    {
        Self { 
            handle: null_mut(), 
            name: Bytes::from_str(name),
            stack_depth, 
            priority, 
            callback: None,
            param: None,
            join_state: None,
        }
    }

    /// Creates a thread from an existing task handle.
    ///
    /// # Returns
    ///
    /// * `Err(Error::NullPtr)` if handle is null
    pub fn new_with_handle(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: UBaseType) -> Result<Self> {
        if handle.is_null() {
            return Err(Error::NullPtr);
        }
        Ok(Self { 
            handle, 
            name: Bytes::from_str(name), 
            stack_depth, 
            priority, 
            callback: None,
            param: None,
            join_state: None,
        })
    }

    /// Creates a new thread with a priority that implements `ToPriority`.
    ///
    /// This is a convenience constructor that allows using various priority types.
    ///
    /// # Parameters
    ///
    /// * `name` - Thread name for debugging
    /// * `stack_depth` - Stack size in words (not bytes)
    /// * `priority` - Thread priority (any type implementing `ToPriority`)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    /// 
    /// let thread = Thread::new_with_to_priority("worker", 2048, 5);
    /// ```
    pub fn new_with_to_priority(name: &str, stack_depth: StackType, priority: impl ToPriority) -> Self 
    {
        Self { 
            handle: null_mut(), 
            name: Bytes::from_str(name), 
            stack_depth, 
            priority: priority.to_priority(), 
            callback: None,
            param: None,
            join_state: None,
        }
    }

    /// Creates a thread from an existing FreeRTOS task handle.
    ///
    /// # Parameters
    ///
    /// * `handle` - Valid FreeRTOS task handle
    /// * `name` - Thread name
    /// * `stack_depth` - Stack size
    /// * `priority` - Thread priority
    ///
    /// # Returns
    ///
    /// * `Err(Error::NullPtr)` if handle is null
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    /// 
    /// // Get current task handle from FreeRTOS
    /// let handle = get_task_handle();
    /// let thread = Thread::new_with_handle_and_to_priority(handle, "existing", 2048, 5).unwrap();
    /// ```
    pub fn new_with_handle_and_to_priority(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: impl ToPriority) -> Result<Self> {
        if handle.is_null() {
            return Err(Error::NullPtr);
        }
        Ok(Self { 
            handle, 
            name: Bytes::from_str(name),
            stack_depth, 
            priority: priority.to_priority(), 
            callback: None,
            param: None,
            join_state: None,
        })
    }

    /// Retrieves metadata for a thread from its handle.
    ///
    /// # Parameters
    ///
    /// * `handle` - FreeRTOS task handle
    ///
    /// # Returns
    ///
    /// Thread metadata including state, priority, stack usage, etc.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    /// 
    /// let handle = get_some_task_handle();
    /// let metadata = Thread::get_metadata_from_handle(handle);
    /// println!("Thread '{}' state: {:?}", metadata.name, metadata.state);
    /// ```
    pub fn get_metadata_from_handle(handle: ThreadHandle) -> ThreadMetadata {
        let mut status = TaskStatus::default();
        unsafe {
            vTaskGetInfo(handle, &mut status, pdTRUE, INVALID);
        }
        ThreadMetadata::from((handle, status))
    }

    /// Retrieves metadata for a thread object.
    ///
    /// # Parameters
    ///
    /// * `thread` - Thread reference
    ///
    /// # Returns
    ///
    /// Thread metadata or default if handle is null
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::Thread;
    /// 
    /// let thread = Thread::new("worker", 2048, 5);
    /// let metadata = Thread::get_metadata(&thread);
    /// println!("Stack high water mark: {}", metadata.stack_high_water_mark);
    /// ```
    pub fn get_metadata(thread: &Thread) -> ThreadMetadata {
        // Name/stack/priority reflect what was passed to `new()` regardless of
        // whether the thread has been spawned yet; only `state`/`thread`
        // depend on there being a live task behind it. Mirrors
        // `posix::Thread::get_metadata`.
        if thread.is_null() {
            return ThreadMetadata {
                name: thread.name,
                stack_depth: thread.stack_depth,
                priority: thread.priority,
                current_priority: thread.priority,
                base_priority: thread.priority,
                ..ThreadMetadata::default()
            };
        }
        Self::get_metadata_from_handle(thread.handle)
    }

    /// Waits for a thread notification with a timeout that implements `ToTick`.
    ///
    /// Convenience method that accepts `Duration` or other tick-convertible types.
    ///
    /// # Parameters
    ///
    /// * `bits_to_clear_on_entry` - Bits to clear before waiting
    /// * `bits_to_clear_on_exit` - Bits to clear after receiving notification
    /// * `timeout_ticks` - Maximum time to wait (convertible to ticks)
    ///
    /// # Returns
    ///
    /// * `Ok(u32)` - Notification value received
    /// * `Err(Error::NullPtr)` - Thread handle is null
    /// * `Err(Error::Timeout)` - No notification received within timeout
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn};
    /// use core::time::Duration;
    /// 
    /// let thread = Thread::current();
    /// match thread.wait_notification_with_to_tick(0, 0xFF, Duration::from_secs(1)) {
    ///     Ok(value) => println!("Received: {}", value),
    ///     Err(_) => println!("Timeout"),
    /// }
    /// ```
    #[inline]
    pub fn wait_notification_with_to_tick(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32 , timeout_ticks: impl ToTick) -> Result<u32> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }
        self.wait_notification(bits_to_clear_on_entry, bits_to_clear_on_exit, timeout_ticks.to_ticks())
    }

}

/// Internal C-compatible wrapper for thread callbacks.
///
/// This function bridges between FreeRTOS C API and Rust closures.
/// It unpacks the boxed thread instance, initializes the handle,
/// and calls the user-provided callback.
///
/// # Safety
///
/// This function is marked unsafe because it:
/// - Expects a valid pointer to a boxed Thread instance
/// - Performs raw pointer conversions
/// - Is called from C code (FreeRTOS)
unsafe extern "C" fn callback_c_wrapper(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    let mut thread_instance: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut _) };

    thread_instance.as_mut().handle = unsafe { xTaskGetCurrentTaskHandle() };

    let join_state = thread_instance.join_state.clone();

    let param_arc: Option<ThreadParam> = thread_instance
        .param
        .clone();

    let ret = if let Some(callback) = &thread_instance.callback.clone() {
        callback(thread_instance, param_arc)
    } else {
        Err(Error::NullPtr)
    };

    // Release anyone blocked in `join()` *before* self-deleting: after
    // `vTaskDelete` this task never runs again.
    if let Some(join_state) = join_state {
        join_state.publish(ret);
    }

    // Self-delete rather than `Thread::delete()`: that call now skips
    // already-finished tasks, and the task function must not return.
    unsafe { vTaskDelete( xTaskGetCurrentTaskHandle() ); }
}

/// Internal C-compatible wrapper for simple thread callbacks.
///
/// This function bridges between FreeRTOS C API and simple Rust closures
/// (without parameters). It unpacks the boxed function pointer and executes it.
///
/// # Safety
///
/// This function is marked unsafe because it:
/// - Expects a valid pointer to a boxed `(Arc<ThreadSimpleFnPtr>, Arc<JoinState>)`
/// - Performs raw pointer conversions
/// - Is called from C code (FreeRTOS)
/// - Directly calls vTaskDelete after execution
///
/// The callback's `Result<ThreadParam>` is published through the shared
/// [`JoinState`], the same way `callback_c_wrapper` does, so `Thread::join()`
/// works identically for threads spawned with `spawn_simple()`.
unsafe extern "C" fn simple_callback_wrapper(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    let payload: Box<(Arc<ThreadSimpleFnPtr>, Arc<JoinState>)> =
        unsafe { Box::from_raw(param_ptr as *mut _) };
    let (func, join_state) = *payload;

    let ret = func();

    // Release anyone blocked in `join()` *before* self-deleting: after
    // `vTaskDelete` this task never runs again.
    join_state.publish(ret);

    unsafe { vTaskDelete( xTaskGetCurrentTaskHandle()); }
}



impl Thread {
    /// `true` once a thread spawned through this crate has run its callback to
    /// completion and self-deleted, which makes `self.handle` refer to a freed
    /// TCB. Always `false` for handles this crate did not spawn - there is no
    /// way to observe their lifetime.
    fn is_finished(&self) -> bool {
        self.join_state
            .as_ref()
            .is_some_and(|state| state.finished.load(Ordering::Acquire))
    }
}

impl ThreadFn for Thread {

    fn is_null(&self) -> bool {
        self.handle.is_null()
    }

    /// Spawns a new thread with a callback.
    /// 
    /// # Important
    /// The callback must be `'static`, which means it cannot borrow local variables.
    /// Use `move` in the closure to transfer ownership of any captured values:
    /// 
    /// ```ignore
    /// let data = Arc::new(Mutex::new(0));
    /// let thread = Thread::new("my_thread", 4096, 3, move |_thread, _param| {
    ///     // Use 'move' to capture 'data' by value
    ///     let mut guard = data.lock().unwrap();
    ///     *guard += 1;
    ///     Ok(Arc::new(()))
    /// });
    /// ``
    fn spawn<F>(&mut self, param: Option<ThreadParam>, callback: F) -> Result<Self> 
        where 
        F: Fn(Box<dyn ThreadFn>, Option<ThreadParam>) -> Result<ThreadParam>,
        F: Send + Sync + 'static {

        let mut handle: ThreadHandle =  null_mut();

        let func: Arc<ThreadFnPtr> = Arc::new(callback);
        
        self.callback = Some(func);
        self.param = param.clone();

        // Allocated before `xTaskCreate` so the task's own copy is already in
        // place by the time it can run.
        let join_state = JoinState::new()?;
        self.join_state = Some(join_state.clone());

        let boxed_thread = Box::new(self.clone());

        let ret = unsafe {
            xTaskCreate(
                Some(super::thread::callback_c_wrapper),
                self.name.as_cstr().as_ptr(),
                self.stack_depth,
                Box::into_raw(boxed_thread) as *mut _,
                self.priority,
                &mut handle,
            )
        };

        if ret != pdPASS {
            self.join_state = None;
            return Err(Error::OutOfMemory)
        }

        Ok(Self { 
            handle,
            callback: self.callback.clone(),
            param,
            join_state: Some(join_state),
            ..self.clone()
        })
    }

    /// Spawns a new thread with a simple closure, similar to `std::thread::spawn`.
    /// This is the recommended way to create threads for most use cases.
    /// 
    /// # Example
    /// ```ignore
    /// let counter = Arc::new(Mutex::new(0));
    /// let counter_clone = Arc::clone(&counter);
    /// 
    /// let handle = Thread::spawn_simple("worker", 4096, 3, move || {
    ///     let mut num = counter_clone.lock().unwrap();
    ///     *num += 1;
    /// }).unwrap();
    /// 
    /// handle.join(core::ptr::null_mut());
    /// ```
    fn spawn_simple<F>(&mut self, callback: F) -> Result<Self>
    where
        F: Fn() -> Result<ThreadParam> + Send + Sync + 'static,
    {
        let func: Arc<ThreadSimpleFnPtr> = Arc::new(callback);

        // Allocated before `xTaskCreate` so the task's own copy is already in
        // place by the time it can run.
        let join_state = JoinState::new()?;
        let boxed_func = Box::new((func, join_state.clone()));
        
        let mut handle: ThreadHandle = null_mut();

        let ret = unsafe {
            xTaskCreate(
                Some(simple_callback_wrapper),
                self.name.as_cstr().as_ptr(),
                self.stack_depth,
                Box::into_raw(boxed_func) as *mut _,
                self.priority,
                &mut handle,
            )
        };

        if ret != pdPASS {
            return Err(Error::OutOfMemory);
        }

        Ok(Self {
            handle,
            join_state: Some(join_state),
            ..self.clone()
        })
    }

    /// Deletes the thread and frees its resources.
    ///
    /// # Safety
    ///
    /// After calling this, the thread handle becomes invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn};
    /// 
    /// let thread = Thread::new("temp", 2048, 5);
    /// thread.delete();
    /// ```
    fn delete(&self) {
        if self.is_null() {
            return;
        }

        // A task spawned through this crate deletes itself once its callback
        // returns, so its handle refers to a freed TCB from that moment on.
        // Deleting it a second time would act on freed memory.
        if self.is_finished() {
            return;
        }

        unsafe { vTaskDelete( self.handle ); }
    }

    /// Suspends the thread execution.
    ///
    /// The thread remains suspended until `resume()` is called.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn};
    /// use core::time::Duration;
    /// 
    /// let thread = get_some_thread();
    /// thread.suspend();
    /// Duration::from_secs(1).sleep();
    /// thread.resume();
    /// ```
    fn suspend(&self) {
        // Same rationale as `delete`: suspending a task that already deleted
        // itself would act on a freed TCB.
        if !self.is_null() && !self.is_finished() {
            unsafe { vTaskSuspend( self.handle ); }
        }
    }

    /// Resumes a previously suspended thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// thread.resume();
    /// ```
    fn resume(&self) {
        // See `suspend`.
        if !self.is_null() && !self.is_finished() {
            unsafe { vTaskResume( self.handle ); }
        }
    }

    /// Blocks until the thread's callback returns, then hands back its exit
    /// value through `ret_val` if that pointer is non-null.
    ///
    /// FreeRTOS has no `pthread_join`, so this waits on the per-thread exit
    /// latch [`JoinState`] that `spawn`/`spawn_simple` allocate. The task
    /// deletes itself once its callback returns, so - unlike the older
    /// behaviour of this method - joining does *not* delete anything, and
    /// calling [`ThreadFn::delete`] afterwards is a no-op rather than a
    /// double delete.
    ///
    /// # Returns
    ///
    /// * `Ok(0)` - the thread finished and its exit value has been collected
    /// * `Err(Error::NullPtr)` - this handle refers to no thread
    /// * `Err(Error::TaskNotFound)` - the thread was not spawned through this
    ///   crate (nothing to wait on), or it has already been joined
    // The out-parameter is written through directly here, where the POSIX
    // backend hands the same pointer to `pthread_join` for libc to write. The
    // safety contract is identical either way and comes from `ThreadFn::join`
    // itself: `ret_val` must be null or point to a writable `*mut c_void`.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn join(&self, ret_val: DoublePtr) -> Result<i32> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        // Only threads spawned through this crate carry an exit latch. A
        // foreign task wrapped with `new_with_handle` has nothing to wait on,
        // so say so rather than silently deleting it - which is what this
        // used to do, and what made `join` mean the opposite of `pthread_join`.
        let Some(join_state) = &self.join_state else {
            return Err(Error::TaskNotFound);
        };

        // A second join would block forever on an already-consumed `done`;
        // report it instead, as `pthread_join` does with `ESRCH`.
        if join_state.joined.swap(true, Ordering::AcqRel) {
            return Err(Error::TaskNotFound);
        }

        // Already finished: `publish` left `done` signalled, so this returns
        // at once. Still running: block until it does.
        join_state.done.wait(MAX_DELAY);

        let raw = join_state.retval.swap(null_mut(), Ordering::Acquire);

        if !ret_val.is_null() {
            // Hands ownership of the boxed `Result<ThreadParam>` to the
            // caller, exactly as the POSIX backend does through
            // `pthread_join`'s out-parameter.
            unsafe { *ret_val = raw; }
        } else if !raw.is_null() {
            // Nobody asked for the exit value: free it rather than leak.
            // SAFETY: `raw` came from `Box::into_raw` in `JoinState::publish`.
            drop(unsafe { Box::from_raw(raw as *mut Result<ThreadParam>) });
        }

        Ok(0)
    }

    /// Retrieves this thread's metadata.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn};
    /// 
    /// let thread = Thread::current();
    /// let meta = thread.get_metadata();
    /// println!("Running thread: {}", meta.name);
    /// ```
    fn get_metadata(&self) -> ThreadMetadata {
        let mut status = TaskStatus::default();
        unsafe {
            vTaskGetInfo(self.handle, &mut status, pdTRUE, INVALID);
        }
        ThreadMetadata::from((self.handle, status))
    }

    /// Returns a Thread object representing the currently executing thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn};
    /// 
    /// let current = Thread::get_current();
    /// println!("Current thread: {}", current.get_metadata().name);
    /// ```
    fn get_current() -> Self {
        let handle = unsafe { xTaskGetCurrentTaskHandle() };
        let metadata = Self::get_metadata_from_handle(handle);
        Self {
            handle,
            name: metadata.name.clone(),
            stack_depth: metadata.stack_depth,
            priority: metadata.priority,
            callback: None,
            param: None,
            join_state: None,
        }
    }

    /// Sends a notification to this thread.
    ///
    /// # Parameters
    ///
    /// * `notification` - Type of notification action to perform
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Notification sent successfully
    /// * `Err(Error::NullPtr)` - Thread handle is null
    /// * `Err(Error::QueueFull)` - Notification failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn, ThreadNotification};
    /// 
    /// let thread = get_worker_thread();
    /// thread.notify(ThreadNotification::SetValueWithOverwrite(42)).unwrap();
    /// ```
    fn notify(&self, notification: ThreadNotification) -> Result<()> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        let (action, value) = notification.into();

        let ret = xTaskNotify!(
            self.handle,
            value,
            action
        );
        
        if ret != pdPASS {
            Err(Error::QueueFull)
        } else {
            Ok(())
        }

    }

    /// Sends a notification to this thread from an ISR.
    ///
    /// # Parameters
    ///
    /// * `notification` - Type of notification action
    /// * `higher_priority_task_woken` - Set to pdTRUE if a higher priority task was woken
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Notification sent successfully
    /// * `Err(Error::NullPtr)` - Thread handle is null
    /// * `Err(Error::QueueFull)` - Notification failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In ISR context:
    /// let mut woken = pdFALSE;
    /// thread.notify_from_isr(ThreadNotification::Increment, &mut woken).ok();
    /// ```
    fn notify_from_isr(&self, notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        let (action, value) = notification.into();

        let ret = xTaskNotifyFromISR!(
            self.handle,
            value,
            action,
            higher_priority_task_woken
        );

        if ret != pdPASS {
            Err(Error::QueueFull)
        } else {
            Ok(())
        }
    }

    /// Waits for a thread notification.
    ///
    /// # Parameters
    ///
    /// * `bits_to_clear_on_entry` - Bits to clear in notification value before waiting
    /// * `bits_to_clear_on_exit` - Bits to clear after receiving notification
    /// * `timeout_ticks` - Maximum ticks to wait
    ///
    /// # Returns
    ///
    /// * `Ok(u32)` - Notification value received
    /// * `Err(Error::NullPtr)` - Thread handle is null
    /// * `Err(Error::Timeout)` - No notification within timeout
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Thread, ThreadFn};
    /// 
    /// let thread = Thread::current();
    /// match thread.wait_notification(0, 0xFFFFFFFF, 1000) {
    ///     Ok(value) => println!("Received notification: {}", value),
    ///     Err(_) => println!("Timeout waiting for notification"),
    /// }
    /// ```
    fn wait_notification(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32 , timeout_ticks: TickType) -> Result<u32> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        let mut notification_value: u32 = 0;

        let ret = xTaskNotifyWait!(
            bits_to_clear_on_entry,
            bits_to_clear_on_exit,
            &mut notification_value,
            timeout_ticks
        );
        

        if ret == pdTRUE {
            Ok(notification_value)
        } else {
            Err(Error::Timeout)
        }
    }

}


// impl Drop for Thread {
//     fn drop(&mut self) {
//         if !self.handle.is_null() {
//             unsafe { vTaskDelete( self.handle ); } 
//         }
//     }
// }

/// Allows dereferencing to the underlying FreeRTOS thread handle.
///
/// This enables direct access to the handle when needed for low-level operations.
impl Deref for Thread {
    type Target = ThreadHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// Formats the thread for debugging purposes.
///
/// Includes handle, name, stack depth, priority, and callback status.
impl Debug for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thread")
            .field("handle", &self.handle)
            .field("name", &self.name)
            .field("stack_depth", &self.stack_depth)
            .field("priority", &self.priority)
            .field("callback", &self.callback.as_ref().map(|_| "Some(...)"))
            .field("param", &self.param)
            .finish()
    }
}

/// Formats the thread for display purposes.
///
/// Shows a concise representation with handle, name, priority, and stack depth.
impl Display for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread {{ handle: {:?}, name: {}, priority: {}, stack_depth: {} }}", self.handle, self.name, self.priority, self.stack_depth)
    }
}


