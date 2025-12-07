use core::ptr::null_mut;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use crate::freertos::ffi::{ThreadHandle, vTaskDelete};
use crate::freertos::types::{StackType, UBaseType};
use crate::traits::{ThreadFn, ThreadParam, ThreadFnPtr};
use crate::utils::Result;

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
    pub thread: Thread,
    pub thread_number: UBaseType,
    pub state: ThreadState,
    pub current_priority: UBaseType,
    pub base_priority: UBaseType,
    pub run_time_counter: UBaseType,
    pub stack_high_water_mark: StackType,
}


#[derive(Clone)]
pub struct Thread {
    handle: ThreadHandle,
    name: String,
    stack_depth: StackType,
    priority: UBaseType,
    callback: Option<Arc<ThreadFnPtr>>,
    param: ThreadParam
}

impl ThreadFn for Thread {
    fn new<F>(name: &str, stack_depth: StackType, priority: UBaseType, f: Option<F>) -> Self 
    where 
        F: Fn(ThreadParam) -> Result<ThreadParam>,
        F: Send + Sync + 'static
    {
        Self { 
            handle: null_mut(), 
            name: name.to_string(), 
            stack_depth, 
            priority, 
            callback: if let Some(f) = f {
                Some(Arc::new(f))
            } else {
                None
            }, 
            param: None 
        }
    }

    fn new_with_handle(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: UBaseType) -> Self {
        Self { 
            handle, 
            name: name.to_string(), 
            stack_depth, 
            priority, 
            callback: None,
            param: None 
        }
    }

    fn spawn(&mut self, param: ThreadParam) -> Result<Self>
    where 
        Self: Sized {
        todo!()
    }

    fn suspend(&self) {
        todo!()
    }

    fn resume(&self) {
        todo!()
    }

    fn join(&self, retval: super::types::DoublePtr) -> Result<i32> {
        todo!()
    }
}


impl Drop for Thread {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { vTaskDelete( self.handle ); } 
        }
    }
}

impl core::fmt::Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

unsafe impl Send for Thread {}
