use core::any::Any;
use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::os::{ThreadMetadata};
use crate::os::types::{BaseType, ConstPtr, DoublePtr, StackType, TickType, UBaseType};
use crate::utils::Result;

pub type ThreadParam = Arc<dyn Any + Send + Sync>;
pub type ThreadFnPtr = dyn Fn(Box<dyn Thread>, Option<ThreadParam>) -> Result<ThreadParam> + Send + Sync + 'static;

#[derive(Debug, Copy, Clone)]
pub enum ThreadNotification {
    NoAction,
    SetBits(u32),
    Increment,
    SetValueWithOverwrite(u32),
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
    fn new<F>(name: &str, stack_depth: StackType, priority: UBaseType, callback: F) -> Self 
    where 
        F: Fn(Box<dyn Thread>, Option<ThreadParam>) -> Result<ThreadParam>,
        F: Send + Sync + 'static,
        Self: Sized;

    fn new_with_handle(handle: ConstPtr, name: &str, stack_depth: StackType, priority: UBaseType) -> Result<Self>  
    where 
        Self: Sized;

    fn spawn(&mut self, param: Option<ThreadParam>) -> Result<Self>
    where 
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