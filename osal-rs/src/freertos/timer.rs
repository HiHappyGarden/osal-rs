use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use crate::traits::{ToTick, TimerParam, TimerFn, TimerFnPtr};
use crate::utils::{OsalRsBool, Result};
use super::ffi::TimerHandle;
use super::types::{TickType};


pub struct Timer {
    pub handle: TimerHandle,
    name: String, 
    timer_period_in_ticks: TickType, 
    auto_reload: bool, 
    callback: Option<Arc<TimerFnPtr>>,
    param: Option<TimerParam>, 
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl TimerFn for Timer {
    fn new<F>(name: &str, timer_period_in_ticks: impl ToTick, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + 'static {
        Ok(Self {
            handle: core::ptr::null_mut(),
            name: name.to_string(),
            timer_period_in_ticks: timer_period_in_ticks.to_ticks(),
            auto_reload,
            callback: Some(Arc::new(callback)),
            param,
        })
    }

    fn start(&self, ticks_to_wait: TickType) -> OsalRsBool {
        todo!()
    }

    fn stop(&self, ticks_to_wait: TickType)  -> OsalRsBool {
        todo!()
    }

    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool {
        todo!()
    }

    fn change_period(&self, new_period_in_ticks: TickType, new_period_ticks: TickType) -> OsalRsBool {
        todo!()
    }

    fn delete(&mut self) -> OsalRsBool {
        todo!()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Add any necessary cleanup code here
    }
}


