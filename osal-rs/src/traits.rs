
mod system;
mod thread;
mod tick;

pub use crate::traits::system::System as SystemFn;
pub use crate::traits::thread::{Thread as ThreadFn, ThreadParam, ThreadFnPtr, ThreadNotification};
pub use crate::traits::tick::{*, Duration as DurationFn};
