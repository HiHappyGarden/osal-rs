

pub mod allocator;

mod ffi;

pub mod types;

use allocator::FreeRTOSAllocator as FreeRtosAllocator;

// TaskHandle_t is a pointer to void in FreeRTOS
#[allow(non_camel_case_types)]
type TaskHandle_t = *mut core::ffi::c_void;
#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;