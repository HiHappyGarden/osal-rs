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

//! Thread management and synchronization for FreeRTOS.
//!
//! This module provides a safe Rust interface for creating and managing FreeRTOS tasks.
//! It supports thread creation with callbacks, priority management, and thread notifications.

use core::any::Any;
use core::ffi::{c_char, c_void};
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null_mut;

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use super::ffi::{INVALID, TaskStatus, ThreadHandle, pdPASS, pdTRUE, vTaskDelete, vTaskGetInfo, vTaskResume, vTaskSuspend, xTaskCreate, xTaskGetCurrentTaskHandle};
use super::types::{StackType, UBaseType, BaseType, TickType};
use super::thread::ThreadState::*;
use crate::os::ThreadSimpleFnPtr;
use crate::traits::{ThreadFn, ThreadParam, ThreadFnPtr, ThreadNotification, ToTick, ToPriority};
use crate::utils::{Result, Error, DoublePtr};
use crate::{from_c_str, xTaskNotify, xTaskNotifyFromISR, xTaskNotifyWait};

/// Represents the possible states of a FreeRTOS task/thread.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Thread, ThreadState};
/// 
/// let thread = Thread::current();
/// let metadata = thread.metadata().unwrap();
/// 
/// match metadata.state {
///     ThreadState::Running => println!("Thread is currently executing"),
///     ThreadState::Ready => println!("Thread is ready to run"),
///     ThreadState::Blocked => println!("Thread is waiting for an event"),
///     ThreadState::Suspended => println!("Thread is suspended"),
///     _ => println!("Unknown state"),
/// }
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
/// ```ignore
/// use osal_rs::os::Thread;
/// 
/// let thread = Thread::current();
/// let metadata = thread.metadata().unwrap();
/// 
/// println!("Thread: {}", metadata.name);
/// println!("Priority: {}", metadata.priority);
/// println!("Stack high water mark: {}", metadata.stack_high_water_mark);
/// ```
#[derive(Clone, Debug)]
pub struct ThreadMetadata {
    /// FreeRTOS task handle
    pub thread: ThreadHandle,
    /// Thread name
    pub name: String,
    /// Original stack depth allocated for this thread
    pub stack_depth: StackType,
    /// Thread priority
    pub priority: UBaseType,
    /// Unique thread number assigned by FreeRTOS
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
            name: from_c_str!(status.1.pcTaskName),
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

impl Default for ThreadMetadata {
    fn default() -> Self {
        ThreadMetadata {
            thread: null_mut(),
            name: String::new(),
            stack_depth: 0,
            priority: 0,
            thread_number: 0,
            state: Invalid,
            current_priority: 0,
            base_priority: 0,
            run_time_counter: 0,
            stack_high_water_mark: 0,
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
    name: String,
    stack_depth: StackType,
    priority: UBaseType,
    callback: Option<Arc<ThreadFnPtr>>,
    param: Option<ThreadParam>
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {

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
            name: name.to_string(), 
            stack_depth, 
            priority: priority.to_priority(), 
            callback: None,
            param: None 
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
            name: name.to_string(), 
            stack_depth, 
            priority: priority.to_priority(), 
            callback: None,
            param: None 
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
        if thread.handle.is_null() {
            return ThreadMetadata::default();
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
        if self.handle.is_null() {
            return Err(Error::NullPtr);
        }
        self.wait_notification(bits_to_clear_on_entry, bits_to_clear_on_exit, timeout_ticks.to_ticks())
    }

}

unsafe extern "C" fn callback_c_wrapper(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    let mut thread_instance: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut _) };

    thread_instance.as_mut().handle = unsafe { xTaskGetCurrentTaskHandle() };

    let thread = *thread_instance.clone();

    let param_arc: Option<ThreadParam> = thread_instance
        .param
        .clone();

    if let Some(callback) = &thread_instance.callback.clone() {
        let _ = callback(thread_instance, param_arc);
    }

    thread.delete();
}

unsafe extern "C" fn simple_callback_wrapper(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    let func: Box<Arc<ThreadSimpleFnPtr>> = unsafe { Box::from_raw(param_ptr as *mut _) };
    func();

    unsafe { vTaskDelete( xTaskGetCurrentTaskHandle()); } 
}



impl ThreadFn for Thread {
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
    fn new(name: &str, stack_depth: StackType, priority: UBaseType) -> Self 
    {
        Self { 
            handle: null_mut(), 
            name: name.to_string(), 
            stack_depth, 
            priority, 
            callback: None,
            param: None 
        }
    }

    /// Creates a thread from an existing task handle.
    ///
    /// # Returns
    ///
    /// * `Err(Error::NullPtr)` if handle is null
    fn new_with_handle(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: UBaseType) -> Result<Self> {
        if handle.is_null() {
            return Err(Error::NullPtr);
        }
        Ok(Self { 
            handle, 
            name: name.to_string(), 
            stack_depth, 
            priority, 
            callback: None,
            param: None 
        })
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

        let boxed_thread = Box::new(self.clone());

        // Convert name to CString to ensure null termination and proper lifetime
        let c_name = CString::new(self.name.as_str())
            .map_err(|_| Error::Unhandled("Failed to convert thread name to CString"))?;

        let ret = unsafe {
            xTaskCreate(
                Some(super::thread::callback_c_wrapper),
                c_name.as_ptr(),
                self.stack_depth,
                Box::into_raw(boxed_thread) as *mut _,
                self.priority,
                &mut handle,
            )
        };

        if ret != pdPASS {
            return Err(Error::OutOfMemory)
        }

        Ok(Self { 
            handle,
            callback: self.callback.clone(),
            param,
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
        F: Fn() + Send + Sync + 'static,
    {
        let func: Arc<ThreadSimpleFnPtr> = Arc::new(callback);
        let boxed_func = Box::new(func);
        
        let mut handle: ThreadHandle = null_mut();

        // Convert name to CString to ensure null termination and proper lifetime
        let c_name = CString::new(self.name.as_str())
            .map_err(|_| Error::Unhandled("Failed to convert thread name to CString"))?;

        let ret = unsafe {
            xTaskCreate(
                Some(simple_callback_wrapper),
                c_name.as_ptr(),
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
        if !self.handle.is_null() {
            unsafe { vTaskDelete( self.handle ); } 
        }
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
        if !self.handle.is_null() {
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
        if !self.handle.is_null() {
            unsafe { vTaskResume( self.handle ); } 
        }
    }

    /// Waits for the thread to complete (currently deletes the thread).
    ///
    /// # Returns
    ///
    /// Always returns `Ok(0)`
    fn join(&self, _retval: DoublePtr) -> Result<i32> {
        if !self.handle.is_null() {
            unsafe { vTaskDelete( self.handle ); } 
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
            name: metadata.name,
            stack_depth: metadata.stack_depth,
            priority: metadata.priority,
            callback: None,
            param: None,
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
        if self.handle.is_null() {
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
        if self.handle.is_null() {
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
        if self.handle.is_null() {
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

impl Deref for Thread {
    type Target = ThreadHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

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

impl Display for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thread {{ handle: {:?}, name: {}, priority: {}, stack_depth: {} }}", self.handle, self.name, self.priority, self.stack_depth)
    }
}


