

pub mod allocator;
pub mod config;
pub mod duration;
pub mod event_group;
mod ffi;
pub mod system;
pub mod thread;
pub mod tick;
pub mod types;

use core::ffi::{CStr, c_char};
use alloc::string::{String, ToString};
use crate::utils::{Result, Error::Unhandled}; 
use allocator::FreeRTOSAllocator as FreeRtosAllocator;

#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;

pub(crate) fn ptr_char_to_string(str: *const c_char) -> String {
    unsafe {
        let c_str = CStr::from_ptr(str);
        String::from_utf8_lossy(c_str.to_bytes()).to_string()
    }
}

pub(crate) fn string_to_ptr_char(s: &str) -> Result<*const c_char>  {
    let c_string = alloc::ffi::CString::new(s).map_err(|_| Unhandled("Failed to convert string to CString"))?;
    Ok(c_string.into_raw() as *const c_char)
}