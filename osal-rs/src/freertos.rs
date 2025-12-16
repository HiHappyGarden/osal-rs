pub mod allocator;
pub mod config;
pub mod duration;
pub mod event_group;
mod ffi;
pub mod mutex;
pub mod queue;
pub mod semaphore;
pub mod system;
pub mod thread;
pub mod timer;
pub mod types;

pub use allocator::FreeRTOSAllocator as FreeRtosAllocator;

#[global_allocator]
pub static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;