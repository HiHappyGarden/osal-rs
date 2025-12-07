
mod system;
mod thread;

pub use crate::traits::system::System as SystemFn;
pub use crate::traits::thread::Thread as ThreadFn;
pub use crate::traits::thread::ThreadParam;
pub use crate::traits::thread::ThreadFnPtr;