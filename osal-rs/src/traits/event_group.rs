
use crate::os::types::{BaseType, EventBits};
use crate::utils::Result;
use super::ToTick;

pub trait EventGroup {
    fn new() -> Self 
    where 
        Self: Sized;

    fn set(&mut self, bits: EventBits) -> EventBits;

    fn set_from_isr(&mut self, bits: EventBits, higher_priority_task_woken: &mut BaseType) -> Result<()>;

    fn get(&self) -> EventBits;

    fn get_from_isr(&self) -> EventBits;

    fn clear(&mut self, bits: EventBits) -> EventBits;

    fn clear_from_isr(&mut self, bits: EventBits) -> Result<()>;

    fn wait(&mut self, mask: EventBits, timeout_ticks: impl ToTick) -> EventBits;

}