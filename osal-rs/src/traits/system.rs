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

use core::time::Duration;

use crate::os::types::{BaseType, TickType};
use crate::os::{ThreadState};
use crate::os::SystemState;
use crate::utils::OsalRsBool;

pub trait System {
    fn start();
    fn get_state() -> ThreadState;
    fn suspend_all();
    fn resume_all() -> BaseType;
    fn stop();
    fn get_tick_count() -> TickType;
    fn get_current_time_us () -> Duration;
    fn get_us_from_tick(duration: &Duration) -> TickType;
    fn count_threads() -> usize;
    fn get_all_thread() -> SystemState;
    fn delay(ticks: TickType);
    fn delay_until(previous_wake_time: &mut TickType, time_increment: TickType);
    fn critical_section_enter();
    fn critical_section_exit();
    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool;
    fn yield_from_isr(higher_priority_task_woken: BaseType);
    fn end_switching_isr( switch_required: BaseType );
    fn get_free_heap_size() -> usize;
}
