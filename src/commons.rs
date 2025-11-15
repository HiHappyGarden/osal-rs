use alloc::sync::Arc;
use core::any::Any;
use core::fmt::Debug;

pub const WAIT_FOREVER: u32 = 0xFFFFFFFF;


pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(PartialEq)]
pub enum Error {
    Std(i32, &'static str)
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Std(code, msg) => write!(f, "Error::Std({}, {})", code, msg),
        }
    }
}

pub type ThreadFunc = dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static;