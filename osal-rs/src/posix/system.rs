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

//! System-level control and timing for POSIX.
//!
//! [`System`] provides the scheduler-adjacent operations that don't belong
//! to any single primitive: starting/stopping the "run loop", timing
//! (`CLOCK_MONOTONIC`-based), and querying/suspending the threads spawned
//! through this crate's [`crate::os::Thread`] API. Unlike FreeRTOS, POSIX has
//! no real scheduler to hand control to, so [`System::start`] just spins
//! until [`System::stop`] is called from another thread.
//!
//! # Examples
//!
//! ```
//! use osal_rs::os::*;
//! use std::sync::Arc;
//!
//! // Something else must call `System::stop()` for `start()` to return.
//! let mut stopper = Thread::new("stopper", 1024, 1);
//! stopper.spawn_simple(|| {
//!     System::delay(10);
//!     System::stop();
//!     Ok(Arc::new(()))
//! }).unwrap();
//!
//! System::start(); // blocks here until `stop()` runs above
//! ```

use core::ffi::c_long;
use core::ops::Deref;
use core::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

use alloc::vec::Vec;

use crate::os::ThreadFn;
use crate::posix::ffi::{
    CLOCK_MONOTONIC, PTHREAD_ONCE_INIT, _SC_AVPHYS_PAGES, _SC_PAGESIZE, clock_gettime, nanosleep, pthread_once, pthread_once_t, pthread_self, sched_yield, sysconf, timespec,
};
use crate::posix::thread::{Thread, all_registered_threads, registered_thread_count};
use crate::posix::types::{BaseType, TickType, UBaseType};
use crate::traits::{SystemFn, ThreadMetadata, ThreadState, ToTick};
use crate::utils::OsalRsBool;

static RUN: AtomicBool = AtomicBool::new(true);

/// Snapshot returned by [`System::get_all_thread`]: every thread spawned
/// through this crate's [`crate::os::Thread`] API (plus the calling thread
/// itself), and the total elapsed run time at the moment of the snapshot.
/// Derefs to `&[ThreadMetadata]` for convenient iteration.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
///
/// let state = System::get_all_thread();
/// // The calling thread is always included.
/// assert!(!state.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct SystemState {
    pub tasks: Vec<ThreadMetadata>,
    pub total_run_time: u32,
}

impl Deref for SystemState {
    type Target = [ThreadMetadata];

    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

pub struct System;

impl System {
    /// Blocks like [`System::delay`], but accepts any [`ToTick`] duration
    /// (e.g. a [`core::time::Duration`]) instead of a raw tick count.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use core::time::Duration;
    ///
    /// let before = System::get_tick_count();
    /// System::delay_with_to_tick(Duration::from_millis(10));
    /// assert!(System::get_tick_count() >= before);
    /// ```
    #[inline]
    pub fn delay_with_to_tick(ticks: impl ToTick) {
        Self::delay(ticks.to_ticks());
    }

    /// Blocks like [`System::delay_until`], but accepts any [`ToTick`]
    /// increment (e.g. a [`core::time::Duration`]) instead of a raw tick
    /// count.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use core::time::Duration;
    ///
    /// let mut previous = System::get_tick_count();
    /// System::delay_until_with_to_tick(&mut previous, Duration::from_millis(5));
    /// ```
    #[inline]
    pub fn delay_until_with_to_tick(previous_wake_time: &mut TickType, time_increment: impl ToTick) {
        Self::delay_until(previous_wake_time, time_increment.to_ticks());
    }

    fn monotonic_now() -> Duration {
        let mut ts = timespec::default();
        unsafe { clock_gettime(CLOCK_MONOTONIC, &mut ts) };

        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }

    fn start_time() -> Duration {
        static mut ONCE: pthread_once_t = PTHREAD_ONCE_INIT;
        static mut START_TIME: Duration = Duration::ZERO;

        extern "C" fn init() {
            unsafe {
                START_TIME = System::monotonic_now();
            }
            // Without this, a caller landing within nanoseconds of the
            // epoch capture above would measure 0 elapsed ms (millisecond
            // resolution) on its very first read, since `pthread_once`
            // blocks every other racing caller until `init` returns.
            // Paid once per process, lazily, only if timing is ever used.
            System::delay(1);
        }

        unsafe {
            pthread_once(&raw mut ONCE, Some(init));
            START_TIME
        }
    }

    fn elapsed() -> Duration {
        Self::monotonic_now().checked_sub(Self::start_time()).unwrap_or_default()
    }
}

impl SystemFn for System {
    /// Spins until [`System::stop`] is called from another thread. There is
    /// no real scheduler on POSIX to hand control to, so this is just a busy
    /// loop over an atomic flag - unlike FreeRTOS, where the equivalent call
    /// never returns.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut stopper = Thread::new("stopper", 1024, 1);
    /// stopper.spawn_simple(|| {
    ///     System::delay(10);
    ///     System::stop();
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// System::start(); // blocks here until `stop()` runs above
    /// ```
    fn start() {
        loop {
            if !RUN.load(Ordering::Acquire) {
                break;
            }
        }
    }

    /// Suspends every currently `Ready`/`Running` thread spawned through
    /// this crate's [`crate::os::Thread`] API (see
    /// [`crate::os::ThreadFn::suspend`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut worker = Thread::new("worker", 1024, 1);
    /// worker.spawn_simple(|| {
    ///     System::delay(200);
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// System::delay(10); // give it a moment to start running
    /// System::suspend_all();
    /// assert!(System::resume_all() >= 1);
    /// ```
    fn suspend_all() {
        for tm in all_registered_threads() {
            if let Ok(t) = Thread::new_with_handle(tm.thread, tm.name.as_str(), tm.stack_depth, tm.current_priority) {
                if tm.state == ThreadState::Ready || tm.state == ThreadState::Running {
                    t.suspend();
                }
            }
        }
    }

    /// Resumes every currently `Suspended` thread spawned through this
    /// crate's [`crate::os::Thread`] API, returning how many were resumed.
    ///
    /// See [`System::suspend_all`] for a complete example.
    fn resume_all() -> BaseType {
        let mut count = 0;

        for tm in all_registered_threads() {
            if let Ok(t) = Thread::new_with_handle(tm.thread, tm.name.as_str(), tm.stack_depth, tm.current_priority) {
                if tm.state == ThreadState::Suspended {
                    t.resume();
                    count += 1;
                }
            }
        }

        count
    }

    /// Signals [`System::start`]'s spin loop to return. See
    /// [`System::start`] for a complete example.
    fn stop() {
        RUN.store(false, Ordering::Release);
    }

    /// Returns the number of ticks elapsed since the first time any of
    /// [`System::get_tick_count`]/[`System::get_current_time_us`] was called
    /// in this process (that first call defines tick `0`).
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let before = System::get_tick_count();
    /// System::delay(5);
    /// assert!(System::get_tick_count() >= before);
    /// ```
    fn get_tick_count() -> TickType {
        Self::elapsed().as_millis().min(TickType::MAX as u128) as TickType
    }

    /// Same reference point as [`System::get_tick_count`], but returned as a
    /// [`Duration`] instead of a raw tick count.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let before = System::get_current_time_us();
    /// System::delay(5);
    /// assert!(System::get_current_time_us() >= before);
    /// ```
    fn get_current_time_us() -> Duration {
        Self::elapsed()
    }

    /// Converts a [`Duration`] to POSIX ticks (milliseconds); see
    /// `crate::posix::duration` for the same conversion via [`ToTick`].
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use core::time::Duration;
    ///
    /// assert_eq!(System::get_ms_from_tick(&Duration::from_millis(250)), 250);
    /// ```
    fn get_ms_from_tick(duration: &Duration) -> TickType {
        duration.as_millis().min(TickType::MAX as u128) as TickType
    }

    /// Number of threads known to the system: every thread spawned through
    /// this crate's [`crate::os::Thread`] API, plus the calling thread
    /// itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// // Just the calling thread: nothing else has been spawned yet.
    /// assert_eq!(System::count_threads(), 1);
    /// ```
    fn count_threads() -> usize {
        // +1 for the calling thread itself, which `get_all_thread()` below
        // always reports even when it wasn't spawned through this crate's API.
        1 + registered_thread_count()
    }

    /// Returns a [`SystemState`] snapshot of every thread known to the
    /// system, mirroring [`System::count_threads`]'s "+1 for the caller"
    /// accounting.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let state = System::get_all_thread();
    /// assert_eq!(state.len(), System::count_threads());
    /// ```
    fn get_all_thread() -> SystemState {
        let mut tasks = all_registered_threads();

        // Mirror `count_threads()`'s +1: report the calling thread even when
        // it wasn't spawned through this crate's API. Skip it if the caller
        // is itself a registered thread (e.g. a spawned worker calling this
        // from within its own thread function), to avoid double-counting.
        let caller = unsafe { pthread_self() };
        if !tasks.iter().any(|metadata| metadata.thread == caller) {
            tasks.push(Thread::get_metadata_from_handle(caller));
        }

        SystemState {
            tasks,
            total_run_time: Self::get_tick_count().min(TickType::MAX) as u32,
        }
    }

    /// Blocks the calling thread for `ticks` (milliseconds on this
    /// backend), automatically resuming the sleep if interrupted by a
    /// signal before it elapsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let before = System::get_tick_count();
    /// System::delay(20);
    /// assert!(System::get_tick_count() - before >= 20);
    /// ```
    fn delay(ticks: TickType) {
        let mut req = timespec {
            tv_sec: (ticks / 1000) as c_long,
            tv_nsec: ((ticks % 1000) as c_long) * 1_000_000,
        };

        loop {
            let mut rem = timespec::default();

            if unsafe { nanosleep(&req, &mut rem) } == 0 {
                break;
            }

            // Interrupted by a signal before `req` elapsed: `rem` holds the
            // time still left to sleep, so resume with that. If the kernel
            // left `rem` untouched (a real error, not EINTR), it'll be zero
            // and the loop exits instead of spinning forever.
            if rem.tv_sec == 0 && rem.tv_nsec == 0 {
                break;
            }

            req = rem;
        }
    }

    /// Blocks until `*previous_wake_time + time_increment` (absolute ticks),
    /// then advances `*previous_wake_time` by `time_increment` - a fixed
    /// period loop that doesn't drift with the time spent doing work each
    /// iteration, unlike calling [`System::delay`] with the same increment
    /// every time.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let before = System::get_tick_count();
    /// let mut previous = before;
    /// System::delay_until(&mut previous, 20);
    ///
    /// assert_eq!(previous, before + 20);
    /// assert!(System::get_tick_count() >= previous);
    /// ```
    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType) {
        let next_wake_time = previous_wake_time.saturating_add(time_increment);
        let now = Self::get_tick_count();

        if next_wake_time > now {
            Self::delay(next_wake_time - now);
        }

        *previous_wake_time = next_wake_time;
    }

    /// Returns [`OsalRsBool::True`] once at least `time` has elapsed since
    /// `timestamp` (both measured against [`System::get_current_time_us`]'s
    /// clock).
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::utils::OsalRsBool;
    /// use core::time::Duration;
    ///
    /// let start = System::get_current_time_us();
    /// assert_eq!(System::check_timer(&start, &Duration::from_millis(500)), OsalRsBool::False);
    ///
    /// System::delay(20);
    /// assert_eq!(System::check_timer(&start, &Duration::from_millis(10)), OsalRsBool::True);
    /// ```
    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool {
        let elapsed = Self::get_current_time_us().checked_sub(*timestamp).unwrap_or_default();

        if elapsed >= *time {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    /// Yields the processor (`sched_yield(2)`) if `higher_priority_task_woken`
    /// is non-zero, a no-op otherwise. On FreeRTOS this triggers a context
    /// switch to a just-woken higher-priority task from within an ISR; POSIX
    /// has no real interrupt context, so this exists purely for API
    /// compatibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// System::yield_from_isr(1); // yields
    /// System::yield_from_isr(0); // no-op
    /// ```
    fn yield_from_isr(higher_priority_task_woken: BaseType) {
        if higher_priority_task_woken != 0 {
            unsafe {
                sched_yield();
            }
        }
    }

    /// Identical to [`System::yield_from_isr`] under a different name,
    /// matching FreeRTOS's `portEND_SWITCHING_ISR` naming convention.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// System::end_switching_isr(1);
    /// ```
    fn end_switching_isr(switch_required: BaseType) {
        if switch_required != 0 {
            unsafe {
                sched_yield();
            }
        }
    }

    /// No-op on POSIX: there is no real interrupt/scheduler state to guard,
    /// unlike FreeRTOS where this disables interrupts/the scheduler.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// System::critical_section_enter();
    /// System::critical_section_exit();
    /// ```
    fn critical_section_enter() {}

    /// See [`System::critical_section_enter`].
    fn critical_section_exit() {}

    /// ISR-context counterpart of [`System::critical_section_enter`]; always
    /// returns `0` (nothing to restore) since it's a no-op on POSIX.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let saved = System::critical_section_enter_from_isr();
    /// System::critical_section_exit_from_isr(saved);
    /// ```
    fn critical_section_enter_from_isr() -> UBaseType {
        0
    }

    /// See [`System::critical_section_enter_from_isr`].
    fn critical_section_exit_from_isr(_: UBaseType) {}

    /// POSIX processes don't have a fixed heap the way FreeRTOS does (the
    /// allocator can keep extending it via `mmap`/`brk`), so this reports
    /// available physical memory as the closest analogue.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// assert!(System::get_free_heap_size() > 0);
    /// ```
    fn get_free_heap_size() -> usize {
        // POSIX processes don't have a fixed heap the way FreeRTOS does (the
        // allocator can keep extending it via mmap/brk), so this reports
        // available physical memory as the closest analogue.
        let page_size = unsafe { sysconf(_SC_PAGESIZE) };
        let avail_pages = unsafe { sysconf(_SC_AVPHYS_PAGES) };

        if page_size <= 0 || avail_pages <= 0 {
            0
        } else {
            (page_size as usize).saturating_mul(avail_pages as usize)
        }
    }
}