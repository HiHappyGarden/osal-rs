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

//! Software timer support for POSIX.
//!
//! pthreads has no notion of a shared "timer daemon task" the way FreeRTOS
//! does, so each [`Timer`] gets its own dedicated background thread, which
//! sleeps on a condition variable until the timer's next deadline:
//!
//! 1. The deadline lives in the timer's shared state (mutex + condvar +
//!    next expiry on the monotonic clock). `start`/`reset`/`change_period`
//!    set it, `stop` clears it, and every change wakes the thread so it
//!    re-evaluates how long to sleep.
//! 2. The background thread waits with
//!    [`cond_timedwait_monotonic`](crate::posix::ffi), so the countdown is
//!    unaffected by wall-clock changes on both Linux and macOS. When the
//!    deadline passes it re-arms (auto-reload) or disarms (one-shot) the
//!    timer, releases the lock and invokes the user callback.
//! 3. Auto-reload deadlines advance from the previous deadline rather than
//!    from "now", so the period does not drift by the callback's run time.
//!    If the callback overruns a whole period the missed expirations are
//!    coalesced into one, as with a kernel `timer_settime` interval.
//!
//! The same implementation runs on every supported platform: it relies only
//! on pthread mutexes/condvars, unlike a `timer_create(2)`/`SIGEV_THREAD_ID`
//! design, which macOS does not provide. No signal is involved, so creating
//! a `Timer` leaves the calling thread's signal mask untouched, and a
//! one-shot timer can be started again after it fired.
//!
//! # Examples
//!
//! ```
//! use osal_rs::os::*;
//! use std::sync::Arc;
//! use std::sync::atomic::{AtomicBool, Ordering};
//! use core::time::Duration;
//!
//! static FIRED: AtomicBool = AtomicBool::new(false);
//!
//! let timer = Timer::new_with_to_tick(
//!     "heartbeat",
//!     Duration::from_millis(10),
//!     false, // one-shot
//!     None,
//!     |_timer, _param| {
//!         FIRED.store(true, Ordering::SeqCst);
//!         Ok(Arc::new(()))
//!     }
//! ).unwrap();
//!
//! timer.start(0);
//! System::delay(50);
//! assert!(FIRED.load(Ordering::SeqCst));
//! ```

use core::ffi::c_long;
use core::fmt::{Debug, Display};
use core::ops::Deref;
use core::ptr::null_mut;
use core::time::Duration;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::os::ThreadFn;
use crate::posix::config::TICK_PERIOD_MS;
use crate::posix::ffi::{CLOCK_MONOTONIC, clock_gettime, monotonic_deadline, pthread_self, timespec};
use crate::posix::mutex::Mutex;
use crate::posix::thread::{RawCondvar, Thread};
use crate::posix::types::{StackType, TickType, TimerHandle, UBaseType};
use crate::traits::{MAX_TASK_NAME_LEN, MutexFn, TimerFn, TimerFnPtr, TimerParam, ToTick};
use crate::utils::{Bytes, OsalRsBool, Result};

/// Name (`<= 15` chars, the Linux limit) given to every timer's background
/// thread. Fixed rather than derived from the timer's own name so it's
/// always valid regardless of what the caller passed to `Timer::new`.
const TIMER_THREAD_NAME: &str = "os_timer";

/// Stack size requested for a timer's background thread. `Thread::spawn`
/// enforces its own safe minimum regardless, so this only matters as a
/// lower bound.
const TIMER_THREAD_STACK: StackType = 1024;

const NSECS_PER_SEC: c_long = 1_000_000_000;

/// Priority given to a timer's background thread: the lowest valid
/// `SCHED_FIFO` priority with the `real_time` feature (1 on Linux, 15 on
/// macOS), otherwise a placeholder, since the thread then inherits the
/// creating thread's scheduling policy/priority.
fn timer_thread_priority() -> UBaseType {
    #[cfg(feature = "real_time")]
    {
        use crate::posix::ffi::{SCHED_FIFO, sched_get_priority_min};

        (unsafe { sched_get_priority_min(SCHED_FIFO) }).max(0) as UBaseType
    }

    #[cfg(not(feature = "real_time"))]
    {
        1
    }
}

/// Mutable timer state, guarded by [`TimerShared::state`].
#[derive(Default)]
struct TimerState {
    /// Next expiry on the monotonic clock; `None` while the timer is stopped.
    deadline: Option<timespec>,
    /// Set by teardown to make the background thread return.
    exit: bool,
}

/// State shared, via `Arc`, between every clone of a given [`Timer`], the
/// handles given to its callback, and its background thread.
///
/// [`Timer`] itself is freely `Clone` (matching every other handle type in
/// this crate), but the background thread, its deadline, and the mutable
/// period all belong to one underlying resource — this is that resource.
/// Holding it does not keep the timer running; [`TimerOwner`] does.
struct TimerShared {
    /// Deadline and exit flag; every change is announced on `cv`.
    state: Mutex<TimerState>,
    cv: RawCondvar,
    /// `true` from successful creation until teardown. Lets `delete()`
    /// claim teardown exactly once and makes every clone see it.
    ready: AtomicBool,
    /// Current period, in microseconds.
    us: AtomicU32,
    oneshot: AtomicBool,
    /// The background thread, so teardown can wake and join it.
    thread: Mutex<Option<Thread>>,
}

impl TimerShared {
    /// Stops the background thread and reaps it, at most once however many
    /// handles ask for it. Shared by [`TimerFn::delete`] and by
    /// [`TimerOwner`]'s `Drop`.
    ///
    /// Joins the background thread, so once this returns no callback is
    /// running or will run - except when called *from* the background
    /// thread (a callback deleting its timer or dropping its last handle),
    /// which detaches instead and lets the thread exit after the callback.
    fn destroy(&self) {
        self.ready.store(false, Ordering::Release);

        if let Ok(mut state) = self.state.lock() {
            state.exit = true;
            state.deadline = None;
        }
        self.cv.notify_all();

        let Ok(mut guard) = self.thread.lock() else {
            return;
        };

        let Some(bg_thread) = guard.take() else {
            return;
        };

        if *bg_thread == unsafe { pthread_self() } {
            // Joining would be joining ourselves, so detach instead and let
            // the thread release itself once it sees `exit`.
            bg_thread.detach();
            return;
        }

        bg_thread.delete();
    }
}

/// Ownership token held by every [`Timer`] handle the user creates or
/// clones, but never by the handle given to the callback (nor by the
/// background thread). Dropping the last one destroys the timer - the RAII
/// half of [`TimerFn::delete`], and the reason `Timer` itself has no `Drop`
/// of its own (a per-handle `Drop` would tear the timer down as soon as
/// *any* clone was dropped).
///
/// Keeping it apart from [`TimerShared`] is what makes the drop
/// deterministic: if a callback were running with an owning reference, the
/// user's last drop would not be the last one, and the timer would only
/// stop after that callback - on the background thread, after the user's
/// drop had already returned.
struct TimerOwner(Arc<TimerShared>);

impl Drop for TimerOwner {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

/// A software timer backed by a dedicated background thread that sleeps
/// until the timer's next deadline and invokes the user callback. Freely
/// [`Clone`]-able - every clone shares the same underlying timer. See
/// [`Timer::new`] for a complete, testable example.
#[derive(Clone)]
pub struct Timer {
    /// Opaque identifier of the underlying timer, exposed for diagnostics
    /// (`Debug`/`Display`). `null` once [`TimerFn::delete`]d.
    pub handle: TimerHandle,
    /// In the same fixed-size buffer every other named object in this crate
    /// uses. `Bytes` is `Copy`, so handing a named handle to the callback on
    /// every firing costs nothing.
    name: Bytes<MAX_TASK_NAME_LEN>,
    callback: Option<Arc<TimerFnPtr>>,
    param: Option<TimerParam>,
    /// `Some` for handles the user owns; `None` for the borrowed handle
    /// passed to the callback (see [`TimerOwner`]).
    owner: Option<Arc<TimerOwner>>,
    shared: Option<Arc<TimerShared>>,
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

/// Converts a tick count (this crate's ticks are milliseconds, see
/// [`TICK_PERIOD_MS`]) to microseconds, saturating instead of overflowing
/// `u32`.
fn ticks_to_us(ticks: TickType) -> u32 {
    (ticks as u64).saturating_mul(TICK_PERIOD_MS).saturating_mul(1000).min(u32::MAX as u64) as u32
}

/// Current time on the monotonic clock.
fn monotonic_now() -> timespec {
    let mut now = timespec::default();
    unsafe {
        clock_gettime(CLOCK_MONOTONIC, &mut now);
    }
    now
}

/// `true` once `deadline` is no later than `now`.
fn is_due(deadline: &timespec, now: &timespec) -> bool {
    (deadline.tv_sec, deadline.tv_nsec) <= (now.tv_sec, now.tv_nsec)
}

/// `time` advanced by `us` microseconds.
fn add_us(time: &timespec, us: u32) -> timespec {
    let nanoseconds = (us as c_long % 1_000_000) * 1000 + time.tv_nsec;

    timespec {
        tv_sec: time.tv_sec + us as c_long / 1_000_000 + nanoseconds / NSECS_PER_SEC,
        tv_nsec: nanoseconds % NSECS_PER_SEC,
    }
}

/// Arms `shared`'s timer to fire `us` microseconds from now, or disarms it
/// if `us == 0`, and wakes the background thread so it picks up the change.
/// No-op (returns `False`) once the timer has been deleted.
fn arm(shared: &TimerShared, us: u32) -> OsalRsBool {
    if !shared.ready.load(Ordering::Acquire) {
        return OsalRsBool::False;
    }

    let Ok(mut state) = shared.state.lock() else {
        return OsalRsBool::False;
    };

    state.deadline = if us == 0 {
        None
    } else {
        Some(monotonic_deadline(Duration::from_micros(us as u64)))
    };

    drop(state);
    shared.cv.notify_all();

    OsalRsBool::True
}

/// Body of the background thread every [`Timer`] spawns: sleeps until the
/// deadline in `shared` passes, re-arms or disarms it, then invokes the user
/// callback with the lock released. See the module docs for the full
/// rationale.
///
/// Holding `shared` does not keep the timer alive (see [`TimerOwner`]): the
/// thread runs until teardown sets `exit`.
fn run_timer_thread(shared: Arc<TimerShared>, name: Bytes<MAX_TASK_NAME_LEN>, callback: Option<Arc<TimerFnPtr>>, mut param: Option<TimerParam>) -> Result<TimerParam> {
    loop {
        let Ok(mut state) = shared.state.lock() else {
            break;
        };

        // Sleep until the deadline passes. Any change to the deadline (or
        // teardown) wakes the condvar, and waits may also return spuriously,
        // so the state is re-examined after every wake-up.
        loop {
            if state.exit {
                break;
            }

            match state.deadline {
                None => shared.cv.wait(&state),
                Some(deadline) if is_due(&deadline, &monotonic_now()) => break,
                Some(deadline) => {
                    shared.cv.wait_until(&state, deadline);
                }
            }
        }

        if state.exit {
            break;
        }

        // Re-arm before running the callback, so a callback that calls
        // `stop`/`start`/`change_period` overrides this.
        state.deadline = if shared.oneshot.load(Ordering::Acquire) {
            None
        } else {
            let us = shared.us.load(Ordering::Acquire);
            let now = monotonic_now();
            let next = state.deadline.map(|deadline| add_us(&deadline, us)).unwrap_or(now);

            // Overran a whole period: coalesce the missed expirations.
            Some(if is_due(&next, &now) { add_us(&now, us) } else { next })
        };

        drop(state);

        if let Some(cb) = &callback {
            // The callback is handed a borrowed handle onto the same shared
            // state: it does not own the timer (`owner: None`), since
            // `TimerFnPtr` takes its `Box<dyn TimerFn>` by value and drops
            // it on return.
            let timer_self = Timer {
                handle: Arc::as_ptr(&shared) as TimerHandle,
                name,
                callback: callback.clone(),
                param: param.clone(),
                owner: None,
                shared: Some(shared.clone()),
            };

            if let Ok(new_param) = cb(Box::new(timer_self), param.clone()) {
                param = Some(new_param);
            }
        }
    }

    let final_param: TimerParam = match param {
        Some(p) => p,
        None => Arc::new(()),
    };

    Ok(final_param)
}

impl Timer {
    /// Same as [`Timer::new`], but accepts any [`ToTick`] period (e.g. a
    /// [`core::time::Duration`]) instead of a raw tick count. See the
    /// module-level docs above for a complete example.
    #[inline]
    pub fn new_with_to_tick<F>(name: &str, timer_period_in_ticks: impl ToTick, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static,
    {
        Self::new(name, timer_period_in_ticks.to_ticks(), auto_reload, param, callback)
    }

    /// Same as [`TimerFn::start`], but accepts any [`ToTick`] value (e.g. a
    /// [`core::time::Duration`]) instead of a raw tick count.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use core::time::Duration;
    ///
    /// let timer = Timer::new_with_to_tick("t", Duration::from_millis(50), false, None, |_t, _p| Ok(Arc::new(()))).unwrap();
    /// assert_eq!(timer.start_with_to_tick(Duration::from_millis(10)), osal_rs::utils::OsalRsBool::True);
    /// ```
    #[inline]
    pub fn start_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.start(ticks_to_wait.to_ticks())
    }

    /// Same as [`TimerFn::stop`], but accepts any [`ToTick`] value instead
    /// of a raw tick count.
    #[inline]
    pub fn stop_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.stop(ticks_to_wait.to_ticks())
    }

    /// Same as [`TimerFn::reset`], but accepts any [`ToTick`] value instead
    /// of a raw tick count.
    #[inline]
    pub fn reset_with_to_tick(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.reset(ticks_to_wait.to_ticks())
    }

    /// Same as [`TimerFn::change_period`], but accepts any [`ToTick`] values
    /// instead of raw tick counts.
    #[inline]
    pub fn change_period_with_to_tick(&self, new_period_in_ticks: impl ToTick, new_period_ticks: impl ToTick) -> OsalRsBool {
        self.change_period(new_period_in_ticks.to_ticks(), new_period_ticks.to_ticks())
    }

    /// Same as [`TimerFn::delete`], but accepts any [`ToTick`] value instead
    /// of a raw tick count.
    #[inline]
    pub fn delete_with_to_tick(&mut self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        self.delete(ticks_to_wait.to_ticks())
    }

    /// Creates a new timer named `name`, firing `callback` every
    /// `timer_period_in_ticks` ticks if `auto_reload` (one-shot otherwise).
    /// `param` is handed to the first callback invocation; each invocation
    /// can return an updated value for the next one. The timer is created
    /// stopped - call [`TimerFn::start`] to arm it.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let timer = Timer::new("t", 50, false, None, |_timer, _param| Ok(Arc::new(()))).unwrap();
    /// assert!(!timer.is_null());
    /// ```
    pub fn new<F>(name: &str, timer_period_in_ticks: TickType, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn TimerFn>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static,
    {
        let shared = Arc::new(TimerShared {
            state: Mutex::new(TimerState::default()),
            cv: RawCondvar::new(),
            ready: AtomicBool::new(false),
            us: AtomicU32::new(ticks_to_us(timer_period_in_ticks)),
            oneshot: AtomicBool::new(!auto_reload),
            thread: Mutex::new(None),
        });

        let name = Bytes::<MAX_TASK_NAME_LEN>::from_str(name);

        let mut timer = Self {
            handle: Arc::as_ptr(&shared) as TimerHandle,
            name,
            callback: Some(Arc::new(callback)),
            param,
            owner: Some(Arc::new(TimerOwner(shared.clone()))),
            shared: Some(shared.clone()),
        };

        let bg_shared = shared.clone();
        let bg_name = name;
        let bg_callback = timer.callback.clone();
        let bg_param = timer.param.clone();

        let mut bg_thread = Thread::new(TIMER_THREAD_NAME, TIMER_THREAD_STACK, timer_thread_priority());
        let bg_thread = match bg_thread.spawn_simple(move || run_timer_thread(bg_shared.clone(), bg_name, bg_callback.clone(), bg_param.clone())) {
            Ok(thread) => thread,
            Err(err) => {
                timer.handle = null_mut();
                return Err(err);
            }
        };

        *shared.thread.lock().unwrap() = Some(bg_thread);
        shared.ready.store(true, Ordering::Release);

        Ok(timer)
    }
}

impl TimerFn for Timer {
    /// Returns `true` if this timer has been [`TimerFn::delete`]d (by this
    /// handle or by any clone of it).
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut timer = Timer::new("t", 50, false, None, |_t, _p| Ok(Arc::new(()))).unwrap();
    /// assert!(!timer.is_null());
    ///
    /// timer.delete(0);
    /// assert!(timer.is_null());
    /// ```
    fn is_null(&self) -> bool {
        match &self.shared {
            Some(shared) => !shared.ready.load(Ordering::Acquire),
            None => true,
        }
    }

    /// Arms the timer to fire after its configured period. See the
    /// module-level docs above for a complete example.
    fn start(&self, _ticks_to_wait: TickType) -> OsalRsBool {
        let Some(shared) = &self.shared else {
            return OsalRsBool::False;
        };

        arm(shared, shared.us.load(Ordering::Acquire))
    }

    /// Disarms the timer; a no-op if it isn't currently running.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static FIRED: AtomicBool = AtomicBool::new(false);
    ///
    /// let timer = Timer::new("t", 20, false, None, |_t, _p| {
    ///     FIRED.store(true, Ordering::SeqCst);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// timer.start(0);
    /// timer.stop(0); // cancels before it can fire
    ///
    /// System::delay(50);
    /// assert!(!FIRED.load(Ordering::SeqCst));
    /// ```
    fn stop(&self, _ticks_to_wait: TickType) -> OsalRsBool {
        let Some(shared) = &self.shared else {
            return OsalRsBool::False;
        };

        arm(shared, 0)
    }

    /// Restarts the countdown from now, using the current period -
    /// equivalent to calling [`TimerFn::start`] again, whether the timer was
    /// previously running or stopped.
    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool {
        // Arming always restarts the countdown from now, whether the timer
        // was previously running or stopped.
        self.start(ticks_to_wait)
    }

    /// Updates the timer's period and immediately (re)arms it with the new
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static FIRED: AtomicBool = AtomicBool::new(false);
    ///
    /// let timer = Timer::new("t", 10_000, false, None, |_t, _p| {
    ///     FIRED.store(true, Ordering::SeqCst);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// // Original 10s period would never fire in time; shrink it to 10ms.
    /// timer.change_period(10, 0);
    ///
    /// System::delay(50);
    /// assert!(FIRED.load(Ordering::SeqCst));
    /// ```
    fn change_period(&self, new_period_in_ticks: TickType, ticks_to_wait: TickType) -> OsalRsBool {
        let Some(shared) = &self.shared else {
            return OsalRsBool::False;
        };

        shared.us.store(ticks_to_us(new_period_in_ticks), Ordering::Release);
        self.start(ticks_to_wait)
    }

    /// Stops the timer and reaps its background thread,
    /// resetting this [`Timer`] to its "null" state. See
    /// [`TimerFn::is_null`] for a complete example.
    fn delete(&mut self, _ticks_to_wait: TickType) -> OsalRsBool {
        // Giving up the shared state is what makes this handle null; the
        // timer itself is destroyed by whichever handle gets there first,
        // and every clone sees it through the `ready` flag.
        let Some(shared) = self.shared.take() else {
            return OsalRsBool::False;
        };

        shared.destroy();
        self.owner = None;

        self.handle = null_mut();
        OsalRsBool::True
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
            .field("has_callback", &self.callback.is_some())
            .field("has_param", &self.param.is_some())
            .field("is_null", &self.is_null())
            .finish()
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Timer {{ name: {}, handle: {:?} }}", self.name, self.handle)
    }
}
