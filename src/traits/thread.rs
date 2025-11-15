use alloc::sync::Arc;
use core::any::Any;
use core::ffi::c_void;
use crate::types::Result;
use crate::traits::thread_priority::ThreadPriority;

pub trait Thread<T> {

     fn create<F>(
        callback: F,
        name: &str,
        stack: u32,
        param: Option<Arc<dyn Any + Send + Sync>>,
        priority: impl ThreadPriority
    ) -> Result<T>
     where
         F: Fn(Option<Arc<dyn Any + Send + Sync>>) -> Result<Arc<dyn Any + Send + Sync>> + Send + Sync + 'static;

    fn delete_current();

    fn suspend(&self);

    fn resume(&self);

    fn join(&self, retval: *mut c_void) -> Result<i32>;
}