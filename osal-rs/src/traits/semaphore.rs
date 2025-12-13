use crate::os::types::UBaseType;
use super::ToTick;
use crate::utils::{OsalRsBool, Result};


pub trait Semaphore {
 
    fn new(max_count: UBaseType, initial_count: UBaseType) -> Result<Self> 
    where 
        Self: Sized;

    fn new_with_count(initial_count: UBaseType) -> Result<Self> 
    where 
        Self: Sized;

    fn wait(&self, ticks_to_wait: impl ToTick) -> OsalRsBool;

    fn wait_from_isr(&self) -> OsalRsBool;

    fn signal(&self) -> OsalRsBool;
    
    fn signal_from_isr(&self) -> OsalRsBool;
    
    fn delete(&mut self);

}
