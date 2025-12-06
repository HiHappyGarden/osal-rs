mod ffi {
    include!(concat!(env!("OUT_DIR"), "/types_generated.rs"));    
}

use crate::freertos::types::ffi::{BaseType as BaseTypeFfi, TickType as TickTypeFfi, UBaseType as UBaseTypeFfi};

pub type TickType = TickTypeFfi;
pub type UBaseType = UBaseTypeFfi;
pub type BaseType = BaseTypeFfi;