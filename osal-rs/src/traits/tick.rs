use crate::os::types::TickType;


pub trait ToTick : Sized + Copy {
    fn to_tick(&self) -> TickType;
} 


pub trait FromTick {
    fn tick(&mut self, tick: TickType);
}