/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

use core::ffi::{CStr, c_char};
use core::{ffi::c_void, str::from_utf8_mut};
use core::fmt::{Debug, Display}; 
use core::ops::Deref;
use core::time::Duration;
use alloc::string::{String, ToString};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    OutOfMemory,
    QueueSendTimeout,
    QueueReceiveTimeout,
    MutexTimeout,
    MutexLockFailed,
    Timeout,
    QueueFull,
    StringConversionError,
    TaskNotFound,
    InvalidQueueSize,
    NullPtr,
    NotFound,
    OutOfIndex,
    InvalidType,
    Unhandled(&'static str)
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CpuRegisterSize {
    Bit64,
    Bit32
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum OsalRsBool {
    False = 1,
    True = 0
}

pub const MAX_DELAY: Duration = Duration::from_millis(usize::MAX as u64);

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub type DoublePtr = *mut *mut c_void;
pub type Ptr = *mut c_void;
pub type ConstPtr = *const c_void;


pub const fn register_bit_size() -> CpuRegisterSize {
    if size_of::<usize>() == 8 {
        CpuRegisterSize::Bit64
    } else {
        CpuRegisterSize::Bit32
    }
}

#[macro_export]
macro_rules! from_c_str {
    ($str:expr) => {
        unsafe {
            let c_str = core::ffi::CStr::from_ptr($str);
            alloc::string::String::from_utf8_lossy(c_str.to_bytes()).to_string()
        }
    };
}

#[macro_export]
macro_rules! to_cstring {
    ($s:expr) => {
        alloc::ffi::CString::new($s.as_str())
            .map_err(|_| $crate::utils::Error::Unhandled("Failed to convert string to CString"))
    };
}

#[macro_export]
macro_rules! to_c_str {
    ($s:expr) => {
        alloc::ffi::CString::new($s.as_ref() as &str).unwrap().as_ptr()
    };
}

#[macro_export]
macro_rules! from_str_to_array {
    ($str:expr, $buff_name:ident, $buff_size:expr) => {
        let mut $buff_name = [b' '; $buff_size];
        let _bytes = $str.as_bytes();
        let _len = core::cmp::min(_bytes.len(), $buff_size);
        $buff_name[.._len].copy_from_slice(&_bytes[.._len]);
    };
}

#[macro_export]
macro_rules! thread_extract_param {
    ($param:expr, $t:ty) => {
        match $param.as_ref() {
            Some(p) => {
                match p.downcast_ref::<$t>() {
                    Some(value) => value,
                    None => return Err($crate::utils::Error::InvalidType),
                }
            }
            None => return Err($crate::utils::Error::NullPtr),
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bytes<const SIZE: usize = 0> (pub [u8; SIZE]);

impl<const SIZE: usize> Deref for Bytes<SIZE> {
    type Target = [u8; SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const SIZE: usize> Display for Bytes<SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let str = unsafe {
            CStr::from_ptr(self.0.as_ptr() as *const c_char)
            .to_str()
            .unwrap_or("Conversion error")
        };
        
        write!(f, "{}", str.to_string())
    }
}


impl<const SIZE: usize> Bytes<SIZE> {
    pub const fn new() -> Self {
        Self( [0u8; SIZE] )
    }

    pub fn new_by_str(str: &str) -> Self {

        let mut array = [0u8; SIZE];
        
        let mut i = 0usize ;
        for byte in str.as_bytes() {
            if i > SIZE - 1{
                break;
            }
            array[i] = *byte;
            i += 1;
        }  

        Self( array )
    }

    pub fn new_by_string(str: &impl ToString) -> Self {
        Self::new_by_str(&str.to_string())
    }

    pub fn fill_str(&mut self, dest: &mut str) {
        match from_utf8_mut(&mut self.0) {
            Ok(str) => {
                let len = core::cmp::min(str.len(), dest.len());
                unsafe {
                    dest.as_bytes_mut()[..len].copy_from_slice(&str.as_bytes()[..len]);
                }
            }
            Err(_) => todo!(),
        }
    }
}
