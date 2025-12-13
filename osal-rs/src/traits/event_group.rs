use crate::utils::Result;
use crate::os::types::EventBits;
use super::ToTick;

pub trait EventGroup {
    fn new() -> Result<Self> 
    where 
        Self: Sized;

    fn set(&mut self, bits: EventBits) -> EventBits;

    fn set_from_isr(&mut self, bits: EventBits) -> Result<()>;

    fn get(&self) -> EventBits;

    fn get_from_isr(&self) -> EventBits;

    fn clear(&mut self, bits: EventBits) -> EventBits;

    fn clear_from_isr(&mut self, bits: EventBits) -> Result<()>;

    fn wait(&mut self, mask: EventBits, timeout_ticks: impl ToTick) -> EventBits;

    fn delete(&mut self);
}