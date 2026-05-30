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

use core::ops::Deref;
use core::time::Duration;

use alloc::vec;
use alloc::vec::Vec;
use std::sync::OnceLock;
use std::time::Instant;

use crate::posix::thread::{ThreadMetadata, ThreadState};
use crate::posix::types::{BaseType, TickType, UBaseType};
use crate::traits::{SystemFn, ToTick};
use crate::utils::OsalRsBool;

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

    fn start_time() -> &'static Instant {
        static START_TIME: OnceLock<Instant> = OnceLock::new();

        START_TIME.get_or_init(Instant::now)
    }

    fn elapsed() -> Duration {
        Self::start_time().elapsed()
    }
}

impl SystemFn for System {
    fn start() {}

    fn get_state() -> ThreadState {
        ThreadState::Running
    }

    fn suspend_all() {}

    fn resume_all() -> BaseType {
        0
    }

    fn stop() {}

    fn get_tick_count() -> TickType {
        Self::elapsed().as_millis().min(TickType::MAX as u128) as TickType
    }

    fn get_current_time_us() -> Duration {
        Self::elapsed()
    }

    fn get_us_from_tick(duration: &Duration) -> TickType {
        duration.as_millis().min(TickType::MAX as u128) as TickType
    }

    fn count_threads() -> usize {
        1
    }

    fn get_all_thread() -> SystemState {
        let mut thread = ThreadMetadata::default();
        thread.state = ThreadState::Running;

        SystemState {
            tasks: vec![thread],
            total_run_time: Self::get_tick_count().min(u32::MAX as TickType) as u32,
        }
    }

    fn delay(ticks: TickType) {
        std::thread::sleep(Duration::from_millis(ticks));
    }

    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType) {
        let next_wake_time = previous_wake_time.saturating_add(time_increment);
        let now = Self::get_tick_count();

        if next_wake_time > now {
            Self::delay(next_wake_time - now);
        }

        *previous_wake_time = next_wake_time;
    }

    fn critical_section_enter() {}

    fn critical_section_exit() {}

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
            std::thread::yield_now();
        }
    }

    fn end_switching_isr(switch_required: BaseType) {
        if switch_required != 0 {
            std::thread::yield_now();
        }
    }

    fn enter_critical() {}

    fn exit_critical() {}

    fn enter_critical_from_isr() -> UBaseType {
        0
    }

    fn exit_critical_from_isr(_saved_interrupt_status: UBaseType) {}

    fn get_free_heap_size() -> usize {
        0
    }
}