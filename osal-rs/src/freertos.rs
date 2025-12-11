

pub mod allocator;
pub mod config;
pub mod duration;
pub mod event_group;
mod ffi;
pub mod system;
pub mod thread;
pub mod types;

use allocator::FreeRTOSAllocator as FreeRtosAllocator;

#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;