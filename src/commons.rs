use alloc::sync::Arc;
use core::any::Any;

pub const WAIT_FOREVER: u32 = 0xFFFFFFFF;

pub type ThreadFunc = dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static;


#[derive(Clone)]
pub enum ThreadDefaultPriority {
    None = 0,
    Idle = 1,
    Low = 2,
    BelowNormal = 3,
    Normal = 4,
    AboveNormal = 5,
    High = 6,
    Realtime = 7,
    ISR = 8,
}
