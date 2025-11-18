
use crate::Result;

pub trait Semaphore {

    fn new(count: usize) -> Self where Self: Sized;

    fn wait(&mut self, time: u64) -> Result<()>;

    fn wait_from_isr(&mut self, time: u64) -> Result<()>;

    fn signal(&mut self);

    fn signal_from_isr(&mut self);
}