use core::any::Any;
use crate::Result;
use alloc::boxed::Box;

pub trait Timer {

    fn new<F>(us: u64, handler: F, oneshot: bool) -> Self
    where
        F: Fn(&mut Self, Option<Box<dyn Any>>) + Send + Sync + 'static,
        Self: Sized;

    fn create(&mut self, param: Option<Box<dyn Any>>) -> Result<()>;

    fn set(&mut self, us: u64) -> Result<()>;

    fn set_from_isr(&mut self, us: u64) -> Result<()>;

    fn start(&mut self);

    fn start_from_isr(&mut self);

    fn stop(&mut self);

    fn stop_from_isr(&mut self);
}