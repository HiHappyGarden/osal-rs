// Central FFI bindings module - included only once
pub(crate) mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clashing_extern_declarations)]
    include!(concat!(env!("OUT_DIR"), "/freertos_bindings.rs"));

pub const pdFALSE: BaseType_t = 0;
pub const pdTRUE: BaseType_t = 1;
pub const pdPASS: BaseType_t = pdTRUE;
pub const pdFAIL: BaseType_t = pdFALSE;
pub const errQUEUE_EMPTY: BaseType_t = 0;
pub const errQUEUE_FULL: BaseType_t = 0;


/// Converts a time in milliseconds to a time in ticks.
#[macro_export]
macro_rules! pdMS_TO_TICKS {
    ($xTimeInMs:expr) => {
        (((($xTimeInMs as u64) * (unsafe { $crate::freertos::ffi::FREERTOS_TICK_RATE_HZ } as u64)) / 1000u64) as $crate::freertos::ffi::TickType_t)
    };
}

/// Converts a time in ticks to a time in milliseconds.
#[macro_export]
macro_rules! pdTICKS_TO_MS {
    ($xTimeInTicks:expr) => {
        (((($xTimeInTicks as u64) * 1000u64) / (unsafe { $crate::freertos::ffi::FREERTOS_TICK_RATE_HZ } as u64)) as $crate::freertos::ffi::TickType_t)
    };
}

/// FreeRTOS errno codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FreeRtosErrno {
    /// No errors
    None = 0,
    /// No such file or directory
    NoEnt = 2,
    /// Interrupted system call
    Intr = 4,
    /// I/O error
    Io = 5,
    /// No such device or address
    Nxio = 6,
    /// Bad file number
    Badf = 9,
    /// No more processes / Operation would block
    Again = 11,
    /// Not enough memory
    NoMem = 12,
    /// Permission denied
    Acces = 13,
    /// Bad address
    Fault = 14,
    /// Mount device busy
    Busy = 16,
    /// File exists
    Exist = 17,
    /// Cross-device link
    Xdev = 18,
    /// No such device
    NoDev = 19,
    /// Not a directory
    NotDir = 20,
    /// Is a directory
    IsDir = 21,
    /// Invalid argument
    Inval = 22,
    /// No space left on device
    NoSpc = 28,
    /// Illegal seek
    Spipe = 29,
    /// Read only file system
    Rofs = 30,
    /// Protocol driver not attached
    Unatch = 42,
    /// Invalid exchange
    Bade = 50,
    /// Inappropriate file type or format
    Ftype = 79,
    /// No more files
    NmFile = 89,
    /// Directory not empty
    NotEmpty = 90,
    /// File or path name too long
    NameTooLong = 91,
    /// Operation not supported on transport endpoint
    OpNotSupp = 95,
    /// Address family not supported by protocol
    AfNoSupport = 97,
    /// No buffer space available
    NoBufs = 105,
    /// Protocol not available
    NoProtoOpt = 109,
    /// Address already in use
    AddrInUse = 112,
    /// Connection timed out
    TimedOut = 116,
    /// Connection already in progress
    InProgress = 119,
    /// Socket already connected
    Already = 120,
    /// Address not available
    AddrNotAvail = 125,
    /// Socket is already connected
    IsConn = 127,
    /// Socket is not connected
    NotConn = 128,
    /// No medium inserted
    NoMedium = 135,
    /// An invalid UTF-16 sequence was encountered
    Ilseq = 138,
    /// Operation canceled
    Canceled = 140,
}

impl FreeRtosErrno {
    /// Converts from a raw errno value
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            2 => Some(Self::NoEnt),
            4 => Some(Self::Intr),
            5 => Some(Self::Io),
            6 => Some(Self::Nxio),
            9 => Some(Self::Badf),
            11 => Some(Self::Again),
            12 => Some(Self::NoMem),
            13 => Some(Self::Acces),
            14 => Some(Self::Fault),
            16 => Some(Self::Busy),
            17 => Some(Self::Exist),
            18 => Some(Self::Xdev),
            19 => Some(Self::NoDev),
            20 => Some(Self::NotDir),
            21 => Some(Self::IsDir),
            22 => Some(Self::Inval),
            28 => Some(Self::NoSpc),
            29 => Some(Self::Spipe),
            30 => Some(Self::Rofs),
            42 => Some(Self::Unatch),
            50 => Some(Self::Bade),
            79 => Some(Self::Ftype),
            89 => Some(Self::NmFile),
            90 => Some(Self::NotEmpty),
            91 => Some(Self::NameTooLong),
            95 => Some(Self::OpNotSupp),
            97 => Some(Self::AfNoSupport),
            105 => Some(Self::NoBufs),
            109 => Some(Self::NoProtoOpt),
            112 => Some(Self::AddrInUse),
            116 => Some(Self::TimedOut),
            119 => Some(Self::InProgress),
            120 => Some(Self::Already),
            125 => Some(Self::AddrNotAvail),
            127 => Some(Self::IsConn),
            128 => Some(Self::NotConn),
            135 => Some(Self::NoMedium),
            138 => Some(Self::Ilseq),
            140 => Some(Self::Canceled),
            _ => None,
        }
    }
}

}

pub mod event;

mod free_rtos_allocator;

pub mod mutex;

pub mod queue;

pub mod semaphore;

pub mod stream_buffer;

pub mod system;

pub mod thread;

pub mod time;

pub mod timer;

