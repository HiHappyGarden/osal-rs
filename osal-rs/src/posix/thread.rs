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

use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null;

use alloc::sync::Arc;

use crate::posix::types::{BaseType, StackType, ThreadHandle, TickType, UBaseType};
use crate::traits::{ThreadFn, ThreadFnPtr, ThreadNotification, ThreadParam, ToPriority, ToTick};
use crate::utils::{Bytes, DoublePtr, Error, Result};

const MAX_TASK_NAME_LEN: usize = 16;

fn dummy_thread_handle() -> ThreadHandle {
    1usize as ThreadHandle
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ThreadState {
    Running = 0,
    Ready = 1,
    Blocked = 2,
    Suspended = 3,
    Deleted = 4,
    Invalid,
}

#[derive(Clone, Debug)]
pub struct ThreadMetadata {
    pub thread: ThreadHandle,
    pub name: Bytes<MAX_TASK_NAME_LEN>,
    pub stack_depth: StackType,
    pub priority: UBaseType,
    pub thread_number: UBaseType,
    pub state: ThreadState,
    pub current_priority: UBaseType,
    pub base_priority: UBaseType,
    pub run_time_counter: UBaseType,
    pub stack_high_water_mark: StackType,
}

unsafe impl Send for ThreadMetadata {}
unsafe impl Sync for ThreadMetadata {}

impl Default for ThreadMetadata {
    fn default() -> Self {
        Self {
            thread: null(),
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
            handle: null(),
            name: Bytes::from_str(name),
            stack_depth,
            priority,
            callback: None,
            param: None,
        }
    }

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
        })
    }

    pub fn new_with_to_priority(name: &str, stack_depth: StackType, priority: impl ToPriority) -> Self {
        Self::new(name, stack_depth, priority.to_priority())
    }

    pub fn new_with_handle_and_to_priority(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: impl ToPriority) -> Result<Self> {
        Self::new_with_handle(handle, name, stack_depth, priority.to_priority())
    }

    pub fn get_metadata_from_handle(handle: ThreadHandle) -> ThreadMetadata {
        if handle.is_null() {
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
        if thread.handle.is_null() {
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

        if self.handle.is_null() {
            self.handle = dummy_thread_handle();
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
        let _ = callback;

        if self.handle.is_null() {
            self.handle = dummy_thread_handle();
        }

        Ok(self.clone())
    }

    fn delete(&self) {}

    fn suspend(&self) {}

    fn resume(&self) {}

    fn join(&self, _retval: DoublePtr) -> Result<i32> {
        Ok(0)
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
        if self.handle.is_null() {
            Err(Error::NullPtr)
        } else {
            Ok(())
        }
    }

    fn notify_from_isr(&self, _notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()> {
        *higher_priority_task_woken = 0;

        if self.handle.is_null() {
            Err(Error::NullPtr)
        } else {
            Ok(())
        }
    }

    fn wait_notification(&self, _bits_to_clear_on_entry: u32, _bits_to_clear_on_exit: u32, _timeout_ticks: TickType) -> Result<u32> {
        if self.handle.is_null() {
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