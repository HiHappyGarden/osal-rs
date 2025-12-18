use core::{fmt::Debug, time::Duration};


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

pub const fn register_bit_size() -> CpuRegisterSize {
    if size_of::<usize>() == 8 {
        CpuRegisterSize::Bit64
    } else {
        CpuRegisterSize::Bit32
    }
}
