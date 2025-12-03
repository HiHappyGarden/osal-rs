
use crate::Result;

pub trait Event {

    fn new() -> Self where Self: Sized;
    fn wait(&mut self, mask: u32, value: &mut u32, time: u64) -> Result<()>;

    fn wait_from_isr(&mut self, mask: u32, value: &mut u32, time: u64) -> Result<()>;

    fn set(&mut self, value: u32);

    fn set_from_isr(&mut self, value: u32);

    fn get(&mut self) -> u32;

    fn get_from_isr(&mut self) -> u32;

    fn clear(&mut self, value: u32);

    fn clear_from_isr(&mut self, value: u32);
}

