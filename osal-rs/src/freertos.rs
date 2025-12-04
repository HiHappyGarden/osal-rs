

pub mod free_rtos_allocator;

mod ffi;

use free_rtos_allocator::FreeRTOSAllocator as FreeRtosAllocator;

#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;
