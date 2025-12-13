use super::ToTick;
use crate::os::types::UBaseType;
use crate::utils::Result;



pub trait Queue {
    fn new (size: UBaseType, message_size: UBaseType) -> Result<Self>
    where 
        Self: Sized;

    fn fetch(&mut self, buffer: &mut [u8], time: impl ToTick) -> Result<()>;

    fn fetch_from_isr(&mut self, buffer: &mut [u8]) -> Result<()>;
    
    fn post(&mut self, item: &[u8], time: impl ToTick) -> Result<()>;

    fn post_from_isr(&mut self, item: &[u8]) -> Result<()>;

    fn delete(&mut self);
}