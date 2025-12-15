use core::time::Duration;

use crate::os::types::{BaseType, TickType};
use crate::os::{ThreadState, ToTick};
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
    fn delay(ticks: impl ToTick);
    fn delay_until(previous_wake_time: &mut TickType, time_increment: impl ToTick);
    fn critical_section_enter();
    fn critical_section_exit();
    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool;
    fn yield_from_isr(higher_priority_task_woken: BaseType);
    fn end_switching_isr( switch_required: BaseType );
    fn get_free_heap_size() -> usize;
}
