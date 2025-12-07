

pub mod allocator;
pub mod config;
pub mod delay;
mod ffi;
pub mod system;
pub mod thread;
pub mod types;




use allocator::FreeRTOSAllocator as FreeRtosAllocator;
use types::TickType;
use ffi::xTaskGetTickCount;


#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;

pub fn get_tick_count() -> TickType {
    unsafe {
        xTaskGetTickCount()
    }
}