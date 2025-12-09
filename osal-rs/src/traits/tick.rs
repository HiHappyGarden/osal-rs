use crate::os::types::TickType;


pub trait ToTick {
    type Target;

    fn get(&self) -> Self::Target;
} 

pub trait Duration : ToTick<Target = TickType> + Sized + Copy
{
    fn new_sec(sec: impl Into<TickType>) -> Self;
    fn new_millis(millis: impl Into<TickType>) -> Self;
    fn new_micros(micros: impl Into<TickType>) -> Self;
}

