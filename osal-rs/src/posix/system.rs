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

use core::ffi::c_long;
use core::ops::Deref;
use core::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

use alloc::vec::Vec;

use crate::os::ThreadFn;
use crate::posix::ffi::{
    CLOCK_MONOTONIC, PTHREAD_ONCE_INIT, _SC_AVPHYS_PAGES, _SC_PAGESIZE, clock_gettime, nanosleep, pthread_once, pthread_once_t, sched_yield, sysconf, timespec,
};
use crate::posix::thread::{Thread, all_registered_threads, registered_thread_count};
use crate::posix::types::{BaseType, TickType, UBaseType};
use crate::traits::{SystemFn, ThreadMetadata, ThreadState, ToTick};
use crate::utils::OsalRsBool;

static RUN: AtomicBool = AtomicBool::new(true);

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
    #[inline]
    pub fn delay_with_to_tick(ticks: impl ToTick) {
        Self::delay(ticks.to_ticks());
    }

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
    fn start() {
        loop {
            if !RUN.load(Ordering::Acquire) {
                break;
            }
        }
    } 

    fn suspend_all() {
        for tm in all_registered_threads() {
            if let Ok(t) = Thread::new_with_handle(tm.thread, tm.name.as_str(), tm.stack_depth, tm.current_priority) {
                if tm.state == ThreadState::Ready || tm.state == ThreadState::Running {
                    t.suspend();
                }
            }
        }
    }

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

    fn stop() {
        RUN.store(false, Ordering::Release);
    }

    fn get_tick_count() -> TickType {
        Self::elapsed().as_millis().min(TickType::MAX as u128) as TickType
    }

    fn get_current_time_us() -> Duration {
        Self::elapsed()
    }

    fn get_ms_from_tick(duration: &Duration) -> TickType {
        duration.as_millis().min(TickType::MAX as u128) as TickType
    }

    fn count_threads() -> usize {
        // +1 for the calling thread itself, which `get_all_thread()` below
        // always reports even when it wasn't spawned through this crate's API.
        1 + registered_thread_count()
    }

    fn get_all_thread() -> SystemState {
        SystemState {
            tasks: all_registered_threads(),
            total_run_time: Self::get_tick_count().min(TickType::MAX) as u32,
        }
    }

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

    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType) {
        let next_wake_time = previous_wake_time.saturating_add(time_increment);
        let now = Self::get_tick_count();

        if next_wake_time > now {
            Self::delay(next_wake_time - now);
        }

        *previous_wake_time = next_wake_time;
    }

    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool {
        let elapsed = Self::get_current_time_us().checked_sub(*timestamp).unwrap_or_default();

        if elapsed >= *time {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn yield_from_isr(higher_priority_task_woken: BaseType) {
        if higher_priority_task_woken != 0 {
            unsafe {
                sched_yield();
            }
        }
    }

    fn end_switching_isr(switch_required: BaseType) {
        if switch_required != 0 {
            unsafe {
                sched_yield();
            }
        }
    }

    fn critical_section_enter() {}

    fn critical_section_exit() {}

    fn critical_section_enter_from_isr() -> UBaseType {
        0
    }

    fn critical_section_exit_from_isr(_: UBaseType) {}

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