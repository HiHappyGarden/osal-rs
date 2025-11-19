use std::any::Any;
use crate::Result;
pub trait Timer {

    fn new<F>(us: u64, handler: F, oneshot: bool) -> Self
    where
        F: Fn(&mut Self, Option<Box<dyn Any>>) + 'static,
        Self: Sized;

    fn create(&mut self, param: Option<Box<dyn Any>>) -> Result<()>;

    fn set(&mut self, us: u64) -> Result<()>;

    fn set_from_isr(&mut self, us: u64) -> Result<()>;

    fn start(&mut self);

    fn start_from_isr(&mut self);

    fn stop(&mut self);

    fn stop_from_isr(&mut self);
}