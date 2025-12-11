use core::any::Any;
use core::ffi::c_void;
use core::ops::Deref;
use core::ptr::null_mut;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use crate::freertos::ffi::{INVALID, TaskStatus, ThreadHandle, pdPASS, pdTRUE, vTaskDelete, vTaskGetInfo, vTaskResume, vTaskSuspend, xTaskCreate, xTaskGetCurrentTaskHandle};
use crate::freertos::types::{StackType, UBaseType, BaseType, DoublePtr};
use crate::freertos::thread::ThreadState::*;
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
            stack_depth: unsafe {*status.1.pxStackBase},
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


#[derive(Clone)]
pub struct Thread {
    handle: ThreadHandle,
    name: String,
    stack_depth: StackType,
    priority: UBaseType,
    callback: Option<Arc<ThreadFnPtr>>,
    param: ThreadParam
}

unsafe extern "C" fn callback(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    let boxed_thread: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut _) };

    let param_arc: Arc<dyn Any + Send + Sync> = boxed_thread
        .param
        .clone()
        .unwrap_or_else(|| Arc::new(()) as Arc<dyn Any + Send + Sync>);

    if let Some(callback) = &boxed_thread.callback {
        let _ = callback(Some(param_arc));
    }
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

    fn spawn(&mut self, param: ThreadParam) -> Result<Self> {        

        let name = to_cstring!(self.name)?;

        let mut handle: ThreadHandle =  null_mut();

        let boxed_thread = Box::new(self.clone());

        let ret = unsafe {
            xTaskCreate(
                Some(super::thread::callback),
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

    fn get_metadata(handle: ThreadHandle) -> ThreadMetadata {
        let mut status = TaskStatus::default();
        unsafe {
            vTaskGetInfo(handle, &mut status, pdTRUE, INVALID);
        }
        ThreadMetadata::from((handle, status))
    }

    fn get_current() -> Self {
        let handle = unsafe { xTaskGetCurrentTaskHandle() };
        let metadata = Self::get_metadata(handle);
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

    fn wait_notification(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32 , timeout_ticks: impl ToTick) -> Result<u32> {
        if self.handle.is_null() {
            return Err(Error::NullPtr);
        }

        let mut notification_value: u32 = 0;

        let ret = xTaskNotifyWait!(
            bits_to_clear_on_entry,
            bits_to_clear_on_exit,
            &mut notification_value,
            timeout_ticks.to_tick()
        );
        

        if ret == pdTRUE {
            Ok(notification_value)
        } else {
            Err(Error::Timeout)
        }
    }

}


impl Drop for Thread {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { vTaskDelete( self.handle ); } 
        }
    }
}

impl Deref for Thread {
    type Target = ThreadHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
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
