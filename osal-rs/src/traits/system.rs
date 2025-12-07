
use crate::{ThreadState, types::{BaseType, TickType}};


pub trait System {
    fn start();
    fn get_state() -> ThreadState;
    fn suspend_all();
    fn resume_all() -> BaseType;
    fn stop();
    fn get_tick_count() -> TickType;
    fn count_threads() -> usize;
}