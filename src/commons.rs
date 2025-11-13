use alloc::sync::Arc;
use core::any::Any;

pub const WAIT_FOREVER: u32 = 0xFFFFFFFF;

pub type ThreadFunc = dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static;
