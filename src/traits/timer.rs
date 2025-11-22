use core::any::Any;
use crate::Result;
use alloc::sync::Arc;


pub type TimerFunc = dyn Fn(&mut dyn Timer, Option<Arc<dyn Any + Send + Sync>>) + Send + Sync + 'static;

pub trait Timer {

    fn new<F>(us: u64, callback: F, param: Option<Arc<dyn Any + Send + Sync>>, one_shot: bool) -> Self
    where
        F: Fn(&mut dyn Timer, Option<Arc<dyn Any + Send + Sync>>) + Send + Sync + 'static,
        Self: Sized;

    fn set(&mut self, us: u64) -> Result<()>;

    fn set_from_isr(&mut self, us: u64) -> Result<()>;

    fn start(&mut self) -> Result<()>;

    fn start_from_isr(&mut self) -> Result<()>;

    fn stop(&mut self);

    fn stop_from_isr(&mut self);
}