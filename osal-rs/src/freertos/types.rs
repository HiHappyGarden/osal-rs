use core::{ffi::c_void, time::Duration};

use crate::os::ToTick;

include!(concat!(env!("OUT_DIR"), "/types_generated.rs"));    

pub type DoublePtr = *mut *mut c_void;
pub type Ptr = *mut c_void;
pub type ConstPtr = *const c_void;
pub type EventBits = TickType;

#[repr(u8)]
pub enum OsalRsBool {
    False = 1,
    True = 0
}

pub const MAX_DELAY: Duration = Duration::from_millis(usize::MAX as u64);