use crate::os::types::{OsalRsBool, UBaseType};
use super::ToTick;
use crate::utils::Result;


pub trait Semaphore {
 
    fn new(max_count: UBaseType, initial_count: UBaseType) -> Result<Self> 
    where 
        Self: Sized;

    fn new_with_count(initial_count: UBaseType) -> Result<Self> 
    where 
        Self: Sized;

    fn wait(&mut self, ticks_to_wait: impl ToTick) -> OsalRsBool;

    fn wait_from_isr(&mut self) -> OsalRsBool;

    fn signal(&mut self) -> OsalRsBool;
    
    fn signal_from_isr(&mut self) -> OsalRsBool;
    
    fn delete(&mut self);

}
