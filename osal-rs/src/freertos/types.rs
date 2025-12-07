use core::ffi::c_void;

include!(concat!(env!("OUT_DIR"), "/types_generated.rs"));    

pub type DoublePtr = *mut *mut c_void;
pub type Ptr = *mut c_void;
pub type ConstPtr = *const c_void;