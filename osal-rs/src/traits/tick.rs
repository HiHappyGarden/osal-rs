use crate::os::types::TickType;


pub trait ToTick : Sized + Copy {
    fn to_ticks(&self) -> TickType;
} 


pub trait FromTick {
    fn ticks(&mut self, tick: TickType);
}