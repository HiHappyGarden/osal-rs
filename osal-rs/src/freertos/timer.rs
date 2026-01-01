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

use core::any::Any;
use core::fmt::{Debug, Display};
use core::ops::Deref;
use core::ptr::null_mut;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use crate::freertos::ffi::pdPASS;
use crate::to_c_str;
use crate::traits::{ToTick, TimerParam, TimerFn, TimerFnPtr};
use crate::utils::{OsalRsBool, Result, Error};
use super::ffi::{TimerHandle, pvTimerGetTimerID, xTimerCreate, osal_rs_timer_start, osal_rs_timer_change_period, osal_rs_timer_delete, osal_rs_timer_reset, osal_rs_timer_stop};
use super::types::{TickType};

#[derive(Clone)]
pub struct Timer {
    pub handle: TimerHandle,
    name: String, 
    callback: Option<Arc<TimerFnPtr>>,
    param: Option<TimerParam>, 
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl Timer {
    #[inline]
    pub fn new_with_to_tick<F>(name: &str, timer_period_in_ticks: impl ToTick, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static {
            Self::new(name, timer_period_in_ticks.to_ticks(), auto_reload, param, callback)
        }

    #[inline]
    pub fn start_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.start(ticks_to_wait.to_ticks())
    }

    #[inline]
    pub fn stop_with_to_tick(&self, ticks_to_wait: impl ToTick)  -> OsalRsBool {
        self.stop(ticks_to_wait.to_ticks())
    }

    #[inline]
    pub fn reset_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.reset(ticks_to_wait.to_ticks())
    }

    #[inline]
    pub fn change_period_with_to_tick(&self, new_period_in_ticks: impl ToTick, new_period_ticks: impl ToTick) -> OsalRsBool {
        self.change_period(new_period_in_ticks.to_ticks(), new_period_ticks.to_ticks())
    }

    #[inline]
    pub fn delete_with_to_tick(&mut self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.delete(ticks_to_wait.to_ticks())
    }
}

extern "C" fn callback_c_wrapper(handle: TimerHandle) {

    if handle.is_null() {
        return;
    }

    let param_ptr = unsafe {
        pvTimerGetTimerID(handle) 
    };
    
    let mut timer_instance: Box<Timer> = unsafe { Box::from_raw(param_ptr as *mut _) };

    timer_instance.as_mut().handle = handle;

    let param_arc: Option<Arc<dyn Any + Send + Sync>> = timer_instance
        .param
        .clone();

    if let Some(callback) = &timer_instance.callback.clone() {
        let _ = callback(timer_instance, param_arc);
    }
}

impl TimerFn for Timer {
    fn new<F>(name: &str, timer_period_in_ticks: TickType, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static {

            let mut boxed_timer = Box::new(Self {
                handle: core::ptr::null_mut(),
                name: name.to_string(),
                callback: Some(Arc::new(callback.clone())),
                param: param.clone(),
            });

            let handle = unsafe {
                xTimerCreate( to_c_str!(name), 
                    timer_period_in_ticks, 
                    if auto_reload { 1 } else { 0 }, 
                    Box::into_raw(boxed_timer.clone()) as *mut _, 
                    Some(super::timer::callback_c_wrapper)
                )
            };

            if handle.is_null() {
                Err(Error::NullPtr)
            } else {
                boxed_timer.as_mut().handle = handle;
                Ok(*boxed_timer)
            }

    }

    fn start(&self, ticks_to_wait: TickType) -> OsalRsBool {
        if unsafe {
            osal_rs_timer_start(self.handle, ticks_to_wait)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    fn stop(&self, ticks_to_wait: TickType)  -> OsalRsBool {
        if unsafe {
            osal_rs_timer_stop(self.handle, ticks_to_wait)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool {
        if unsafe {
            osal_rs_timer_reset(self.handle, ticks_to_wait)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    fn change_period(&self, new_period_in_ticks: TickType, new_period_ticks: TickType) -> OsalRsBool {
        if unsafe {
            osal_rs_timer_change_period(self.handle, new_period_in_ticks, new_period_ticks)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    fn delete(&mut self, ticks_to_wait: TickType) -> OsalRsBool {
        if unsafe {
            osal_rs_timer_delete(self.handle, ticks_to_wait)
        } != pdPASS {
            self.handle = null_mut();
            OsalRsBool::False
        } else {
            self.handle = null_mut();
            OsalRsBool::True
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.delete(0);
    }
}

impl Deref for Timer {
    type Target = TimerHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Debug for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Timer")
            .field("handle", &self.handle)
            .field("name", &self.name)
            .finish()
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Timer {{ name: {}, handle: {:?} }}", self.name, self.handle)
    }
}