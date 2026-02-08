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

use core::cell::UnsafeCell;
use core::ffi::{CStr, c_char, c_void};
use core::str::{from_utf8_mut, FromStr};
use core::fmt::{Debug, Display}; 
use core::ops::{Deref, DerefMut};
use core::time::Duration;

use alloc::ffi::CString;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

#[cfg(not(feature = "serde"))]
use crate::os::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use osal_rs_serde::{Deserialize, Serialize};

use crate::os::{AsSyncStr, Mutex};

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
    /// No data available
    Empty,
    /// Write error occurred
    WriteError(&'static str),
    /// Read error occurred
    ReadError(&'static str),
    /// Return error with code
    ReturnWithCode(i32),
    /// Unhandled error with description
    Unhandled(&'static str),
    /// Unhandled error with description owned
    UnhandledOwned(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Error::*;

        match self {
            OutOfMemory => write!(f, "Out of memory"),
            QueueSendTimeout => write!(f, "Queue send timeout"),
            QueueReceiveTimeout => write!(f, "Queue receive timeout"),
            MutexTimeout => write!(f, "Mutex timeout"),
            MutexLockFailed => write!(f, "Mutex lock failed"),
            Timeout => write!(f, "Operation timeout"),
            QueueFull => write!(f, "Queue full"),
            StringConversionError => write!(f, "String conversion error"),
            TaskNotFound => write!(f, "Task not found"),
            InvalidQueueSize => write!(f, "Invalid queue size"),
            NullPtr => write!(f, "Null pointer encountered"),
            NotFound => write!(f, "Item not found"),
            OutOfIndex => write!(f, "Index out of bounds"),
            InvalidType => write!(f, "Invalid type for operation"),
            Empty => write!(f, "No data available"),
            WriteError(desc) => write!(f, "Write error occurred: {}", desc),
            ReadError(desc) => write!(f, "Read error occurred: {}", desc),
            ReturnWithCode(code) => write!(f, "Return with code: {}", code),
            Unhandled(desc) => write!(f, "Unhandled error: {}", desc),
            UnhandledOwned(desc) => write!(f, "Unhandled error owned: {}", desc),
        }
    }
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
        alloc::sync::Arc::new($crate::os::Mutex::new($value))
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

#[cfg(feature = "serde")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bytes<const SIZE: usize> (pub [u8; SIZE]);

#[cfg(not(feature = "serde"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bytes<const SIZE: usize> (pub [u8; SIZE]);

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

impl<const SIZE: usize> DerefMut for Bytes<SIZE> {
    /// Provides mutable access to the underlying byte array.
    ///
    /// This allows `Bytes` to be mutably dereferenced, enabling direct modification
    /// of the internal byte array through the `DerefMut` trait.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let mut bytes = Bytes::<8>::new();
    /// bytes[0] = b'H';
    /// bytes[1] = b'i';
    /// assert_eq!(bytes[0], b'H');
    /// ```
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

impl<const SIZE: usize> FromStr for Bytes<{SIZE}> {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(Self::new_by_str(s))
    }
}

impl<const SIZE: usize> AsSyncStr for Bytes<SIZE> {
    /// Returns a string slice reference.
    ///
    /// This method provides access to the underlying string data in a way
    /// that is safe to use across thread boundaries.
    ///
    /// # Returns
    ///
    /// A reference to a string slice with lifetime tied to `self`.
    fn as_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.as_ptr() as *const c_char)
            .to_str()
            .unwrap_or("Conversion error")
        }
    }
}

/// Serialization implementation for `Bytes<SIZE>` when the `serde` feature is enabled.
///
/// This implementation provides serialization by directly serializing each byte
/// in the array using the osal-rs-serde serialization framework.
#[cfg(feature = "serde")]
impl<const SIZE: usize> Serialize for Bytes<SIZE> {
    /// Serializes the `Bytes` instance using the given serializer.
    ///
    /// # Parameters
    ///
    /// * `serializer` - The serializer to use
    ///
    /// # Returns
    ///
    /// * `Ok(())` - On successful serialization
    /// * `Err(S::Error)` - If serialization fails
    fn serialize<S: osal_rs_serde::Serializer>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> {
        for &byte in self.0.iter() {
            serializer.serialize_u8(name, byte)?;
        }
        Ok(())
    }
}

/// Deserialization implementation for `Bytes<SIZE>` when the `serde` feature is enabled.
///
/// This implementation provides deserialization by reading bytes from the deserializer
/// into a fixed-size array using the osal-rs-serde deserialization framework.
#[cfg(feature = "serde")]
impl<const SIZE: usize> Deserialize for Bytes<SIZE> {
    /// Deserializes a `Bytes` instance using the given deserializer.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - The deserializer to use
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes<SIZE>)` - A new `Bytes` instance with deserialized data
    /// * `Err(D::Error)` - If deserialization fails
    fn deserialize<D: osal_rs_serde::Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        let mut array = [0u8; SIZE];
        for i in 0..SIZE {
            array[i] = deserializer.deserialize_u8(name)?;
        }
        Ok(Self(array))
    }
}

/// Serialization implementation for `Bytes<SIZE>` when the `serde` feature is disabled.
///
/// This implementation provides basic serialization by directly returning a reference
/// to the underlying byte array. It's used when the library is compiled without the
/// `serde` feature, providing a lightweight alternative serialization mechanism.
#[cfg(not(feature = "serde"))]
impl<const SIZE: usize> Serialize for Bytes<SIZE> {
    /// Converts the `Bytes` instance to a byte slice.
    ///
    /// # Returns
    ///
    /// A reference to the internal byte array.
    fn to_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Deserialization implementation for `Bytes<SIZE>` when the `serde` feature is disabled.
///
/// This implementation provides basic deserialization by copying bytes from a slice
/// into a fixed-size array. If the source slice is shorter than `SIZE`, the remaining
/// bytes are zero-filled. If longer, it's truncated to fit.
#[cfg(not(feature = "serde"))]
impl<const SIZE: usize> Deserialize for Bytes<SIZE> {
    /// Creates a `Bytes` instance from a byte slice.
    ///
    /// # Parameters
    ///
    /// * `bytes` - The source byte slice to deserialize from
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes<SIZE>)` - A new `Bytes` instance with data copied from the slice
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// use osal_rs::os::Deserialize;
    /// 
    /// let data = b"Hello";
    /// let bytes = Bytes::<16>::from_bytes(data).unwrap();
    /// // Result: [b'H', b'e', b'l', b'l', b'o', 0, 0, 0, ...]
    /// ```
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut array = [0u8; SIZE];
        let len = core::cmp::min(bytes.len(), SIZE);
        array[..len].copy_from_slice(&bytes[..len]);
        Ok(Self( array ))
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

    /// Creates a new `Bytes` instance from a C string pointer.
    ///
    /// Safely converts a null-terminated C string pointer into a `Bytes` instance.
    /// If the pointer is null, returns a zero-initialized `Bytes`. The function
    /// copies bytes from the C string into the fixed-size array, truncating if
    /// the source is longer than `SIZE`.
    ///
    /// # Parameters
    ///
    /// * `str` - A pointer to a null-terminated C string (`*const c_char`)
    ///
    /// # Safety
    ///
    /// While this function is not marked unsafe, it internally uses `unsafe` code
    /// to dereference the pointer. The caller must ensure that:
    /// - If not null, the pointer points to a valid null-terminated C string
    /// - The memory the pointer references remains valid for the duration of the call
    ///
    /// # Returns
    ///
    /// A `Bytes` instance containing the C string data, or zero-initialized if the pointer is null.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// use core::ffi::c_char;
    /// use alloc::ffi::CString;
    ///
    /// // From a CString
    /// let c_string = CString::new("Hello").unwrap();
    /// let bytes = Bytes::<16>::new_by_ptr(c_string.as_ptr());
    ///
    /// // From a null pointer
    /// let null_bytes = Bytes::<16>::new_by_ptr(core::ptr::null());
    /// // Returns zero-initialized Bytes
    ///
    /// // Truncation example
    /// let long_string = CString::new("This is a very long string").unwrap();
    /// let short_bytes = Bytes::<8>::new_by_ptr(long_string.as_ptr());
    /// // Only first 8 bytes are copied
    /// ```
    pub fn new_by_ptr(str: *const c_char) -> Self {
        if str.is_null() {
            return Self::new();
        }

        let mut array = [0u8; SIZE];

        let mut i = 0usize ;
        for byte in unsafe { CStr::from_ptr(str) }.to_bytes() {
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
    pub fn new_by_as_sync_str(str: &impl ToString) -> Self {
        Self::new_by_str(&str.to_string())
    }

    pub fn new_by_bytes(bytes: &[u8]) -> Self {
        let mut array = [0u8; SIZE];
        let len = core::cmp::min(bytes.len(), SIZE);
        array[..len].copy_from_slice(&bytes[..len]);
        Self( array )
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
    /// # Returns
    ///
    /// `Ok(())` if the operation succeeds, or `Err(Error::StringConversionError)` if the byte array cannot be converted to a valid UTF-8 string.
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
    pub fn fill_str(&mut self, dest: &mut str) -> Result<()>{
        match from_utf8_mut(&mut self.0) {
            Ok(str) => {
                let len = core::cmp::min(str.len(), dest.len());
                unsafe {
                    dest.as_bytes_mut()[..len].copy_from_slice(&str.as_bytes()[..len]);
                }
                Ok(())
            }
            Err(_) => Err(Error::StringConversionError),
        }
    }

    /// Converts the byte array to a C string reference.
    ///
    /// Creates a `CStr` reference from the internal byte array, treating it as
    /// a null-terminated C string. This is useful for passing strings to C FFI
    /// functions that expect `*const c_char` or `&CStr`.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it assumes:
    /// - The byte array contains valid UTF-8 data
    /// - The byte array is null-terminated
    /// - There are no interior null bytes before the terminating null
    ///
    /// Violating these assumptions may lead to undefined behavior.
    ///
    /// # Returns
    ///
    /// A reference to a `CStr` with lifetime tied to `self`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes = Bytes::<16>::new_by_str("Hello");
    /// let c_str = bytes.as_c_str();
    /// 
    /// extern "C" {
    ///     fn print_string(s: *const core::ffi::c_char);
    /// }
    /// 
    /// unsafe {
    ///     print_string(c_str.as_ptr());
    /// }
    /// ```
    pub fn as_c_str(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(self.0.as_ptr() as *const c_char)
        }
    }

    /// Converts the byte array to an owned C string.
    ///
    /// Creates a new `CString` by copying the contents of the internal byte array.
    /// Unlike [`as_c_str`](Self::as_c_str), this method allocates heap memory and
    /// returns an owned string that can outlive the original `Bytes` instance.
    ///
    /// # Safety
    ///
    /// This method uses `from_vec_unchecked` which assumes the byte array
    /// does not contain any interior null bytes. If this assumption is violated,
    /// the resulting `CString` will be invalid.
    ///
    /// # Returns
    ///
    /// An owned `CString` containing a copy of the byte array data.
    ///
    /// # Memory Allocation
    ///
    /// This method allocates on the heap. In memory-constrained embedded systems,
    /// prefer [`as_c_str`](Self::as_c_str) when possible.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// fn process_name(bytes: &Bytes<16>) -> alloc::ffi::CString {
    ///     // Create an owned copy that can be returned
    ///     bytes.as_cstring()
    /// }
    /// 
    /// let name = Bytes::<16>::new_by_str("Task");
    /// let owned = process_name(&name);
    /// // 'name' can be dropped, 'owned' still valid
    /// ```
    pub fn as_cstring(&self) -> CString {
        unsafe {
            CString::from_vec_unchecked(self.0.to_vec())
        }
    }

    /// Appends a string slice to the existing content in the `Bytes` buffer.
    ///
    /// This method finds the current end of the content (first null byte) and appends
    /// the provided string starting from that position. If the buffer is already full
    /// or if the appended content would exceed the buffer size, the content is truncated
    /// to fit within the `SIZE` limit.
    ///
    /// # Parameters
    ///
    /// * `str` - The string slice to append
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// bytes.append_str(" World");
    /// assert_eq!(bytes.as_str(), "Hello World");
    ///
    /// // Truncation when exceeding buffer size
    /// let mut small_bytes = Bytes::<8>::new_by_str("Hi");
    /// small_bytes.append_str(" there friend");
    /// assert_eq!(small_bytes.as_str(), "Hi ther");
    /// ```
    pub fn append_str(&mut self, str: &str) {
        let current_len = self.0.iter().position(|&b| b == 0).unwrap_or(SIZE);
        let mut i = current_len;
        for byte in str.as_bytes() {
            if i > SIZE - 1{
                break;
            }
            self.0[i] = *byte;
            i += 1;
        }
    }

    /// Appends content from any type implementing `AsSyncStr` to the buffer.
    ///
    /// This method accepts any type that implements the `AsSyncStr` trait, converts
    /// it to a string slice, and appends it to the existing content. If the buffer
    /// is already full or if the appended content would exceed the buffer size,
    /// the content is truncated to fit within the `SIZE` limit.
    ///
    /// # Parameters
    ///
    /// * `c_str` - A reference to any type implementing `AsSyncStr`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// let other_bytes = Bytes::<8>::new_by_str(" World");
    /// bytes.append_as_sync_str(&other_bytes);
    /// assert_eq!(bytes.as_str(), "Hello World");
    /// ```
    pub fn append_as_sync_str(&mut self, c_str: & impl AsSyncStr) {
        let current_len = self.0.iter().position(|&b| b == 0).unwrap_or(SIZE);
        let mut i = current_len;
        for byte in c_str.as_str().as_bytes() {
            if i > SIZE - 1{
                break;
            }
            self.0[i] = *byte;
            i += 1;
        }
    }

    /// Appends raw bytes to the existing content in the `Bytes` buffer.
    ///
    /// This method finds the current end of the content (first null byte) and appends
    /// the provided byte slice starting from that position. If the buffer is already
    /// full or if the appended content would exceed the buffer size, the content is
    /// truncated to fit within the `SIZE` limit.
    ///
    /// # Parameters
    ///
    /// * `bytes` - The byte slice to append
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// bytes.append_bytes(b" World");
    /// assert_eq!(bytes.as_str(), "Hello World");
    ///
    /// // Appending arbitrary bytes
    /// let mut data = Bytes::<16>::new_by_str("Data: ");
    /// data.append_bytes(&[0x41, 0x42, 0x43]);
    /// assert_eq!(data.as_str(), "Data: ABC");
    /// ```
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        let current_len = self.0.iter().position(|&b| b == 0).unwrap_or(SIZE);
        let mut i = current_len;
        for byte in bytes {
            if i > SIZE - 1{
                break;
            }
            self.0[i] = *byte;
            i += 1;
        }
    }

    /// Appends the content of another `Bytes` instance to this buffer.
    ///
    /// This method allows appending content from a `Bytes` instance of a different
    /// size (specified by the generic parameter `OHTER_SIZE`). The method finds the
    /// current end of the content (first null byte) and appends the content from the
    /// other `Bytes` instance. If the buffer is already full or if the appended content
    /// would exceed the buffer size, the content is truncated to fit within the `SIZE` limit.
    ///
    /// # Type Parameters
    ///
    /// * `OHTER_SIZE` - The size of the source `Bytes` buffer (can be different from `SIZE`)
    ///
    /// # Parameters
    ///
    /// * `other` - A reference to the `Bytes` instance to append
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// let other = Bytes::<8>::new_by_str(" World");
    /// bytes.append(&other);
    /// assert_eq!(bytes.as_str(), "Hello World");
    ///
    /// // Appending from a larger buffer
    /// let mut small = Bytes::<8>::new_by_str("Hi");
    /// let large = Bytes::<32>::new_by_str(" there friend");
    /// small.append(&large);
    /// assert_eq!(small.as_str(), "Hi ther");
    /// ```
    pub fn append<const OHTER_SIZE: usize>(&mut self, other: &Bytes<OHTER_SIZE>) {
        let current_len = self.0.iter().position(|&b| b == 0).unwrap_or(SIZE);
        let mut i = current_len;
        for &byte in other.0.iter() {
            if i > SIZE - 1{
                break;
            }
            self.0[i] = byte;
            i += 1;
        }
    }
}

/// Converts a byte slice to a hexadecimal string representation.
///
/// Each byte is converted to its two-character hexadecimal representation
/// in lowercase. This function allocates a new `String` on the heap.
///
/// # Parameters
///
/// * `bytes` - The byte slice to convert
///
/// # Returns
///
/// A `String` containing the hexadecimal representation of the bytes.
/// Each byte is represented by exactly 2 hex characters (lowercase).
///
/// # Memory Allocation
///
/// This function allocates heap memory. In memory-constrained environments,
/// consider using [`bytes_to_hex_into_slice`] instead.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::bytes_to_hex;
/// 
/// let data = &[0x01, 0x23, 0xAB, 0xFF];
/// let hex = bytes_to_hex(data);
/// assert_eq!(hex, "0123abff");
/// 
/// let empty = bytes_to_hex(&[]);
/// assert_eq!(empty, "");
/// ```
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter()
         .map(|b| format!("{:02x}", b))
         .collect()
}

/// Converts a byte slice to hexadecimal representation into a pre-allocated buffer.
///
/// This is a zero-allocation version of [`bytes_to_hex`] that writes the
/// hexadecimal representation directly into a provided output buffer.
/// Suitable for embedded systems and real-time applications.
///
/// # Parameters
///
/// * `bytes` - The source byte slice to convert
/// * `output` - The destination buffer to write hex characters into
///
/// # Returns
///
/// The number of bytes written to the output buffer (always `bytes.len() * 2`).
///
/// # Panics
///
/// Panics if `output.len() < bytes.len() * 2`. The output buffer must be
/// at least twice the size of the input to hold the hex representation.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::bytes_to_hex_into_slice;
/// 
/// let data = &[0x01, 0xAB, 0xFF];
/// let mut buffer = [0u8; 6];
/// 
/// let written = bytes_to_hex_into_slice(data, &mut buffer);
/// assert_eq!(written, 6);
/// assert_eq!(&buffer, b"01abff");
/// 
/// // Will panic - buffer too small
/// // let mut small = [0u8; 4];
/// // bytes_to_hex_into_slice(data, &mut small);
/// ```
pub fn bytes_to_hex_into_slice(bytes: &[u8], output: &mut [u8]) -> usize {
    assert!(output.len() >= bytes.len() * 2, "Buffer too small for hex conversion");
    let mut i = 0;
    for &b in bytes {
        let hex = format!("{:02x}", b);
        output[i..i+2].copy_from_slice(hex.as_bytes());
        i += 2;
    }
    i 
}

/// Converts a hexadecimal string to a vector of bytes.
///
/// Parses a string of hexadecimal digits (case-insensitive) and converts
/// them to their binary representation. Each pair of hex digits becomes
/// one byte in the output.
///
/// # Parameters
///
/// * `hex` - A string slice containing hexadecimal digits (0-9, a-f, A-F)
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - A vector containing the decoded bytes
/// * `Err(Error::StringConversionError)` - If the string has odd length or contains invalid hex digits
///
/// # Memory Allocation
///
/// This function allocates a `Vec` on the heap. For no-alloc environments,
/// use [`hex_to_bytes_into_slice`] instead.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::hex_to_bytes;
/// 
/// // Lowercase hex
/// let bytes = hex_to_bytes("0123abff").unwrap();
/// assert_eq!(bytes, vec![0x01, 0x23, 0xAB, 0xFF]);
/// 
/// // Uppercase hex
/// let bytes2 = hex_to_bytes("ABCD").unwrap();
/// assert_eq!(bytes2, vec![0xAB, 0xCD]);
/// 
/// // Odd length - error
/// assert!(hex_to_bytes("ABC").is_err());
/// 
/// // Invalid character - error
/// assert!(hex_to_bytes("0G").is_err());
/// ```
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(Error::StringConversionError);
    }

    let bytes_result: Result<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| Error::StringConversionError)
        })
        .collect();

    bytes_result
}

/// Converts a hexadecimal string to bytes into a pre-allocated buffer.
///
/// This is a zero-allocation version of [`hex_to_bytes`] that writes decoded
/// bytes directly into a provided output buffer. Suitable for embedded systems
/// and real-time applications where heap allocation is not desired.
///
/// # Parameters
///
/// * `hex` - A string slice containing hexadecimal digits (0-9, a-f, A-F)
/// * `output` - The destination buffer to write decoded bytes into
///
/// # Returns
///
/// * `Ok(usize)` - The number of bytes written to the output buffer (`hex.len() / 2`)
/// * `Err(Error::StringConversionError)` - If:
///   - The hex string has odd length
///   - The output buffer is too small (`output.len() < hex.len() / 2`)
///   - The hex string contains invalid characters
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::hex_to_bytes_into_slice;
/// 
/// let mut buffer = [0u8; 4];
/// let written = hex_to_bytes_into_slice("0123abff", &mut buffer).unwrap();
/// assert_eq!(written, 4);
/// assert_eq!(buffer, [0x01, 0x23, 0xAB, 0xFF]);
/// 
/// // Buffer too small
/// let mut small = [0u8; 2];
/// assert!(hex_to_bytes_into_slice("0123abff", &mut small).is_err());
/// 
/// // Odd length string
/// assert!(hex_to_bytes_into_slice("ABC", &mut buffer).is_err());
/// ```
pub fn hex_to_bytes_into_slice(hex: &str, output: &mut [u8]) -> Result<usize> {
    if hex.len() % 2 != 0 || output.len() < hex.len() / 2 {
        return Err(Error::StringConversionError);
    }

    for i in 0..(hex.len() / 2) {
        output[i] = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
            .map_err(|_| Error::StringConversionError)?;
    }

    Ok(hex.len() / 2)
}

/// Thread-safe wrapper for `UnsafeCell` usable in `static` contexts.
///
/// `SyncUnsafeCell<T>` is a thin wrapper around `UnsafeCell<T>` that manually
/// implements the `Sync` and `Send` traits, allowing its use in `static` variables.
/// This is necessary in Rust 2024+ where `static mut` is no longer allowed.
///
/// # Safety
///
/// The manual implementation of `Sync` and `Send` is **unsafe** because the compiler
/// cannot verify that concurrent access is safe. It is the programmer's responsibility
/// to ensure that:
///
/// 1. In **single-threaded** environments (e.g., embedded bare-metal), there are no
///    synchronization issues since only one thread of execution exists.
///
/// 2. In **multi-threaded** environments, access to `SyncUnsafeCell` must be
///    externally protected via mutexes, critical sections, or other synchronization
///    primitives.
///
/// 3. No **data race** conditions occur during data access.
///
/// # Typical Usage
///
/// This structure is designed to replace `static mut` in embedded scenarios
/// where global mutability is necessary (e.g., hardware registers, shared buffers).
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::SyncUnsafeCell;
///
/// // Global mutable variable in Rust 2024+
/// static COUNTER: SyncUnsafeCell<u32> = SyncUnsafeCell::new(0);
///
/// fn increment_counter() {
///     unsafe {
///         let counter = &mut *COUNTER.get();
///         *counter += 1;
///     }
/// }
/// ```
///
/// # Alternatives
///
/// For non-embedded code or when real synchronization is needed:
/// - Use `Mutex<T>` or `RwLock<T>` for thread-safe protection
/// - Use `AtomicUsize`, `AtomicBool`, etc. for simple atomic types
pub struct SyncUnsafeCell<T>(UnsafeCell<T>);

/// Manual implementation of `Sync` for `SyncUnsafeCell<T>`.
///
/// # Safety
///
/// This is **unsafe** because it asserts that `SyncUnsafeCell<T>` can be shared
/// between threads without causing data races. The caller must ensure synchronization.
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

/// Manual implementation of `Send` for `SyncUnsafeCell<T>`.
///
/// # Safety
///
/// This is **unsafe** because it asserts that `SyncUnsafeCell<T>` can be transferred
/// between threads. The inner type `T` may not be `Send`, so the caller must handle
/// memory safety.
unsafe impl<T> Send for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    /// Creates a new instance of `SyncUnsafeCell<T>`.
    ///
    /// This is a `const` function, allowing initialization in static and
    /// constant contexts.
    ///
    /// # Parameters
    ///
    /// * `value` - The initial value to wrap
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::SyncUnsafeCell;
    ///
    /// static CONFIG: SyncUnsafeCell<u32> = SyncUnsafeCell::new(42);
    /// ```
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }
    
    /// Gets a raw mutable pointer to the contained value.
    ///
    /// # Safety
    ///
    /// This function is **unsafe** because:
    /// - It returns a raw pointer that bypasses the borrow checker
    /// - The caller must ensure there are no mutable aliases
    /// - Dereferencing the pointer without synchronization can cause data races
    ///
    /// # Returns
    ///
    /// A raw mutable pointer `*mut T` to the inner value.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::SyncUnsafeCell;
    ///
    /// static VALUE: SyncUnsafeCell<i32> = SyncUnsafeCell::new(0);
    ///
    /// unsafe {
    ///     let ptr = VALUE.get();
    ///     *ptr = 42;
    /// }
    /// ```
    pub unsafe fn get(&self) -> *mut T {
        self.0.get()
    }
}