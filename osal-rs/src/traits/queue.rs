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

pub trait QueueTyped<T> 
where 
    T: ToBytes + Sized {

    fn typed_new (size: UBaseType, message_size: UBaseType) -> Result<Self>
    where 
        Self: Sized;

    fn typed_fetch(&self, buffer: &mut T, time: impl ToTick) -> Result<()>;

    fn typed_fetch_from_isr(&self, buffer: &mut T) -> Result<()>;
    
    fn typed_post(&self, item: &T, time: impl ToTick) -> Result<()>;

    fn typed_post_from_isr(&self, item: &T) -> Result<()>;

    fn typed_delete(&mut self);
}