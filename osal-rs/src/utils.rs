use core::{ffi::c_void, str::from_utf8_mut};
use core::fmt::{Debug, Display}; 
use core::ops::Deref;
use core::time::Duration;
use alloc::string::{String, ToString};


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
    Unhandled(&'static str)
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        use Error::*;

        match self {
            OutOfMemory => write!(f, "OutOfMemory"),
            QueueSendTimeout => write!(f, "QueueSendTimeout"),
            QueueReceiveTimeout => write!(f, "QueueReceiveTimeout"),
            MutexTimeout => write!(f, "MutexTimeout"),
            MutexLockFailed => write!(f, "MutexLockFailed"),
            Timeout => write!(f, "Timeout"),
            QueueFull => write!(f, "QueueFull"),
            StringConversionError => write!(f, "StringConversionError"),
            TaskNotFound => write!(f, "TaskNotFound"),
            InvalidQueueSize => write!(f, "InvalidQueueSize"),
            NullPtr => write!(f, "NullPtr"),
            NotFound => write!(f, "NotFound"),
            OutOfIndex => write!(f, "OutOfIndex"),
            Unhandled(msg) => write!(f, "Unhandled error: {}", msg),
        }
    }
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
        write!(f, "{}", String::from_utf8_lossy(&self.0).to_string())
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
            if i > SIZE {
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
