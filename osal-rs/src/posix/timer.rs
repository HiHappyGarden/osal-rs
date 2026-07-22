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
//! does, so each [`Timer`] gets its own dedicated background thread plus a
//! real kernel timer (`timer_create(2)`) that notifies that thread directly:
//!
//! 1. `SIGALRM` is blocked in the calling thread (`sigprocmask`) before the
//!    background thread is spawned, so the mask — and with it, the block —
//!    is inherited by the new thread too. A blocked signal isn't discarded;
//!    it becomes *pending* until something explicitly consumes it.
//! 2. The background thread publishes its kernel thread ID (`gettid(2)`,
//!    distinct from its `pthread_t`) and then loops on `sigwait(3)`, which
//!    synchronously consumes one pending, blocked `SIGALRM` at a time and
//!    invokes the user callback in response.
//! 3. `timer_create` is configured with `SIGEV_THREAD_ID` notification,
//!    targeting that kernel thread ID directly — so this timer's expirations
//!    can only ever wake up this timer's own background thread, never any
//!    other timer's or unrelated code's.
//!
//! This mirrors a common pattern for per-thread POSIX timers (create a
//! dedicated waiter thread, mask + `sigwait` instead of an async-signal
//! handler, `SIGEV_THREAD_ID` to target it precisely).
//!
//! # Caveats inherited from this design
//!
//! - Blocking `SIGALRM` in the calling thread is permanent for that thread:
//!   this crate never unblocks it afterwards, so a thread that creates a
//!   `Timer` can no longer receive `SIGALRM` itself.
//! - A one-shot timer's background thread exits after its single callback
//!   invocation. Calling `start()`/`reset()` again on an already-fired
//!   one-shot timer re-arms the kernel timer, but nothing is left running to
//!   consume its `SIGALRM` — create a new `Timer` instead of reusing one.
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

use core::ffi::{c_int, c_long, c_void};
use core::fmt::{Debug, Display};
use core::ops::Deref;
use core::ptr::null_mut;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, Ordering};

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use crate::os::ThreadFn;
use crate::posix::config::TICK_PERIOD_MS;
use crate::posix::ffi::{
    CLOCK_MONOTONIC, SIGALRM, SIGEV_THREAD_ID, SIG_BLOCK, gettid, itimerspec, pthread_kill, sched_yield, sigaddset, sigemptyset, sigevent, sigevent_un, sigprocmask, sigset_t, sigwait,
    timer_create, timer_delete, timer_settime, timer_t, timespec,
};
use crate::posix::thread::Thread;
use crate::posix::types::{StackType, TickType, TimerHandle, UBaseType};
use crate::traits::{TimerFn, TimerFnPtr, TimerParam, ToTick};
use crate::utils::{Error, OsalRsBool, Result};

/// Name (glibc `pthread_setname_np`, `<= 15` chars) given to every timer's
/// background thread. Fixed rather than derived from the timer's own name so
/// it's always valid regardless of what the caller passed to `Timer::new`.
const TIMER_THREAD_NAME: &str = "os_timer";

/// Stack size requested for a timer's background thread. `Thread::spawn`
/// enforces its own safe minimum regardless, so this only matters as a
/// lower bound.
const TIMER_THREAD_STACK: StackType = 1024;

/// Priority given to a timer's background thread. Only meaningful with the
/// `sched_fifo` feature enabled; otherwise the thread inherits the creating
/// thread's scheduling policy/priority.
const TIMER_THREAD_PRIORITY: UBaseType = 1;

const NSECS_PER_SEC: u64 = 1_000_000_000;

/// State shared, via `Arc`, between every clone of a given [`Timer`] and its
/// background thread.
///
/// [`Timer`] itself is freely `Clone` (matching every other handle type in
/// this crate), but the POSIX timer, its background thread, and its
/// mutable period all belong to one underlying resource — this is that
/// resource.
struct TimerShared {
    /// The real POSIX timer, once `ready` is set. Not read before then:
    /// on modern glibc/Linux, `timer_t` is a small kernel-assigned integer
    /// cast to a pointer, so the *first* timer a process ever creates gets
    /// the all-zero handle — a bit pattern that would otherwise look
    /// indistinguishable from "not created yet".
    timerid: AtomicPtr<c_void>,
    /// Set once `timerid` holds a real value from a successful
    /// `timer_create`. Guards every read of `timerid` instead of checking
    /// it for null (see `timerid`'s docs for why null isn't a safe sentinel
    /// here), and lets `delete()` claim deletion exactly once.
    ready: AtomicBool,
    /// Kernel TID (`gettid()`) of the background thread, published once it
    /// starts running; 0 until then.
    thread_id: AtomicI32,
    /// Current period, in microseconds.
    us: AtomicU32,
    oneshot: AtomicBool,
    /// Set by `delete()` to tell the background thread's `sigwait` loop to
    /// stop instead of invoking the callback again.
    exit: AtomicBool,
    /// The background thread, so `delete()` can wake and join it.
    thread: Mutex<Option<Thread>>,
}

#[derive(Clone)]
pub struct Timer {
    pub handle: TimerHandle,
    name: String,
    callback: Option<Arc<TimerFnPtr>>,
    param: Option<TimerParam>,
    shared: Option<Arc<TimerShared>>,
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

/// Converts a tick count (this crate's ticks are milliseconds, see
/// [`TICK_PERIOD_MS`]) to microseconds, saturating instead of overflowing
/// `u32` (the width `timer_settime`'s nanosecond math is done in below).
fn ticks_to_us(ticks: TickType) -> u32 {
    (ticks as u64).saturating_mul(TICK_PERIOD_MS).saturating_mul(1000).min(u32::MAX as u64) as u32
}

/// Arms `shared`'s timer to fire `us` microseconds from now, or disarms it
/// if `us == 0` (per `timer_settime(2)`, an all-zero `it_value` always
/// disarms regardless of `it_interval`). No-op (returns `False`) if the
/// timer hasn't been created yet.
fn arm(shared: &TimerShared, us: u32) -> OsalRsBool {
    if !shared.ready.load(Ordering::Acquire) {
        return OsalRsBool::False;
    }

    let timerid = shared.timerid.load(Ordering::Acquire);

    let nanoseconds = (us as u64) * 1000;
    let it_value = timespec {
        tv_sec: (nanoseconds / NSECS_PER_SEC) as c_long,
        tv_nsec: (nanoseconds % NSECS_PER_SEC) as c_long,
    };

    let it_interval = if us == 0 || shared.oneshot.load(Ordering::Acquire) {
        timespec::default()
    } else {
        it_value
    };

    let its = itimerspec { it_interval, it_value };

    match unsafe { timer_settime(timerid, 0, &its, null_mut()) } {
        0 => OsalRsBool::True,
        _ => OsalRsBool::False,
    }
}

/// Body of the background thread every [`Timer`] spawns: publishes its
/// kernel TID, then loops accepting one blocked `SIGALRM` at a time via
/// `sigwait` and invoking the user callback in response. See the module
/// docs for the full rationale.
fn run_timer_thread(shared: Arc<TimerShared>, callback: Option<Arc<TimerFnPtr>>, mut param: Option<TimerParam>, timer_self: Timer) -> Result<TimerParam> {
    shared.thread_id.store(unsafe { gettid() }, Ordering::Release);

    let mut sigset = sigset_t::default();
    unsafe {
        sigemptyset(&mut sigset);
        sigaddset(&mut sigset, SIGALRM);
    }

    while !shared.exit.load(Ordering::Acquire) {
        let mut sig: c_int = 0;

        if unsafe { sigwait(&sigset, &mut sig) } != 0 || sig != SIGALRM {
            continue;
        }

        // The signal that just woke us might be the real timer expiration,
        // or the artificial one `delete()` sends to break out of `sigwait`.
        if shared.exit.load(Ordering::Acquire) {
            break;
        }

        if let Some(cb) = &callback {
            if let Ok(new_param) = cb(Box::new(timer_self.clone()), param.clone()) {
                param = Some(new_param);
            }
        }

        if shared.oneshot.load(Ordering::Acquire) {
            break;
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
            timerid: AtomicPtr::new(null_mut()),
            ready: AtomicBool::new(false),
            thread_id: AtomicI32::new(0),
            us: AtomicU32::new(ticks_to_us(timer_period_in_ticks)),
            oneshot: AtomicBool::new(!auto_reload),
            exit: AtomicBool::new(false),
            thread: Mutex::new(None),
        });

        let mut timer = Self {
            handle: null_mut(),
            name: name.to_string(),
            callback: Some(Arc::new(callback)),
            param,
            shared: Some(shared.clone()),
        };

        // Block SIGALRM here so the background thread we're about to spawn
        // inherits it blocked too (see module docs).
        let mut sigset = sigset_t::default();
        unsafe {
            sigemptyset(&mut sigset);
            sigaddset(&mut sigset, SIGALRM);
            sigprocmask(SIG_BLOCK, &sigset, null_mut());
        }

        let bg_shared = shared.clone();
        let bg_callback = timer.callback.clone();
        let bg_param = timer.param.clone();
        let bg_self = timer.clone();

        let mut bg_thread = Thread::new(TIMER_THREAD_NAME, TIMER_THREAD_STACK, TIMER_THREAD_PRIORITY);
        let bg_thread = bg_thread.spawn_simple(move || run_timer_thread(bg_shared.clone(), bg_callback.clone(), bg_param.clone(), bg_self.clone()))?;

        // Wait until the background thread has published its kernel TID,
        // needed below to target it with SIGEV_THREAD_ID.
        while shared.thread_id.load(Ordering::Acquire) == 0 {
            unsafe {
                sched_yield();
            }
        }

        let sev = sigevent {
            sigev_notify: SIGEV_THREAD_ID,
            sigev_signo: SIGALRM,
            sigev_un: sigevent_un {
                tid: shared.thread_id.load(Ordering::Acquire),
            },
            ..Default::default()
        };

        let mut timerid: timer_t = null_mut();
        let ret = unsafe { timer_create(CLOCK_MONOTONIC, &sev, &mut timerid) };

        if ret != 0 {
            shared.exit.store(true, Ordering::Release);
            unsafe {
                pthread_kill(*bg_thread, SIGALRM);
            }
            bg_thread.delete();
            return Err(Error::ReturnWithCode(ret));
        }

        shared.timerid.store(timerid, Ordering::Release);
        shared.ready.store(true, Ordering::Release);
        *shared.thread.lock().unwrap() = Some(bg_thread);

        timer.handle = timerid;
        Ok(timer)
    }
}

impl TimerFn for Timer {
    /// Returns `true` if this timer has been [`TimerFn::delete`]d (or the
    /// underlying kernel timer failed to create).
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
        // A relative `timer_settime` call always restarts the countdown
        // from now, whether the timer was previously running or stopped.
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

    /// Destroys the underlying kernel timer and its background thread,
    /// resetting this [`Timer`] to its "null" state. See
    /// [`TimerFn::is_null`] for a complete example.
    fn delete(&mut self, _ticks_to_wait: TickType) -> OsalRsBool {
        let Some(shared) = self.shared.take() else {
            return OsalRsBool::False;
        };

        // `swap` (not `load` + `store`) so concurrent `delete()` calls on
        // clones of the same `Timer` can't both attempt to delete the
        // underlying resources.
        if shared.ready.swap(false, Ordering::AcqRel) {
            let timerid = shared.timerid.load(Ordering::Acquire);
            unsafe {
                timer_delete(timerid);
            }
        }

        shared.exit.store(true, Ordering::Release);

        if let Ok(mut guard) = shared.thread.lock() {
            if let Some(bg_thread) = guard.take() {
                // Wake the background thread out of `sigwait` so it observes
                // `exit` and returns; harmless if it's already on its way
                // out because the timer just fired for the last time.
                unsafe {
                    pthread_kill(*bg_thread, SIGALRM);
                }
                bg_thread.delete();
            }
        }

        self.handle = null_mut();
        OsalRsBool::True
    }
}

impl Drop for Timer {
    fn drop(&mut self) {}
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
