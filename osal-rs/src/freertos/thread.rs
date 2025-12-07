


use alloc::string::{String, ToString};

use crate::{freertos::ffi::{TaskStatus, ThreadHandle}, types::{StackType, UBaseType}};

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

#[derive(Debug, Clone)]
pub struct ThreadMetadata {
    pub thread: Thread,
    pub thread_number: UBaseType,
    pub state: ThreadState,
    pub current_priority: UBaseType,
    pub base_priority: UBaseType,
    pub run_time_counter: UBaseType,
    pub stack_high_water_mark: StackType,
}


#[derive(Debug, Clone)]
pub struct Thread {
    handle: ThreadHandle,
    name: String,
}

impl Thread {
    pub fn new(handle: ThreadHandle, name: String) -> Self {
        Self { 
            handle, 
            name: name
        }
    }

    pub fn get_metadata(t: &TaskStatus) -> ThreadMetadata {
        ThreadMetadata {
            thread: Thread::new(t.xHandle, 
            unsafe {
                    let c_str = core::ffi::CStr::from_ptr(t.pcTaskName);
                    String::from_utf8_lossy(c_str.to_bytes()).to_string()
                }),
            thread_number: t.xTaskNumber,
            state: match t.eCurrentState {
                // RUNNING => ThreadState::Running,
                // READY => ThreadState::Ready,
                // BLOCKED => ThreadState::Blocked,
                // SUSPENDED => ThreadState::Suspended,
                // DELETED => ThreadState::Deleted,
                _ => ThreadState::Invalid,
            },
            current_priority: t.uxCurrentPriority,
            base_priority: t.uxBasePriority,
            run_time_counter: t.ulRunTimeCounter,
            stack_high_water_mark: t.usStackHighWaterMark,
        }
    }
    
}

unsafe impl Send for Thread {}
