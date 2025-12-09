use core::any::Any;
use alloc::sync::Arc;
use crate::os::{ThreadMetadata, ToTick};
use crate::os::types::{BaseType, ConstPtr, DoublePtr, StackType, UBaseType};
use crate::utils::Result;

pub type ThreadParam = Option<Arc<dyn Any + Send + Sync>>;
pub type ThreadFnPtr = dyn Fn(ThreadParam) -> Result<ThreadParam> + Send + Sync + 'static;

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
        match self {
            ThreadNotification::NoAction => (0, 0),
            ThreadNotification::SetBits(bits) => (1, bits),
            ThreadNotification::Increment => (2, 0),
            ThreadNotification::SetValueWithOverwrite(value) => (3, value),
            ThreadNotification::SetValueWithoutOverwrite(value) => (4, value),
        }
    }
}

pub trait Thread {
    fn new<F>(name: &str, stack_depth: StackType, priority: UBaseType, f: Option<F>) -> Self 
    where 
        F: Fn(ThreadParam) -> Result<ThreadParam>,
        F: Send + Sync + 'static,
        Self: Sized;

    fn new_with_handle(handle: ConstPtr, name: &str, stack_depth: StackType, priority: UBaseType) -> Result<Self>  
    where 
        Self: Sized;

    fn spawn(&mut self, param: ThreadParam) -> Result<Self>
    where 
        Self: Sized;

    fn delete(&self);

    fn suspend(&self);

    fn resume(&self);

    fn join(&self, retval: DoublePtr) -> Result<i32>;

    fn get_metadata(handle: ConstPtr) -> ThreadMetadata;

    fn get_current() -> Self
    where 
        Self: Sized;

    fn notify(&self, notification: ThreadNotification) -> Result<()>;

    fn notify_from_isr(&self, notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()>;

    fn wait_notification(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32 , timeout_ticks: impl ToTick) -> Result<u32>;


}