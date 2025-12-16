use core::any::Any;

use alloc::{boxed::Box, sync::Arc};

use crate::os::types::TickType;
use crate::utils::{OsalRsBool, Result};


pub type TimerParam = Arc<dyn Any + Send + Sync>;
pub type TimerFnPtr = dyn Fn(Box<dyn Timer>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + 'static;

pub trait Timer {
    fn new<F>(name: &str, timer_period_in_ticks: TickType, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        Self: Sized,
        F: Fn(Box<dyn Timer>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static;

    fn start(&self, ticks_to_wait: TickType) -> OsalRsBool;
    fn stop(&self, ticks_to_wait: TickType)  -> OsalRsBool;
    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool;
    fn change_period(&self, new_period_in_ticks: TickType, new_period_ticks: TickType) -> OsalRsBool;
    fn delete(&mut self, ticks_to_wait: TickType) -> OsalRsBool;
}