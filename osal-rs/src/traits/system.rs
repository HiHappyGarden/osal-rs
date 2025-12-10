use crate::os::types::{BaseType, TickType};
use crate::os::{ThreadState, ToTick};
use crate::os::SystemState;

pub trait System {
    fn start();
    fn get_state() -> ThreadState;
    fn suspend_all();
    fn resume_all() -> BaseType;
    fn stop();
    fn get_tick_count() -> TickType;
    fn count_threads() -> usize;
    fn get_all_thread() -> SystemState;
    fn delay(ticks: impl ToTick);
    fn delay_until(previous_wake_time: &mut TickType, time_increment: impl ToTick);
}