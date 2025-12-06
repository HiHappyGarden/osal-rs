

pub mod allocator;
mod ffi;
pub mod types;
pub mod config;

use allocator::FreeRTOSAllocator as FreeRtosAllocator;

#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;