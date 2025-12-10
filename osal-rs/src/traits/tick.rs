use crate::os::types::TickType;


pub trait ToTick : Sized + Copy {
    fn get_tick(&self) -> TickType;
} 


pub trait FromTick {
    fn set_tick(&mut self, tick: TickType);
}