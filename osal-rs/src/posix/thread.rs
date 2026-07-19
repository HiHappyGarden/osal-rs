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

use core::ffi::c_void;
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null_mut;

use alloc::sync::Arc;

use crate::os::ThreadSimpleFnPtr;
#[cfg(feature = "sched_fifo")]
use crate::posix::ffi::{PTHREAD_EXPLICIT_SCHED, SCHED_FIFO, pthread_attr_setinheritsched, pthread_attr_setschedparam, pthread_attr_setschedpolicy, sched_param};
use crate::posix::ffi::{get_pthread_stack_min, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t, pthread_create, pthread_join, pthread_self, pthread_setname_np};
use crate::posix::types::{BaseType, StackType, ThreadHandle, TickType, UBaseType};
use crate::traits::{ThreadFn, ThreadFnPtr, ThreadNotification, ThreadParam, ToPriority, ToTick};
use crate::utils::{Bytes, DoublePtr, Error, Result};

const MAX_TASK_NAME_LEN: usize = 16;

fn dummy_thread_handle() -> ThreadHandle {
    todo!("To remove");
}

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
    pub name: Bytes<MAX_TASK_NAME_LEN>,
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

/// Provides default values for ThreadMetadata.
///
/// Creates a metadata instance with null/zero values, representing an
/// invalid or uninitialized thread.
impl Default for ThreadMetadata {
    fn default() -> Self {
        Self {
            thread: 0,
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

#[derive(Clone)]
pub struct Thread {
    handle: ThreadHandle,
    name: Bytes<MAX_TASK_NAME_LEN>,
    stack_depth: StackType,
    priority: UBaseType,
    callback: Option<Arc<ThreadFnPtr>>,
    param: Option<ThreadParam>,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    pub fn new(name: &str, stack_depth: StackType, priority: UBaseType) -> Self {
        Self {
            handle: 0,
            name: Bytes::from_str(name),
            stack_depth,
            priority,
            callback: None,
            param: None,
        }
    }

    pub fn new_with_handle(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: UBaseType) -> Result<Self> {
        if handle == 0 {
            return Err(Error::NullPtr);
        }

        Ok(Self {
            handle,
            name: Bytes::from_str(name),
            stack_depth,
            priority,
            callback: None,
            param: None,
        })
    }

    pub fn new_with_to_priority(name: &str, stack_depth: StackType, priority: impl ToPriority) -> Self {
        Self::new(name, stack_depth, priority.to_priority())
    }

    pub fn new_with_handle_and_to_priority(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: impl ToPriority) -> Result<Self> {
        Self::new_with_handle(handle, name, stack_depth, priority.to_priority())
    }

    pub fn get_metadata_from_handle(handle: ThreadHandle) -> ThreadMetadata {
        if handle == 0 {
            return ThreadMetadata::default();
        }

        ThreadMetadata {
            thread: handle,
            name: Bytes::from_str("thread"),
            stack_depth: 0,
            priority: 0,
            thread_number: 0,
            state: ThreadState::Ready,
            current_priority: 0,
            base_priority: 0,
            run_time_counter: 0,
            stack_high_water_mark: 0,
        }
    }

    pub fn get_metadata(thread: &Thread) -> ThreadMetadata {
        if thread.handle == 0 {
            ThreadMetadata::default()
        } else {
            ThreadMetadata {
                thread: thread.handle,
                name: thread.name.clone(),
                stack_depth: thread.stack_depth,
                priority: thread.priority,
                thread_number: 0,
                state: ThreadState::Ready,
                current_priority: thread.priority,
                base_priority: thread.priority,
                run_time_counter: 0,
                stack_high_water_mark: 0,
            }
        }
    }

    #[inline]
    pub fn wait_notification_with_to_tick(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32, timeout_ticks: impl ToTick) -> Result<u32> {
        self.wait_notification(bits_to_clear_on_entry, bits_to_clear_on_exit, timeout_ticks.to_ticks())
    }

    fn metadata(&self) -> ThreadMetadata {
        Self::get_metadata(self)
    }
}

/// Internal C-compatible wrapper for thread callbacks.
///
/// Bridges between the pthreads C API and Rust closures. It unpacks the
/// boxed thread instance, resolves the thread's own handle via
/// `pthread_self()` (avoiding any race with `pthread_create()`'s caller
/// writing `*thread`, which may not have happened yet once this routine
/// starts running), and invokes the user-provided callback.
///
/// The callback's `Result<ThreadParam>` is boxed and returned as the raw
/// `void *` the pthreads API uses for a thread's exit value: whoever calls
/// `Thread::join()` on this thread receives this same pointer back and can
/// reconstruct it with `Box::from_raw(ptr as *mut Result<ThreadParam>)`.
///
/// # Safety
///
/// - `param_ptr` must be a valid pointer produced by `Box::into_raw` on a `Thread`
/// - Called only by `pthread_create()` as the thread's start routine
unsafe extern "C" fn callback_c_wrapper(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut();
    }

    let mut thread_instance: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut _) };

    thread_instance.as_mut().handle = unsafe { pthread_self() };

    let thread = *thread_instance.clone();

    let param_arc: Option<ThreadParam> = thread_instance.param.clone();

    let ret = if let Some(callback) = &thread_instance.callback.clone() {
        callback(thread_instance, param_arc)
    } else {
        Err(Error::NullPtr)
    };

    thread.delete();

    Box::into_raw(Box::new(ret)) as *mut c_void
}

/// Internal C-compatible wrapper for simple (parameter-less) thread callbacks.
///
/// Unpacks the boxed `Arc<ThreadSimpleFnPtr>` and invokes it directly; unlike
/// [`callback_c_wrapper`] there is no `Thread` instance to reconstruct here.
///
/// # Safety
///
/// - `param_ptr` must be a valid pointer produced by `Box::into_raw` on an `Arc<ThreadSimpleFnPtr>`
/// - Called only by `pthread_create()` as the thread's start routine
unsafe extern "C" fn simple_callback_c_wrapper(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut();
    }

    let func: Box<Arc<ThreadSimpleFnPtr>> = unsafe { Box::from_raw(param_ptr as *mut _) };
    func();

    null_mut()
}

impl ThreadFn for Thread {
    fn spawn<F>(&mut self, param: Option<ThreadParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn ThreadFn>, Option<ThreadParam>) -> Result<ThreadParam>,
        F: Send + Sync + 'static,
        Self: Sized,
    {
        let func: Arc<ThreadFnPtr> = Arc::new(callback);
        self.callback = Some(func);
        self.param = param.clone();

        let mut attr: pthread_attr_t = Default::default();

        unsafe {
            pthread_attr_init (&mut attr);
        }

        let requested_stack_size = unsafe {
            get_pthread_stack_min()
        } + self.stack_depth as usize;

        let min_safe_stack_size = 1024usize * 1024usize;

        unsafe {
            pthread_attr_setstacksize (&mut attr, if requested_stack_size < min_safe_stack_size {  min_safe_stack_size } else { requested_stack_size });
        }

        #[cfg(feature = "sched_fifo")]
        unsafe {
            let fifo_param = sched_param {
                sched_priority: self.priority as core::ffi::c_int,
            };
            pthread_attr_setinheritsched(&mut attr, PTHREAD_EXPLICIT_SCHED);
            pthread_attr_setschedpolicy(&mut attr, SCHED_FIFO);
            pthread_attr_setschedparam(&mut attr, &fifo_param);
        }

        let boxed_thread = Box::new(self.clone());

        let ret = unsafe {
            pthread_create(&mut self.handle, &attr, Some(callback_c_wrapper), Box::into_raw(boxed_thread) as *mut c_void)
        };

        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        unsafe {
            pthread_setname_np(self.handle, self.name.as_cstr().as_ptr());
        }

        Ok(Self {
            handle: self.handle,
            name: self.name.clone(),
            stack_depth: self.stack_depth,
            priority: self.priority,
            callback: self.callback.clone(),
            param,
        })
    }

    fn spawn_simple<F>(&mut self, callback: F) -> Result<Self>
    where
        F: Fn() + Send + Sync + 'static,
        Self: Sized,
    {
        let func: Arc<ThreadSimpleFnPtr> = Arc::new(callback);
        let boxed_func = Box::new(func);


        let mut attr: pthread_attr_t = Default::default();

        unsafe {
            pthread_attr_init (&mut attr);
        }

        let requested_stack_size = unsafe {
            get_pthread_stack_min()
        } + self.stack_depth as usize;

        let min_safe_stack_size = 1024usize * 1024usize;

        unsafe {
            pthread_attr_setstacksize (&mut attr, if requested_stack_size < min_safe_stack_size {  min_safe_stack_size } else { requested_stack_size });
        }

        #[cfg(feature = "sched_fifo")]
        unsafe {
            let fifo_param = sched_param {
                sched_priority: self.priority as core::ffi::c_int,
            };
            pthread_attr_setinheritsched(&mut attr, PTHREAD_EXPLICIT_SCHED);
            pthread_attr_setschedpolicy(&mut attr, SCHED_FIFO);
            pthread_attr_setschedparam(&mut attr, &fifo_param);
        }

        let ret = unsafe {
            pthread_create(&mut self.handle, &attr, Some(simple_callback_c_wrapper), Box::into_raw(boxed_func) as *mut c_void)
        };

        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        unsafe {
            pthread_setname_np(self.handle, self.name.as_cstr().as_ptr());
        }

        Ok(Self {
            handle: self.handle,
            name: self.name.clone(),
            stack_depth: self.stack_depth,
            priority: self.priority,
            callback: self.callback.clone(),
            param: self.param.clone(),
        })
    }

    fn delete(&self) {}

    fn suspend(&self) {}

    fn resume(&self) {}

    fn join(&self, retval: DoublePtr) -> Result<i32> {
        if self.handle == 0 {
            return Err(Error::NullPtr);
        }

        let ret = unsafe { pthread_join(self.handle, retval) };

        if ret != 0 {
            Err(Error::ReturnWithCode(ret))
        } else {
            Ok(0)
        }
    }

    fn get_metadata(&self) -> ThreadMetadata {
        self.metadata()
    }

    fn get_current() -> Self
    where
        Self: Sized,
    {
        Self {
            handle: dummy_thread_handle(),
            name: Bytes::from_str("current"),
            stack_depth: 0,
            priority: 0,
            callback: None,
            param: None,
        }
    }

    fn notify(&self, _notification: ThreadNotification) -> Result<()> {
        if self.handle == 0 {
            Err(Error::NullPtr)
        } else {
            Ok(())
        }
    }

    fn notify_from_isr(&self, _notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()> {
        *higher_priority_task_woken = 0;

        if self.handle == 0 {
            Err(Error::NullPtr)
        } else {
            Ok(())
        }
    }

    fn wait_notification(&self, _bits_to_clear_on_entry: u32, _bits_to_clear_on_exit: u32, _timeout_ticks: TickType) -> Result<u32> {
        if self.handle == 0 {
            Err(Error::NullPtr)
        } else {
            Err(Error::Timeout)
        }
    }
}

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
        write!(
            f,
            "Thread {{ handle: {:?}, name: {}, priority: {}, stack_depth: {} }}",
            self.handle,
            self.name,
            self.priority,
            self.stack_depth
        )
    }
}