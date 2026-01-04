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

//! Thread-related traits and type definitions.

use core::any::Any;
use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::os::{ThreadMetadata};
use crate::os::types::{BaseType, StackType, TickType, UBaseType};
use crate::utils::{Result, ConstPtr, DoublePtr};

/// Type-erased parameter that can be passed to thread callbacks.
///
/// Allows passing arbitrary data to thread functions in a thread-safe manner.
/// The parameter can be downcast to its original type using `downcast_ref()`.
pub type ThreadParam = Arc<dyn Any + Send + Sync>;

/// Thread callback function pointer type.
///
/// Thread callbacks receive a boxed thread handle and optional parameter,
/// and can return an updated parameter value.
pub type ThreadFnPtr = dyn Fn(Box<dyn Thread>, Option<ThreadParam>) -> Result<ThreadParam> + Send + Sync + 'static;

/// Simple thread function pointer type without parameters.
///
/// Used for basic thread functions that don't need access to the thread handle or parameters.
pub type ThreadSimpleFnPtr = dyn Fn() + Send + Sync + 'static;

/// Thread notification actions.
///
/// Defines different ways to notify a thread using the FreeRTOS task notification mechanism.
/// Task notifications provide a lightweight alternative to semaphores and queues for
/// simple signaling.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Thread, ThreadNotification};
/// 
/// let thread = Thread::current();
/// 
/// // Increment notification counter
/// thread.notify_with_action(ThreadNotification::Increment);
/// 
/// // Set specific bits
/// thread.notify_with_action(ThreadNotification::SetBits(0b1010));
/// 
/// // Set value, overwriting any existing value
/// thread.notify_with_action(ThreadNotification::SetValueWithOverwrite(42));
/// ```
#[derive(Debug, Copy, Clone)]
pub enum ThreadNotification {
    /// Don't update the notification value
    NoAction,
    /// Bitwise OR the notification value with the specified bits
    SetBits(u32),
    /// Increment the notification value by one
    Increment,
    /// Set the notification value, overwriting any existing value
    SetValueWithOverwrite(u32),
    /// Set the notification value only if the receiving thread has no pending notifications
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

pub trait Thread {
    fn new(name: &str, stack_depth: StackType, priority: UBaseType) -> Self 
    where
        Self: Sized;

    fn new_with_handle(handle: ConstPtr, name: &str, stack_depth: StackType, priority: UBaseType) -> Result<Self>  
    where 
        Self: Sized;

    fn spawn<F>(&mut self, param: Option<ThreadParam>, callback: F) -> Result<Self>
    where 
        F: Fn(Box<dyn Thread>, Option<ThreadParam>) -> Result<ThreadParam>,
        F: Send + Sync + 'static,
        Self: Sized;

    fn spawn_simple<F>(&mut self, callback: F) -> Result<Self>
    where
        F: Fn() + Send + Sync + 'static,
        Self: Sized;

    fn delete(&self);

    fn suspend(&self);

    fn resume(&self);

    fn join(&self, retval: DoublePtr) -> Result<i32>;

    fn get_metadata(&self) -> ThreadMetadata;

    fn get_current() -> Self
    where 
        Self: Sized;

    fn notify(&self, notification: ThreadNotification) -> Result<()>;

    fn notify_from_isr(&self, notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()>;

    fn wait_notification(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32 , timeout_ticks: TickType) -> Result<u32>; //no ToTick here to maintain dynamic dispatch


}

pub trait ToPriority {
    fn to_priority(&self) -> UBaseType;
}