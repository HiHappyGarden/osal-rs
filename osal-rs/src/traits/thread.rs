use core::any::Any;

use alloc::sync::Arc;

use crate::os::ThreadMetadata;
use crate::os::types::{DoublePtr, ConstPtr, StackType, UBaseType};
use crate::utils::Result;

pub type ThreadParam = Option<Arc<dyn Any + Send + Sync>>;
pub type ThreadFnPtr = dyn Fn(ThreadParam) -> Result<ThreadParam> + Send + Sync + 'static;

pub trait Thread {
    fn new<F>(name: &str, stack_depth: StackType, priority: UBaseType, f: Option<F>) -> Self 
    where 
        F: Fn(ThreadParam) -> Result<ThreadParam>,
        F: Send + Sync + 'static,
        Self: Sized;

    fn new_with_handle(handle: ConstPtr, name: &str, stack_depth: StackType, priority: UBaseType) -> Self 
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
}