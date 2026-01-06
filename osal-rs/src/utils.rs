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

//! Utility types and functions for OSAL-RS.
//!
//! This module contains common types, error definitions, and helper functions
//! used throughout the library.

use core::ffi::{CStr, c_char};
use core::{ffi::c_void, str::from_utf8_mut};
use core::fmt::{Debug, Display}; 
use core::ops::Deref;
use core::time::Duration;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use crate::os::Mutex;

/// Error types for OSAL-RS operations.
///
/// Represents all possible error conditions that can occur when using
/// the OSAL-RS library.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Queue, QueueFn};
/// use osal_rs::utils::Error;
/// 
/// match Queue::new(10, 32) {
///     Ok(queue) => { /* use queue */ },
///     Err(Error::OutOfMemory) => println!("Failed to allocate queue"),
///     Err(e) => println!("Other error: {:?}", e),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// Insufficient memory to complete operation
    OutOfMemory,
    /// Queue send operation timed out
    QueueSendTimeout,
    /// Queue receive operation timed out
    QueueReceiveTimeout,
    /// Mutex operation timed out
    MutexTimeout,
    /// Failed to acquire mutex lock
    MutexLockFailed,
    /// Generic timeout error
    Timeout,
    /// Queue is full and cannot accept more items
    QueueFull,
    /// String conversion failed
    StringConversionError,
    /// Thread/task not found
    TaskNotFound,
    /// Invalid queue size specified
    InvalidQueueSize,
    /// Null pointer encountered
    NullPtr,
    /// Requested item not found
    NotFound,
    /// Index out of bounds
    OutOfIndex,
    /// Invalid type for operation
    InvalidType,
    /// Unhandled error with description
    Unhandled(&'static str)
}

/// CPU register size enumeration.
///
/// Identifies whether the target CPU uses 32-bit or 64-bit registers.
/// This is used for platform-specific tick count overflow handling.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CpuRegisterSize {
    /// 64-bit CPU registers
    Bit64,
    /// 32-bit CPU registers
    Bit32
}

/// Boolean type compatible with RTOS return values.
///
/// Many RTOS functions return 0 for success and non-zero for failure.
/// This type provides a Rust-idiomatic way to work with such values.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Semaphore, SemaphoreFn};
/// use osal_rs::utils::OsalRsBool;
/// use core::time::Duration;
/// 
/// let sem = Semaphore::new(1, 1).unwrap();
/// 
/// match sem.wait(Duration::from_millis(100)) {
///     OsalRsBool::True => println!("Acquired semaphore"),
///     OsalRsBool::False => println!("Failed to acquire"),
/// }
/// 
/// // Can also convert to bool
/// if sem.signal().into() {
///     println!("Semaphore signaled");
/// }
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum OsalRsBool {
    /// Operation failed or condition is false
    False = 1,
    /// Operation succeeded or condition is true
    True = 0
}

/// Maximum delay constant for blocking operations.
///
/// When used as a timeout parameter, indicates the operation should
/// block indefinitely until it succeeds.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Mutex, MutexFn};
/// use osal_rs::utils::MAX_DELAY;
/// 
/// let mutex = Mutex::new(0);
/// let guard = mutex.lock();  // Blocks forever if needed
/// ```
pub const MAX_DELAY: Duration = Duration::from_millis(usize::MAX as u64);

/// Standard Result type for OSAL-RS operations.
///
/// Uses [`Error`] as the default error type.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Pointer to pointer type for C FFI.
pub type DoublePtr = *mut *mut c_void;

/// Mutable pointer type for C FFI.
pub type Ptr = *mut c_void;

/// Const pointer type for C FFI.
pub type ConstPtr = *const c_void;

/// Shortcut for Arc<Mutex<T>>
pub type ArcMux<T> = Arc<Mutex<T>>;

/// Determines the CPU register size at compile time.
///
/// This constant function checks the size of `usize` to determine whether
/// the target architecture uses 32-bit or 64-bit registers. This information
/// is used for platform-specific optimizations and overflow handling.
///
/// # Returns
///
/// * [`CpuRegisterSize::Bit64`] - For 64-bit architectures
/// * [`CpuRegisterSize::Bit32`] - For 32-bit architectures
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::{register_bit_size, CpuRegisterSize};
/// 
/// match register_bit_size() {
///     CpuRegisterSize::Bit64 => println!("Running on 64-bit platform"),
///     CpuRegisterSize::Bit32 => println!("Running on 32-bit platform"),
/// }
/// ```
pub const fn register_bit_size() -> CpuRegisterSize {
    if size_of::<usize>() == 8 {
        CpuRegisterSize::Bit64
    } else {
        CpuRegisterSize::Bit32
    }
}

/// Converts a C string pointer to a Rust String.
///
/// This macro safely converts a raw C string pointer (`*const c_char`) into
/// a Rust `String`. It handles UTF-8 conversion gracefully using lossy conversion.
///
/// # Safety
///
/// The pointer must be valid and point to a null-terminated C string.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::from_c_str;
/// use core::ffi::c_char;
/// 
/// extern "C" {
///     fn get_system_name() -> *const c_char;
/// }
/// 
/// let name = from_c_str!(get_system_name());
/// println!("System: {}", name);
/// ```
#[macro_export]
macro_rules! from_c_str {
    ($str:expr) => {
        unsafe {
            let c_str = core::ffi::CStr::from_ptr($str);
            alloc::string::String::from_utf8_lossy(c_str.to_bytes()).to_string()
        }
    };
}

/// Converts a Rust string to a CString with error handling.
///
/// This macro creates a `CString` from a Rust string reference, returning
/// a `Result` that can be used with the `?` operator. If the conversion fails
/// (e.g., due to interior null bytes), it returns an appropriate error.
///
/// # Returns
///
/// * `Ok(CString)` - On successful conversion
/// * `Err(Error::Unhandled)` - If the string contains interior null bytes
///
/// # Examples
///
/// ```ignore
/// use osal_rs::to_cstring;
/// use osal_rs::utils::Result;
/// 
/// fn pass_to_c_api(name: &str) -> Result<()> {
///     let c_name = to_cstring!(name)?;
///     // Use c_name.as_ptr() with C FFI
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! to_cstring {
    ($s:expr) => {
        alloc::ffi::CString::new($s.as_str())
            .map_err(|_| $crate::utils::Error::Unhandled("Failed to convert string to CString"))
    };
}

/// Converts a Rust string to a C string pointer.
///
/// This macro creates a `CString` from a Rust string and returns its raw pointer.
/// **Warning**: This macro panics if the conversion fails. Consider using
/// [`to_cstring!`] for safer error handling.
///
/// # Panics
///
/// Panics if the string contains interior null bytes.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::to_c_str;
/// 
/// extern "C" {
///     fn set_name(name: *const core::ffi::c_char);
/// }
/// 
/// let name = "FreeRTOS Task";
/// unsafe {
///     set_name(to_c_str!(name));
/// }
/// ```
#[macro_export]
macro_rules! to_c_str {
    ($s:expr) => {
        alloc::ffi::CString::new($s.as_ref() as &str).unwrap().as_ptr()
    };
}

/// Converts a string to a fixed-size byte array.
///
/// This macro creates a byte array of the specified size and fills it with
/// the bytes from the input string. If the string is shorter than the buffer,
/// the remaining bytes are filled with spaces. If the string is longer, it
/// is truncated to fit.
///
/// # Parameters
///
/// * `$str` - The source string to convert
/// * `$buff_name` - The identifier name for the created buffer variable
/// * `$buff_size` - The size of the byte array to create
///
/// # Examples
///
/// ```ignore
/// use osal_rs::from_str_to_array;
/// 
/// let task_name = "MainTask";
/// from_str_to_array!(task_name, name_buffer, 16);
/// // name_buffer is now [u8; 16] containing "MainTask        "
/// 
/// // Use with C FFI
/// extern "C" {
///     fn create_task(name: *const u8, len: usize);
/// }
/// 
/// unsafe {
///     create_task(name_buffer.as_ptr(), name_buffer.len());
/// }
/// ```
#[macro_export]
macro_rules! from_str_to_array {
    ($str:expr, $buff_name:ident, $buff_size:expr) => {
        let mut $buff_name = [b' '; $buff_size];
        let _bytes = $str.as_bytes();
        let _len = core::cmp::min(_bytes.len(), $buff_size);
        $buff_name[.._len].copy_from_slice(&_bytes[.._len]);
    };
}

/// Extracts a typed parameter from an optional boxed Any reference.
///
/// This macro is used in thread/task entry points to safely extract and
/// downcast parameters passed to the thread. It handles both the Option
/// unwrapping and the type downcast, returning appropriate errors if either
/// operation fails.
///
/// # Parameters
///
/// * `$param` - An `Option<Box<dyn Any>>` containing the parameter
/// * `$t` - The type to downcast the parameter to
///
/// # Returns
///
/// * A reference to the downcasted value of type `$t`
/// * `Err(Error::NullPtr)` - If the parameter is None
/// * `Err(Error::InvalidType)` - If the downcast fails
///
/// # Examples
///
/// ```ignore
/// use osal_rs::thread_extract_param;
/// use osal_rs::utils::Result;
/// use core::any::Any;
/// 
/// struct TaskConfig {
///     priority: u8,
///     stack_size: usize,
/// }
/// 
/// fn task_entry(param: Option<Box<dyn Any>>) -> Result<()> {
///     let config = thread_extract_param!(param, TaskConfig);
///     
///     println!("Priority: {}", config.priority);
///     println!("Stack: {}", config.stack_size);
///     
///     Ok(())
/// }
/// ```
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

/// Creates an Arc<Mutex<T>> from a value.
///
/// This is a convenience macro to reduce boilerplate when creating
/// thread-safe shared data structures.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::arcmux;
/// 
/// let shared_counter = arcmux!(0);
/// // Equivalent to: Arc::new(Mutex::new(0))
/// ```
#[macro_export]
macro_rules! arcmux {
    ($value:expr) => {
        alloc::sync::Arc::new($crate::os::MutexFn::new($value))
    };
}



/// Fixed-size byte array wrapper with string conversion utilities.
///
/// `Bytes` is a generic wrapper around a fixed-size byte array that provides
/// convenient methods for converting between strings and byte arrays. It's
/// particularly useful for interfacing with C APIs that expect fixed-size
/// character buffers, or for storing strings in embedded systems with
/// constrained memory.
///
/// # Type Parameters
///
/// * `SIZE` - The size of the internal byte array (default: 0)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::Bytes;
/// 
/// // Create an empty 32-byte buffer
/// let mut buffer = Bytes::<32>::new();
/// 
/// // Create a buffer from a string
/// let name = Bytes::<16>::new_by_str("TaskName");
/// println!("{}", name); // Prints "TaskName"
/// 
/// // Create from any type that implements ToString
/// let number = 42;
/// let num_bytes = Bytes::<8>::new_by_string(&number);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bytes<const SIZE: usize = 0> (pub [u8; SIZE]);

impl<const SIZE: usize> Deref for Bytes<SIZE> {
    type Target = [u8; SIZE];

    /// Dereferences to the underlying byte array.
    ///
    /// This allows `Bytes` to be used anywhere a `[u8; SIZE]` reference is expected.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes = Bytes::<8>::new_by_str("test");
    /// assert_eq!(bytes[0], b't');
    /// ```
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const SIZE: usize> Display for Bytes<SIZE> {
    /// Formats the byte array as a C-style null-terminated string.
    ///
    /// This implementation treats the byte array as a C string and converts it
    /// to a Rust string for display. If the conversion fails, it displays
    /// "Conversion error".
    ///
    /// # Safety
    ///
    /// This method assumes the byte array contains valid UTF-8 data and is
    /// null-terminated. Invalid data may result in the error message being displayed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes = Bytes::<16>::new_by_str("Hello");
    /// println!("{}", bytes); // Prints "Hello"
    /// ```
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
    /// Creates a new `Bytes` instance filled with zeros.
    ///
    /// This is a const function, allowing it to be used in const contexts
    /// and static variable declarations.
    ///
    /// # Returns
    ///
    /// A `Bytes` instance with all bytes set to 0.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// const BUFFER: Bytes<64> = Bytes::new();
    /// 
    /// let runtime_buffer = Bytes::<32>::new();
    /// assert_eq!(runtime_buffer[0], 0);
    /// ```
    pub const fn new() -> Self {
        Self( [0u8; SIZE] )
    }

    /// Creates a new `Bytes` instance from a string slice.
    ///
    /// Copies the bytes from the input string into the fixed-size array.
    /// If the string is shorter than `SIZE`, the remaining bytes are zero-filled.
    /// If the string is longer, it is truncated to fit.
    ///
    /// # Parameters
    ///
    /// * `str` - The source string to convert
    ///
    /// # Returns
    ///
    /// A `Bytes` instance containing the string data.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let short = Bytes::<16>::new_by_str("Hi");
    /// // Internal array: [b'H', b'i', 0, 0, 0, ...]
    /// 
    /// let exact = Bytes::<5>::new_by_str("Hello");
    /// // Internal array: [b'H', b'e', b'l', b'l', b'o']
    /// 
    /// let long = Bytes::<3>::new_by_str("Hello");
    /// // Internal array: [b'H', b'e', b'l'] (truncated)
    /// ```
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

    /// Creates a new `Bytes` instance from any type implementing `ToString`.
    ///
    /// This is a convenience wrapper around [`new_by_str`](Self::new_by_str)
    /// that first converts the input to a string.
    ///
    /// # Parameters
    ///
    /// * `str` - Any value that implements `ToString`
    ///
    /// # Returns
    ///
    /// A `Bytes` instance containing the string representation of the input.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// // From integer
    /// let num_bytes = Bytes::<8>::new_by_string(&42);
    /// 
    /// // From String
    /// let string = String::from("Task");
    /// let str_bytes = Bytes::<16>::new_by_string(&string);
    /// 
    /// // From custom type with ToString
    /// #[derive(Debug)]
    /// struct TaskId(u32);
    /// impl ToString for TaskId {
    ///     fn to_string(&self) -> String {
    ///         format!("Task-{}", self.0)
    ///     }
    /// }
    /// let task_bytes = Bytes::<16>::new_by_string(&TaskId(5));
    /// ```
    pub fn new_by_string(str: &impl ToString) -> Self {
        Self::new_by_str(&str.to_string())
    }

    /// Fills a mutable string slice with the contents of the byte array.
    ///
    /// Attempts to convert the internal byte array to a UTF-8 string and
    /// copies it into the destination string slice. Only copies up to the
    /// minimum of the source and destination lengths.
    ///
    /// # Parameters
    ///
    /// * `dest` - The destination string slice to fill
    ///
    /// # Panics
    ///
    /// Currently panics (todo!) if the byte array contains invalid UTF-8.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes = Bytes::<16>::new_by_str("Hello World");
    /// 
    /// let mut output = String::from("                "); // 16 spaces
    /// bytes.fill_str(unsafe { output.as_mut_str() });
    /// 
    /// assert_eq!(&output[..11], "Hello World");
    /// ```
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

/// Trait for types that can provide a string reference in a thread-safe manner.
///
/// This trait extends the basic string reference functionality with thread-safety
/// guarantees by requiring both `Sync` and `Send` bounds. It's useful for types
/// that need to provide string data across thread boundaries in a concurrent
/// environment.
///
/// # Thread Safety
///
/// Implementors must be both `Sync` (safe to share references across threads) and
/// `Send` (safe to transfer ownership across threads).
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::AsSyncStr;
/// 
/// struct ThreadSafeName {
///     name: &'static str,
/// }
/// 
/// impl AsSyncStr for ThreadSafeName {
///     fn as_str(&self) -> &str {
///         self.name
///     }
/// }
/// 
/// // Can be safely shared across threads
/// fn use_in_thread(item: &dyn AsSyncStr) {
///     println!("Name: {}", item.as_str());
/// }
/// ```
pub trait AsSyncStr : Sync + Send { 
    /// Returns a string slice reference.
    ///
    /// This method provides access to the underlying string data in a way
    /// that is safe to use across thread boundaries.
    ///
    /// # Returns
    ///
    /// A reference to a string slice with lifetime tied to `self`.
    fn as_str(&self) -> &str;
}

impl PartialEq for dyn AsSyncStr {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for dyn AsSyncStr {}

impl Debug for dyn AsSyncStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for dyn AsSyncStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

