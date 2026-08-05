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

use core::ffi::c_void;
use core::fmt::{Debug, Display};
use core::mem::forget;
use core::ops::Deref;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};

use crate::freertos::ffi::pdPASS;
use crate::traits::{MAX_TASK_NAME_LEN, ToTick, TimerParam, TimerFn, TimerFnPtr};
use crate::utils::{Bytes, Error, OsalRsBool, Result};
use super::ffi::{TimerHandle, pvTimerGetTimerID, vTimerSetTimerID, xTimerCreate, osal_rs_timer_start, osal_rs_timer_change_period, osal_rs_timer_delete, osal_rs_timer_reset, osal_rs_timer_stop};
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
    /// FreeRTOS timer handle, exposed for diagnostics (`Debug`/`Display`).
    /// Reset to `null` by [`TimerFn::delete`]; the handle the operations
    /// below actually use lives in [`TimerShared`], so that clones see a
    /// deletion performed through any other handle.
    pub handle: TimerHandle,
    /// Timer name, in the same fixed-size buffer every other named object in
    /// this crate uses. `Bytes` is `Copy`, so handing a named handle to the
    /// callback on every firing costs nothing, and keeping a copy here rather
    /// than only in [`TimerShared`] means `Debug`/`Display` still work after
    /// [`TimerFn::delete`] has dropped the shared state.
    name: Bytes<MAX_TASK_NAME_LEN>,
    /// The one underlying timer, shared by every clone of this handle.
    /// `None` once this particular handle has been deleted.
    shared: Option<Arc<TimerShared>>,
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

/// State shared, via [`Arc`], between every clone of a given [`Timer`].
///
/// [`Timer`] itself is freely `Clone` (matching every other handle type in
/// this crate), but the FreeRTOS timer, its name buffer, its callback and its
/// rolling parameter all belong to one underlying resource - this is that
/// resource. Mirrors `posix::TimerShared`.
struct TimerShared {
    /// The FreeRTOS timer handle; `null` once the timer has been destroyed.
    handle: AtomicPtr<c_void>,
    /// Set between a successful `xTimerCreate` and destruction. Guards every
    /// operation, and lets destruction claim the timer exactly once however
    /// many clones race for it.
    ready: AtomicBool,
    /// The buffer whose address is handed to `xTimerCreate`, which *stores
    /// the pointer* instead of copying the string - so it has to live at a
    /// stable address for as long as the timer does, which is exactly the
    /// lifetime of this block. `Bytes::from_str` zero-fills, so the name is
    /// NUL-terminated unless it fills the buffer exactly, in which case it is
    /// truncated to fit like every other name in this crate.
    name: Bytes<MAX_TASK_NAME_LEN>,
    /// Callback to run when the timer expires.
    callback: Option<Arc<TimerFnPtr>>,
    /// Rolling callback parameter: a `*mut TimerParam` from `Box::into_raw`,
    /// or null for `None`. Each firing hands the callback a clone and stores
    /// whatever it returns for the next one, mirroring the `param` local in
    /// `posix::run_timer_thread`. Only the timer daemon task - which runs
    /// callbacks one at a time - and teardown ever touch it, so an atomic
    /// swap is enough and no lock is needed.
    param: AtomicPtr<c_void>,
}

impl TimerShared {
    /// Takes the current callback parameter out of the shared slot, leaving
    /// it empty.
    fn take_param(&self) -> Option<TimerParam> {
        let raw = self.param.swap(null_mut(), Ordering::AcqRel);

        if raw.is_null() {
            None
        } else {
            Some(*unsafe { Box::from_raw(raw as *mut TimerParam) })
        }
    }

    /// Puts `param` into the shared slot, releasing whatever was there.
    fn store_param(&self, param: Option<TimerParam>) {
        let raw = match param {
            Some(param) => Box::into_raw(Box::new(param)) as *mut c_void,
            None => null_mut(),
        };

        let previous = self.param.swap(raw, Ordering::AcqRel);

        if !previous.is_null() {
            drop(unsafe { Box::from_raw(previous as *mut TimerParam) });
        }
    }

    /// Destroys the underlying FreeRTOS timer, at most once however many
    /// handles ask for it. Returns whether this call is the one that did it.
    fn destroy(&self, ticks_to_wait: TickType) -> bool {
        if !self.ready.swap(false, Ordering::AcqRel) {
            return false;
        }

        let handle = self.handle.swap(null_mut(), Ordering::AcqRel) as TimerHandle;

        // Detach the timer from this block *before* deleting it: deletion
        // only queues a command to the timer daemon, so a firing that was
        // already due can still be dispatched afterwards, and a null ID
        // turns that dispatch into a no-op instead of a visit to a block
        // that is on its way out.
        unsafe {
            vTimerSetTimerID(handle, null_mut());
            osal_rs_timer_delete(handle, ticks_to_wait);
        }

        self.store_param(None);

        true
    }
}

/// Destroys the underlying timer once the last [`Timer`] handle referring to
/// it is gone - the RAII half of [`TimerFn::delete`], and the reason `Timer`
/// itself has no `Drop` of its own (a per-handle `Drop` would tear the timer
/// down as soon as *any* clone was dropped).
impl Drop for TimerShared {
    fn drop(&mut self) {
        // Reaching here means every handle is gone, so nothing but the timer
        // daemon can still reach this block. If the timer is still alive,
        // hand it the last rites rather than deleting it here: re-arming it
        // for a single tick (`xTimerChangePeriod` starts a dormant timer as
        // well as re-arming a running one) makes it dispatch one final time,
        // and `callback_c_wrapper` will find this block dead, delete the
        // timer, and release the `Weak` that keeps this allocation alive.
        // Doing it from the daemon - the same task that processes deletions
        // and dispatches callbacks, one at a time - is what makes releasing
        // that `Weak` free of any race with a dispatch already in flight.
        if self.ready.swap(false, Ordering::AcqRel) {
            let handle = self.handle.swap(null_mut(), Ordering::AcqRel) as TimerHandle;

            if unsafe { osal_rs_timer_change_period(handle, 1, 0) } != pdPASS {
                // Timer command queue full: there is no way to get a final
                // dispatch, so delete from here and accept that the `Weak`
                // is never released (see `Timer::new`).
                unsafe {
                    vTimerSetTimerID(handle, null_mut());
                    osal_rs_timer_delete(handle, 0);
                }
            }
        }

        self.store_param(None);
    }
}

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
/// This function bridges between FreeRTOS C API and Rust closures. It runs on
/// the timer daemon task, recovers the [`TimerShared`] block from the timer
/// ID, and invokes the user callback with a fresh [`Timer`] handle and the
/// current parameter.
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

    let id = unsafe { pvTimerGetTimerID(handle) } as *const TimerShared;

    if id.is_null() {
        // `TimerShared::destroy` has detached the timer: this dispatch was
        // already due when the deletion was queued. Nothing left to look at.
        return;
    }

    // The `Weak` belongs to the FreeRTOS timer object, not to this call, so
    // it is reconstructed only to be inspected and is then handed straight
    // back with `forget`. Consuming it is reserved for the one branch below
    // that also destroys the timer object owning it.
    let weak = unsafe { Weak::from_raw(id) };

    let Some(shared) = weak.upgrade() else {
        // Every `Timer` handle is gone and `TimerShared::drop` deliberately
        // left the timer alive so that this final dispatch could finish the
        // job from the daemon task. See that `Drop` impl for why here is the
        // only place this is race-free.
        unsafe {
            vTimerSetTimerID(handle, null_mut());
            osal_rs_timer_delete(handle, 0);
        }
        drop(weak);
        return;
    };

    forget(weak);

    // A `delete()` through some other handle got in first: the timer is on
    // its way out, so do not run the user callback against a half torn down
    // state. Mirrors `posix::run_timer_thread` re-checking `exit` after
    // `sigwait` returns.
    if !shared.ready.load(Ordering::Acquire) {
        return;
    }

    let Some(callback) = shared.callback.clone() else {
        return;
    };

    // The callback is handed a *clone* of the handle, never anything that
    // owns the timer: `TimerFnPtr` takes its `Box<dyn TimerFn>` by value and
    // drops it on return, so passing an owning handle would destroy the
    // timer at its own first firing.
    let timer_self = Timer {
        handle,
        name: shared.name,
        shared: Some(shared.clone()),
    };

    // Thread the parameter through the callback exactly the way POSIX does:
    // what one firing returns is what the next one receives, and a failed
    // firing leaves the previous value in place.
    let current = shared.take_param();

    match callback(Box::new(timer_self), current.clone()) {
        Ok(next) => shared.store_param(Some(next)),
        Err(_) => shared.store_param(current),
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
    /// ```
    pub fn new<F>(name: &str, timer_period_in_ticks: TickType, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static {

            // `xTimerCreate` keeps the name pointer it is given rather than
            // copying the string, so the name has to be NUL-terminated (a
            // `&str` is not) and live at an address that outlives the timer.
            let name = Bytes::<MAX_TASK_NAME_LEN>::from_str(name);

            let shared = Arc::new(TimerShared {
                handle: AtomicPtr::new(null_mut()),
                ready: AtomicBool::new(false),
                name,
                callback: Some(Arc::new(callback)),
                param: AtomicPtr::new(null_mut()),
            });

            shared.store_param(param);

            // The timer object gets a *weak* reference. A strong one would
            // keep `TimerShared` alive for as long as the timer exists -
            // and since the timer is only destroyed when `TimerShared` is
            // dropped, that cycle would mean neither ever goes away.
            //
            // Ownership of the `Weak` passes to the timer object, and it is
            // released by `callback_c_wrapper` on the final dispatch that
            // `TimerShared::drop` arranges. The one path that cannot get
            // that dispatch - a timer command queue too full to re-arm - is
            // the one case where this allocation is leaked; on a target that
            // creates its timers once at start-up it never arises.
            let id = Weak::into_raw(Arc::downgrade(&shared));

            let handle = unsafe {
                xTimerCreate( shared.name.as_cstr().as_ptr(),
                    timer_period_in_ticks,
                    if auto_reload { 1 } else { 0 },
                    id as *mut c_void,
                    Some(super::timer::callback_c_wrapper)
                )
            };

            if handle.is_null() {
                drop(unsafe { Weak::from_raw(id) });
                return Err(Error::NullPtr);
            }

            shared.handle.store(handle as *mut c_void, Ordering::Release);
            shared.ready.store(true, Ordering::Release);

            Ok(Self { handle, name, shared: Some(shared) })
    }

    /// Returns the live FreeRTOS handle, or `None` if this timer has been
    /// deleted through this handle or any clone of it - the guard every
    /// operation in `impl TimerFn` starts with.
    fn live_handle(&self) -> Option<TimerHandle> {
        let shared = self.shared.as_ref()?;

        if !shared.ready.load(Ordering::Acquire) {
            return None;
        }

        Some(shared.handle.load(Ordering::Acquire) as TimerHandle)
    }

}

impl TimerFn for Timer {

    /// Returns `true` if the timer has already been deleted - through this
    /// handle or through any clone of it, since clones share one underlying
    /// timer.
    fn is_null(&self) -> bool {
        match &self.shared {
            Some(shared) => !shared.ready.load(Ordering::Acquire),
            None => true,
        }
    }

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
        let Some(handle) = self.live_handle() else {
            return OsalRsBool::False;
        };

        if unsafe {
            osal_rs_timer_start(handle, ticks_to_wait)
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
        let Some(handle) = self.live_handle() else {
            return OsalRsBool::False;
        };

        if unsafe {
            osal_rs_timer_stop(handle, ticks_to_wait)
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
        let Some(handle) = self.live_handle() else {
            return OsalRsBool::False;
        };

        if unsafe {
            osal_rs_timer_reset(handle, ticks_to_wait)
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
        let Some(handle) = self.live_handle() else {
            return OsalRsBool::False;
        };

        if unsafe {
            osal_rs_timer_change_period(handle, new_period_in_ticks, new_period_ticks)
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
        // Giving up the shared state is what makes this handle null; the
        // timer itself is destroyed by whichever handle gets there first,
        // and every clone sees it through the `ready` flag. Matches
        // `posix::Timer::delete`, down to reporting `False` only for a
        // second delete through this same handle.
        let Some(shared) = self.shared.take() else {
            return OsalRsBool::False;
        };

        shared.destroy(ticks_to_wait);

        self.handle = null_mut();

        OsalRsBool::True
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
            .field("is_null", &self.is_null())
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