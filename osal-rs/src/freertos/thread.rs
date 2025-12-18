use core::any::Any;
use core::ffi::c_void;
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null_mut;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use super::ffi::{INVALID, TaskStatus, ThreadHandle, pdPASS, pdTRUE, vTaskDelete, vTaskGetInfo, vTaskResume, vTaskSuspend, xTaskCreate, xTaskGetCurrentTaskHandle};
use super::types::{StackType, UBaseType, BaseType, DoublePtr, TickType};
use super::thread::ThreadState::*;
use crate::traits::{ThreadFn, ThreadParam, ThreadFnPtr, ThreadNotification, ToTick};
use crate::utils::{Result, Error};
use crate::{from_c_str, to_cstring, xTaskNotify, xTaskNotifyFromISR, xTaskNotifyWait};

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
    pub name: String,
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
            stack_depth: if status.1.pxStackBase.is_null() { 0 } else { unsafe { *status.1.pxStackBase } },
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

impl Thread {
    pub fn get_metadata_from_handle(handle: ThreadHandle) -> ThreadMetadata {
        let mut status = TaskStatus::default();
        unsafe {
            vTaskGetInfo(handle, &mut status, pdTRUE, INVALID);
        }
        ThreadMetadata::from((handle, status))
    }

    pub fn get_metadata(thread: &Thread) -> ThreadMetadata {
        if thread.handle.is_null() {
            return ThreadMetadata::default();
        }
        Self::get_metadata_from_handle(thread.handle)
    }

    #[inline]
    fn wait_notification_with_to_tick(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32 , timeout_ticks: impl ToTick) -> Result<u32> {
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

    let param_arc: Option<Arc<dyn Any + Send + Sync>> = thread_instance
        .param
        .clone();

    if let Some(callback) = &thread_instance.callback.clone() {
        let _ = callback(thread_instance, param_arc);
    }
}



impl ThreadFn for Thread {
    fn new<F>(name: &str, stack_depth: StackType, priority: UBaseType, callback: F) -> Self 
    where 
        F: Fn(Box<dyn ThreadFn>, Option<ThreadParam>) -> Result<ThreadParam>,
        F: Send + Sync + 'static
    {
        Self { 
            handle: null_mut(), 
            name: name.to_string(), 
            stack_depth, 
            priority, 
            callback: Some(Arc::new(callback)),
            param: None 
        }
    }

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

    fn spawn(&mut self, param: Option<ThreadParam>) -> Result<Self> {        
        let name = to_cstring!(self.name)?;

        let mut handle: ThreadHandle =  null_mut();

        let boxed_thread = Box::new(self.clone());

        let ret = unsafe {
            xTaskCreate(
                Some(super::thread::callback_c_wrapper),
                name.as_ptr(),
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
            name: self.name.clone(),
            stack_depth: self.stack_depth,
            priority: self.priority,
            callback: self.callback.clone(),
            param,
        })
    }

    fn delete(&self) {
        if !self.handle.is_null() {
            unsafe { vTaskDelete( self.handle ); } 
        }
    }

    fn suspend(&self) {
        if !self.handle.is_null() {
            unsafe { vTaskSuspend( self.handle ); } 
        }
    }

    fn resume(&self) {
        if !self.handle.is_null() {
            unsafe { vTaskResume( self.handle ); } 
        }
    }

    fn join(&self, _retval: DoublePtr) -> Result<i32> {
        if !self.handle.is_null() {
            unsafe { vTaskDelete( self.handle ); } 
        }
        Ok(0)
    }

    fn get_metadata(&self) -> ThreadMetadata {
        let mut status = TaskStatus::default();
        unsafe {
            vTaskGetInfo(self.handle, &mut status, pdTRUE, INVALID);
        }
        ThreadMetadata::from((self.handle, status))
    }

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


