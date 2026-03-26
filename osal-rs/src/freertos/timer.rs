/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Software timer support for FreeRTOS.
//!
//! This module provides software timers that run callbacks at specified intervals.
//! Timers can be one-shot or auto-reloading (periodic) and execute their callbacks
//! in the timer daemon task context.

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

/// A software timer that executes a callback at regular intervals.
///
/// Timers can be configured as:
/// - **One-shot**: Executes once after the specified period
/// - **Auto-reload**: Executes repeatedly at the specified interval
///
/// Timer callbacks execute in the context of the timer daemon task, not in
/// interrupt context. This means they can call most RTOS functions safely.
///
/// # Important Notes
///
/// - Timer callbacks should complete quickly to avoid delaying other timers
/// - Callbacks must not block indefinitely
/// - Requires `configUSE_TIMERS = 1` in FreeRTOSConfig.h
///
/// # Examples
///
/// ## One-shot timer
///
/// ```ignore
/// use osal_rs::os::{Timer, TimerFn};
/// use core::time::Duration;
/// 
/// let timer = Timer::new_with_to_tick(
///     "oneshot",
///     Duration::from_secs(1),
///     false,  // Not auto-reload (one-shot)
///     None,
///     |timer, param| {
///         println!("Timer fired once!");
///         Ok(param)
///     }
/// ).unwrap();
/// 
/// timer.start_with_to_tick(Duration::from_millis(10)).unwrap();
/// ```
///
/// ## Periodic timer
///
/// ```ignore
/// use osal_rs::os::{Timer, TimerFn};
/// use core::time::Duration;
/// 
/// let timer = Timer::new_with_to_tick(
///     "periodic",
///     Duration::from_millis(500),
///     true,  // Auto-reload (periodic)
///     None,
///     |timer, param| {
///         println!("Tick every 500ms");
///         Ok(param)
///     }
/// ).unwrap();
/// 
/// timer.start_with_to_tick(Duration::from_millis(10)).unwrap();
/// 
/// // Stop after some time
/// Duration::from_secs(5).sleep();
/// timer.stop_with_to_tick(Duration::from_millis(10));
/// ```
///
/// ## Timer with custom parameters
///
/// ```ignore
/// use osal_rs::os::{Timer, TimerFn, TimerParam};
/// use alloc::sync::Arc;
/// use core::time::Duration;
/// 
/// struct CounterData {
///     count: u32,
/// }
/// 
/// let data = Arc::new(CounterData { count: 0 });
/// let param: TimerParam = data.clone();
/// 
/// let timer = Timer::new_with_to_tick(
///     "counter",
///     Duration::from_secs(1),
///     true,
///     Some(param),
///     |timer, param| {
///         if let Some(param_arc) = param {
///             if let Some(data) = param_arc.downcast_ref::<CounterData>() {
///                 println!("Counter: {}", data.count);
///             }
///         }
///         Ok(None)
///     }
/// ).unwrap();
/// 
/// timer.start_with_to_tick(Duration::from_millis(10));
/// ```
///
/// ## Changing timer period
///
/// ```ignore
/// use osal_rs::os::{Timer, TimerFn};
/// use core::time::Duration;
/// 
/// let timer = Timer::new_with_to_tick(
///     "adjustable",
///     Duration::from_millis(100),
///     true,
///     None,
///     |_, _| { println!("Tick"); Ok(None) }
/// ).unwrap();
/// 
/// timer.start_with_to_tick(Duration::from_millis(10));
/// 
/// // Change period to 500ms
/// Duration::from_secs(2).sleep();
/// timer.change_period_with_to_tick(
///     Duration::from_millis(500),
///     Duration::from_millis(10)
/// );
/// ```
///
/// ## Resetting a timer
///
/// ```ignore
/// use osal_rs::os::{Timer, TimerFn};
/// use core::time::Duration;
/// 
/// let timer = Timer::new_with_to_tick(
///     "watchdog",
///     Duration::from_secs(5),
///     false,
///     None,
///     |_, _| { println!("Timeout!"); Ok(None) }
/// ).unwrap();
/// 
/// timer.start_with_to_tick(Duration::from_millis(10));
/// 
/// // Reset timer before it expires (like a watchdog)
/// Duration::from_secs(2).sleep();
/// timer.reset_with_to_tick(Duration::from_millis(10));  // Restart the 5s countdown
/// ```
#[derive(Clone)]
pub struct Timer {
    /// FreeRTOS timer handle
    pub handle: TimerHandle,
    /// Timer name for debugging
    name: String, 
    /// Callback function to execute when timer expires
    callback: Option<Arc<TimerFnPtr>>,
    /// Optional parameter passed to callback
    param: Option<TimerParam>, 
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl Timer {
    /// Creates a new software timer with tick conversion.
    /// 
    /// This is a convenience method that accepts any type implementing `ToTick`
    /// (like `Duration`) for the timer period.
    /// 
    /// # Parameters
    /// 
    /// * `name` - Timer name for debugging
    /// * `timer_period_in_ticks` - Timer period (e.g., `Duration::from_secs(1)`)
    /// * `auto_reload` - `true` for periodic, `false` for one-shot
    /// * `param` - Optional parameter passed to callback
    /// * `callback` - Function called when timer expires
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - Successfully created timer
    /// * `Err(Error)` - Creation failed
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// use core::time::Duration;
    /// 
    /// let timer = Timer::new_with_to_tick(
    ///     "periodic",
    ///     Duration::from_secs(1),
    ///     true,
    ///     None,
    ///     |_timer, _param| { println!("Tick"); Ok(None) }
    /// ).unwrap();
    /// ```
    #[inline]
    pub fn new_with_to_tick<F>(name: &str, timer_period_in_ticks: impl ToTick, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static {
            Self::new(name, timer_period_in_ticks.to_ticks(), auto_reload, param, callback)
        }

    /// Starts the timer with tick conversion.
    /// 
    /// Convenience method that accepts any type implementing `ToTick`.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for the command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer started successfully
    /// * `OsalRsBool::False` - Failed to start timer
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// use core::time::Duration;
    /// 
    /// timer.start_with_to_tick(Duration::from_millis(10));
    /// ```
    #[inline]
    pub fn start_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.start(ticks_to_wait.to_ticks())
    }

    /// Stops the timer with tick conversion.
    /// 
    /// Convenience method that accepts any type implementing `ToTick`.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for the command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer stopped successfully
    /// * `OsalRsBool::False` - Failed to stop timer
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// use core::time::Duration;
    /// 
    /// timer.stop_with_to_tick(Duration::from_millis(10));
    /// ```
    #[inline]
    pub fn stop_with_to_tick(&self, ticks_to_wait: impl ToTick)  -> OsalRsBool {
        self.stop(ticks_to_wait.to_ticks())
    }

    /// Resets the timer with tick conversion.
    /// 
    /// Resets the timer to restart its period. For one-shot timers, this
    /// restarts them. For periodic timers, this resets the period.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for the command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer reset successfully
    /// * `OsalRsBool::False` - Failed to reset timer
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// use core::time::Duration;
    /// 
    /// // Reset watchdog timer before it expires
    /// timer.reset_with_to_tick(Duration::from_millis(10));
    /// ```
    #[inline]
    pub fn reset_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.reset(ticks_to_wait.to_ticks())
    }

    /// Changes the timer period with tick conversion.
    /// 
    /// Convenience method that accepts any type implementing `ToTick`.
    /// 
    /// # Parameters
    /// 
    /// * `new_period_in_ticks` - New timer period
    /// * `new_period_ticks` - Maximum time to wait for the command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Period changed successfully
    /// * `OsalRsBool::False` - Failed to change period
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// use core::time::Duration;
    /// 
    /// // Change from 1 second to 500ms
    /// timer.change_period_with_to_tick(
    ///     Duration::from_millis(500),
    ///     Duration::from_millis(10)
    /// );
    /// ```
    #[inline]
    pub fn change_period_with_to_tick(&self, new_period_in_ticks: impl ToTick, new_period_ticks: impl ToTick) -> OsalRsBool {
        self.change_period(new_period_in_ticks.to_ticks(), new_period_ticks.to_ticks())
    }

    /// Deletes the timer with tick conversion.
    /// 
    /// Convenience method that accepts any type implementing `ToTick`.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for the command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer deleted successfully
    /// * `OsalRsBool::False` - Failed to delete timer
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// use core::time::Duration;
    /// 
    /// timer.delete_with_to_tick(Duration::from_millis(10));
    /// ```
    #[inline]
    pub fn delete_with_to_tick(&mut self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.delete(ticks_to_wait.to_ticks())
    }
}

/// Internal C-compatible wrapper for timer callbacks.
/// 
/// This function bridges between FreeRTOS C API and Rust closures.
/// It retrieves the timer instance from the timer ID, extracts the callback
/// and parameters, and executes the user-provided callback.
/// 
/// # Safety
/// 
/// This function is marked extern "C" because it:
/// - Is called from FreeRTOS C code (timer daemon task)
/// - Performs raw pointer conversions
/// - Expects a valid timer handle with associated timer instance
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



impl Timer {
    /// Creates a new software timer.
    ///
    /// # Parameters
    ///
    /// * `name` - Timer name for debugging
    /// * `timer_period_in_ticks` - Timer period in ticks
    /// * `auto_reload` - `true` for periodic, `false` for one-shot
    /// * `param` - Optional parameter passed to callback
    /// * `callback` - Function called when timer expires
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created timer
    /// * `Err(Error)` - Creation failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// 
    /// let timer = Timer::new(
    ///     "my_timer",
    ///     1000,
    ///     false,
    ///     None,
    ///     |_timer, _param| Ok(None)
    /// ).unwrap();
    /// ``
    
    pub fn new<F>(name: &str, timer_period_in_ticks: TickType, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
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
    
}

impl TimerFn for Timer {

    /// Starts the timer.
    /// 
    /// Sends a command to the timer daemon to start the timer. If the timer
    /// was already running, this has no effect.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer started successfully
    /// * `OsalRsBool::False` - Failed to start (command queue full)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// 
    /// let timer = Timer::new("my_timer", 1000, true, None, |_, _| Ok(None)).unwrap();
    /// timer.start(10);  // Wait up to 10 ticks
    /// ```
    fn start(&self, ticks_to_wait: TickType) -> OsalRsBool {
        if unsafe {
            osal_rs_timer_start(self.handle, ticks_to_wait)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    /// Stops the timer.
    /// 
    /// Sends a command to the timer daemon to stop the timer. The timer will not
    /// fire again until it is restarted.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer stopped successfully
    /// * `OsalRsBool::False` - Failed to stop (command queue full)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// 
    /// timer.stop(10);  // Wait up to 10 ticks to stop
    /// ```
    fn stop(&self, ticks_to_wait: TickType)  -> OsalRsBool {
        if unsafe {
            osal_rs_timer_stop(self.handle, ticks_to_wait)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    /// Resets the timer.
    /// 
    /// Resets the timer's period. For a one-shot timer that has already expired,
    /// this will restart it. For a periodic timer, this resets the period.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer reset successfully
    /// * `OsalRsBool::False` - Failed to reset (command queue full)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// 
    /// // Reset a watchdog timer before it expires
    /// timer.reset(10);
    /// ```
    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool {
        if unsafe {
            osal_rs_timer_reset(self.handle, ticks_to_wait)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    /// Changes the timer period.
    /// 
    /// Changes the period of a timer that was previously created. The timer
    /// must be stopped, or the period will be changed when it next expires.
    /// 
    /// # Parameters
    /// 
    /// * `new_period_in_ticks` - New period for the timer in ticks
    /// * `new_period_ticks` - Maximum time to wait for command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Period changed successfully
    /// * `OsalRsBool::False` - Failed to change period (command queue full)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// 
    /// // Change period from 1000 ticks to 500 ticks
    /// timer.change_period(500, 10);
    /// ```
    fn change_period(&self, new_period_in_ticks: TickType, new_period_ticks: TickType) -> OsalRsBool {
        if unsafe {
            osal_rs_timer_change_period(self.handle, new_period_in_ticks, new_period_ticks)
        } != pdPASS {
            OsalRsBool::False
        } else {
            OsalRsBool::True
        }
    }

    /// Deletes the timer.
    /// 
    /// Sends a command to the timer daemon to delete the timer.
    /// The timer handle becomes invalid after this call.
    /// 
    /// # Parameters
    /// 
    /// * `ticks_to_wait` - Maximum time to wait for command to be sent to timer daemon
    /// 
    /// # Returns
    /// 
    /// * `OsalRsBool::True` - Timer deleted successfully
    /// * `OsalRsBool::False` - Failed to delete (command queue full)
    /// 
    /// # Safety
    /// 
    /// After calling this function, the timer handle is set to null and should not be used.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use osal_rs::os::{Timer, TimerFn};
    /// 
    /// let mut timer = Timer::new("temp", 1000, false, None, |_, _| Ok(None)).unwrap();
    /// timer.delete(10);
    /// ```
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

/// Automatically deletes the timer when it goes out of scope.
/// 
/// This ensures proper cleanup of FreeRTOS resources by calling
/// `delete(0)` when the timer is dropped.
impl Drop for Timer {
    fn drop(&mut self) {
        self.delete(0);
    }
}

/// Allows dereferencing to the underlying FreeRTOS timer handle.
impl Deref for Timer {
    type Target = TimerHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// Formats the timer for debugging purposes.
/// 
/// Shows the timer handle and name.
impl Debug for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Timer")
            .field("handle", &self.handle)
            .field("name", &self.name)
            .finish()
    }
}

/// Formats the timer for display purposes.
/// 
/// Shows a concise representation with name and handle.
impl Display for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Timer {{ name: {}, handle: {:?} }}", self.name, self.handle)
    }
}