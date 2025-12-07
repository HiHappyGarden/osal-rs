

pub mod allocator;
pub mod config;
pub mod delay;
mod ffi;
pub mod system;
pub mod thread;
pub mod types;

use core::ffi::{CStr, c_char};

use alloc::string::{String, ToString};
use allocator::FreeRTOSAllocator as FreeRtosAllocator;

#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;

pub(crate) fn ptr_char_to_string(str: *const c_char) -> String {
    unsafe {
        let c_str = CStr::from_ptr(str);
        String::from_utf8_lossy(c_str.to_bytes()).to_string()
    }
}