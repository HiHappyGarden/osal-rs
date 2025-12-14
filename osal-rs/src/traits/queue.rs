use super::ToTick;
use crate::os::ToBytes;
use crate::os::types::UBaseType;
use crate::utils::Result;



pub trait Queue {
    fn new (size: UBaseType, message_size: UBaseType) -> Result<Self>
    where 
        Self: Sized;

    fn fetch(&self, buffer: &mut [u8], time: impl ToTick) -> Result<()>;

    fn fetch_from_isr(&self, buffer: &mut [u8]) -> Result<()>;
    
    fn post(&self, item: &[u8], time: impl ToTick) -> Result<()>;

    fn post_from_isr(&self, item: &[u8]) -> Result<()>;

    fn delete(&mut self);
}

pub trait QueueStreamed<T> 
where 
    T: ToBytes + Sized {

    fn new (size: UBaseType, message_size: UBaseType) -> Result<Self>
    where 
        Self: Sized;

    fn fetch(&self, buffer: &mut T, time: impl ToTick) -> Result<()>;

    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()>;
    
    fn post(&self, item: &T, time: impl ToTick) -> Result<()>;

    fn post_from_isr(&self, item: &T) -> Result<()>;

    fn delete(&mut self);
}