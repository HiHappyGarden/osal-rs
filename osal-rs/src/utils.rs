/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Utility types and functions for OSAL-RS.
//!
//! This module contains common types, error definitions, and helper functions
//! used throughout the library.
//!
//! # Overview
//!
//! The utilities module provides essential building blocks for working with
//! OSAL-RS in embedded environments:
//!
//! - **Error handling**: Comprehensive [`Error`] enum for all OSAL operations
//! - **String utilities**: Fixed-size [`Bytes`] type for embedded string handling
//! - **Conversion macros**: Safe C string conversion and parameter extraction
//! - **FFI types**: Type aliases for C interoperability
//!
//! # Main Types
//!
//! ## Error Handling
//!
//! - [`Error<'a>`] - All possible error conditions with optional borrowed error messages
//! - [`Result<T, E>`] - Type alias for `core::result::Result` with default `Error<'static>`
//! - [`OsalRsBool`] - Boolean type compatible with RTOS return values
//!
//! ## String Handling
//!
//! - [`Bytes<SIZE>`] - Fixed-size byte buffer with string conversion utilities
//! - [`AsSyncStr`] - Trait for thread-safe string references
//!
//! ## Constants
//!
//! - [`MAX_DELAY`] - Maximum timeout for blocking indefinitely
//! - [`CpuRegisterSize`] - CPU register size detection (32-bit or 64-bit)
//!
//! ## FFI Types
//!
//! - [`Ptr`], [`ConstPtr`], [`DoublePtr`] - Type aliases for C pointers
//!
//! # Macros
//!
//! ## Parameter Handling
//!
//! - [`thread_extract_param!`] - Extract typed parameter from thread entry point
//! - [`access_static_option!`] - Access static Option variable (panics if None)
//!
//! # Helper Functions
//!
//! ## Hex Conversion
//!
//! - [`bytes_to_hex`] - Convert bytes to hex string (allocates)
//! - [`bytes_to_hex_into_slice`] - Convert bytes to hex into buffer (no allocation)
//! - [`hex_to_bytes`] - Parse hex string to bytes (allocates)
//! - [`hex_to_bytes_into_slice`] - Parse hex string into buffer (no allocation)
//!
//! # Platform Detection
//!
//! - [`register_bit_size`] - Const function to detect CPU register size (32-bit or 64-bit)
//!
//! # Best Practices
//!
//! 1. **Use `Bytes<SIZE>` for embedded strings**: Avoids heap allocation, fixed size
//! 2. **Prefer no-alloc variants**: Use `_into_slice` functions when possible
//! 3. **Handle errors explicitly**: Always check `Result` returns

use core::ffi::{CStr, c_char, c_uchar, c_void};
use core::str::{from_utf8_mut, FromStr};
use core::fmt::{Debug, Display}; 
use core::ops::{Deref, DerefMut};
use core::time::Duration;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(not(feature = "serde"))]
use crate::os::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use osal_rs_serde::{Deserialize, Serialize};

/// Error types for OSAL-RS operations.
///
/// Represents all possible error conditions that can occur when using
/// the OSAL-RS library.
///
/// # Lifetime Parameter
///
/// The error type is generic over lifetime `'a` to allow flexible error messages.
/// Most of the time, you can use the default [`Result<T>`] type alias which uses
/// `Error<'static>`. For custom lifetimes in error messages, use
/// `core::result::Result<T, Error<'a>>` explicitly.
///
/// # Examples
///
/// ## Basic usage with static errors
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
///
/// ## Using borrowed error messages
///
/// ```ignore
/// use osal_rs::utils::Error;
/// 
/// fn validate_input(input: &str) -> core::result::Result<(), Error> {
///     if input.is_empty() {
///         // Use static lifetime for compile-time strings
///         Err(Error::Unhandled("Input cannot be empty"))
///     } else {
///         Ok(())
///     }
/// }
/// 
/// // For dynamic error messages from borrowed data
/// fn process_data<'a>(data: &'a str) -> core::result::Result<(), Error<'a>> {
///     if !data.starts_with("valid:") {
///         // Error message borrows from 'data' lifetime
///         Err(Error::ReadError(data))
///     } else {
///         Ok(())
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error<'a> {
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
    WriteError(&'a str),
    /// Read error occurred
    ReadError(&'a str),
    /// Return error with code
    ReturnWithCode(i32),
    /// Unhandled error with description
    Unhandled(&'a str),
    /// Unhandled error with description owned
    UnhandledOwned(String)
}

impl<'a> Display for Error<'a> {
    /// Formats the error for display.
    ///
    /// Provides human-readable error messages suitable for logging or
    /// presentation to users.
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
/// This is used for platform-specific tick count overflow handling and
/// time calculation optimizations.
///
/// # Usage
///
/// Typically determined at compile time via [`register_bit_size()`] which
/// checks `size_of::<usize>()`.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::{CpuRegisterSize, register_bit_size};
///
/// match register_bit_size() {
///     CpuRegisterSize::Bit64 => {
///         // Use 64-bit optimized calculations
///     }
///     CpuRegisterSize::Bit32 => {
///         // Use 32-bit overflow-safe calculations
///     }
/// }
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CpuRegisterSize {
    /// 64-bit CPU registers (e.g., ARM Cortex-A, x86_64).
    ///
    /// On these platforms, `usize` is 8 bytes.
    Bit64,
    
    /// 32-bit CPU registers (e.g., ARM Cortex-M, RP2040, ESP32).
    ///
    /// On these platforms, `usize` is 4 bytes.
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
/// Uses [`Error`] as the default error type with `'static` lifetime.
/// For custom lifetimes, use `core::result::Result<T, Error<'a>>`.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::Result;
///
/// fn create_resource() -> Result<ResourceHandle> {
///     // Returns Result<ResourceHandle, Error<'static>>
///     Ok(ResourceHandle::new())
/// }
/// ```
pub type Result<T, E = Error<'static>> = core::result::Result<T, E>;

/// Pointer to pointer type for C FFI.
///
/// Equivalent to `void**` in C. Used for double indirection in FFI calls.
pub type DoublePtr = *mut *mut c_void;

/// Mutable pointer type for C FFI.
///
/// Equivalent to `void*` in C. Used for generic mutable data pointers.
pub type Ptr = *mut c_void;

/// Const pointer type for C FFI.
///
/// Equivalent to `const void*` in C. Used for generic immutable data pointers.
pub type ConstPtr = *const c_void;


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

/// Accesses a static Option variable, returning the contained value or panicking if None.
/// 
/// This macro is used to safely access static variables that are initialized at runtime.
/// It checks if the static variable is `Some` and returns the contained value. If the variable
/// is `None`, it panics with a message indicating that the variable is not initialized.
/// 
/// # Parameters
/// * `$static_var` - The identifier of the static variable to access
/// # Returns
/// * The value contained in the static variable if it is `Some`
/// * Panics if the static variable is `None`, with a message indicating it is not initialized
/// # Examples
/// ```ignore
/// use osal_rs::access_static_option;
/// static mut CONFIG: Option<Config> = None;
/// fn get_config() -> &'static Config {
///     access_static_option!(CONFIG)
/// }
/// ```
/// 
/// Note: This macro assumes that the static variable is of type `Option<T>` and that it is initialized at runtime before being accessed. It is intended for use with static variables that are set up during initialization phases of the program, such as in embedded systems where certain resources are not available at compile time.
/// 
/// # Safety
/// This macro uses unsafe code to access the static variable. It is the caller's responsibility to ensure that the static variable is properly initialized before it is accessed, and that it is not accessed concurrently from multiple threads without proper synchronization.
/// # Warning
/// This macro will panic if the static variable is not initialized (i.e., if it is `None`). It should be used in contexts where it is guaranteed that the variable will be initialized before
/// accessing it, such as after an initialization function has been called.
/// # Alternative
/// For safer access to static variables, consider using a function that returns a `Result` instead of panicking, allowing the caller to handle the error condition gracefully.
/// ```ignore
/// fn get_config() -> Result<&'static Config, Error> {
///    unsafe {
///       match &*&raw const CONFIG {
///         Some(config) => Ok(config),
///        None => Err(Error::Unhandled("CONFIG is not initialized")),
///     }
///  }
/// }
/// ```
/// This alternative approach allows for error handling without panicking, which can be more appropriate in many contexts, especially in production code or libraries where robustness is important.
/// # Note
/// This macro is intended for use in embedded systems or low-level code where static variables are commonly used for global state or resources that are initialized at runtime. It provides a convenient way to access such
/// variables while ensuring that they are initialized, albeit with the risk of panicking if they are not. Use with caution and ensure proper initialization to avoid runtime panics.
#[macro_export]
macro_rules! access_static_option {
    ($static_var:ident) => {
        unsafe {
            match &*&raw const $static_var {
                Some(value) => value,
                None => panic!(concat!(stringify!($static_var), " is not initialized")),
            }
        }
    };
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

impl PartialEq for dyn AsSyncStr + '_ {
    fn eq(&self, other: &(dyn AsSyncStr + '_)) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for dyn AsSyncStr + '_ {}

impl Debug for dyn AsSyncStr + '_ {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for dyn AsSyncStr + '_ {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
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
            .unwrap_or("Bytes::fmt() Conversion error - invalid UTF-8")
        };
        
        write!(f, "{}", str.to_string())
    }
}

impl<const SIZE: usize> FromStr for Bytes<SIZE> {
    type Err = Error<'static>;

    /// Creates a `Bytes` instance from a string slice.
    ///
    /// This implementation allows for easy conversion from string literals or
    /// string slices to the `Bytes` type, filling the internal byte array
    /// with the string data and padding with spaces if necessary.
    ///
    /// # Examples
    //// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes: Bytes<16> = "Hello".parse().unwrap();
    /// println!("{}", bytes); // Prints "Hello"
    /// ```
    #[inline]
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
}

impl<const SIZE: usize> From<&str> for Bytes<SIZE> {
    /// Creates a `Bytes` instance from a string slice.
    ///
    /// This implementation allows for easy conversion from string literals or
    /// string slices to the `Bytes` type, filling the internal byte array
    /// with the string data and padding with spaces if necessary.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes: Bytes<16> = "Hello".into();
    /// println!("{}", bytes); // Prints "Hello"
    /// ```
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl<const SIZE: usize> core::fmt::Write for Bytes<SIZE> {
    /// Appends a string slice to the buffer, truncating if the content exceeds `SIZE`.
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.append_str(s);
        Ok(())
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
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
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
        // Find the actual length (up to first null byte or SIZE)
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(SIZE);
        
        // Try to serialize as UTF-8 string if valid, otherwise as hex
        if let Ok(s) = core::str::from_utf8(&self.0[..len]) {
            serializer.serialize_str(name, s)
        } else {
            // For binary data, serialize as bytes (hex encoded)
            serializer.serialize_bytes(name, &self.0[..len])
        }
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
        let _ = deserializer.deserialize_bytes(name, &mut array)?;
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


/// Default implementation for `Bytes<SIZE>`.
/// This provides a default value for `Bytes<SIZE>`, which is a zero-initialized byte array. This allows `Bytes` to be used in contexts that require a default value, such as when using the `Default` trait or when initializing variables without explicit values.
/// # Examples
/// ```ignore
/// use osal_rs::utils::Bytes;
/// 
/// let default_bytes: Bytes<16> = Default::default();
/// assert_eq!(default_bytes[0], 0);
/// ```
/// The default implementation initializes the internal byte array to all zeros, which is a common default state for byte buffers in embedded systems and C APIs. This ensures that any uninitialized `Bytes` instance will contain predictable data (zeros) rather than random memory content.
/// This is particularly useful when `Bytes` is used as a buffer for C string operations, as it ensures that the buffer starts in a known state. Additionally, it allows for easy creation of empty buffers that can be filled later without needing to manually initialize the array each time.
/// Overall, this default implementation enhances the usability of the `Bytes` type by providing a sensible default state that is commonly needed in embedded and systems programming contexts.
/// 
impl<const SIZE: usize> Default for Bytes<SIZE> {
    /// Provides a default value for `Bytes<SIZE>`, which is a zero-initialized byte array.
    ///
    /// This implementation allows `Bytes` to be used in contexts that require a default value,
    /// such as when using the `Default` trait or when initializing variables without explicit values.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let default_bytes: Bytes<16> = Default::default();
    /// assert_eq!(default_bytes[0], 0);
    /// ```
    fn default() -> Self {
        Self( [0u8; SIZE] )
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
    #[inline]
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
    pub fn from_str(str: &str) -> Self {

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
    /// * `ptr` - A pointer to a null-terminated C string (`*const c_char`)
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
    pub fn from_char_ptr(ptr: *const c_char) -> Self {
        if ptr.is_null() {
            return Self::new();
        }

        let mut array = [0u8; SIZE];

        let mut i = 0usize ;
        for byte in unsafe { CStr::from_ptr(ptr) }.to_bytes() {
            if i > SIZE - 1{
                break;
            }
            array[i] = *byte;
            i += 1;
        }

        Self( array )
    }


    /// Creates a new `Bytes` instance from a C unsigned char pointer.
    /// 
    /// Safely converts a pointer to an array of unsigned chars into a `Bytes` instance. If the pointer is null, returns a zero-initialized `Bytes`. The function copies bytes from the source pointer into the fixed-size array, truncating if the source is longer than `SIZE`.
    /// 
    /// # Parameters
    /// * `ptr` - A pointer to an array of unsigned chars (`*const c_uchar`)
    /// 
    /// # Safety
    /// While this function is not marked unsafe, it internally uses `unsafe` code to dereference the pointer. The caller must ensure that:
    /// - If not null, the pointer points to a valid array of unsigned chars with at least `SIZE` bytes
    /// - The memory the pointer references remains valid for the duration of the call
    /// 
    /// # Returns
    /// A `Bytes` instance containing the data from the source pointer, or zero-initialized if the pointer is null.
    /// 
    /// # Examples
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// use core::ffi::c_uchar;
    /// use alloc::ffi::CString;
    /// 
    /// // From a C unsigned char pointer
    /// let data = [b'H', b'e', b'l', b'l', b'o', 0];
    /// let bytes = Bytes::<16>::from_uchar_ptr(data.as_ptr());
    /// 
    /// // From a null pointer
    /// let null_bytes = Bytes::<16>::from_uchar_ptr(core::ptr::null());  
    /// // Returns zero-initialized Bytes
    /// 
    /// // Truncation example
    /// let long_data = [b'T', b'h', b'i', b's', b' ', b'i', b's', b' ', b'v', b'e', b'r', b'y', b' ', b'l', b'o', b'n', b'g', 0];
    /// let short_bytes = Bytes::<8>::from_uchar_ptr(long_data.as_ptr());
    /// // Only first 8 bytes are copied
    /// ```
    pub fn from_uchar_ptr(ptr: *const c_uchar) -> Self {
        if ptr.is_null() {
            return Self::new();
        }

        let mut array = [0u8; SIZE];

        let mut i = 0usize ;
        for byte in unsafe { core::slice::from_raw_parts(ptr, SIZE) } {
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
    #[inline]
    pub fn from_as_sync_str(str: &impl ToString) -> Self {
        Self::from_str(&str.to_string())
    }

    /// Creates a new `Bytes` instance from a byte slice.
    /// 
    /// This function copies bytes from the input slice into the fixed-size array. If the slice is shorter than `SIZE`, the remaining bytes are zero-filled. If the slice is longer, it is truncated to fit.
    /// 
    /// # Parameters
    /// * `bytes` - The source byte slice to convert
    /// 
    /// # Returns
    /// A `Bytes` instance containing the data from the byte slice.
    /// 
    /// # Examples
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let data = b"Hello";
    /// let bytes = Bytes::<16>::from_bytes(data);
    /// // Result: [b'H', b'e', b'l', b'l', b'o', 0, 0, 0, ...]
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Self {
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

    /// Creates a new `Bytes` instance from a C string pointer.
    ///
    /// This is a convenience wrapper around [`new_by_ptr`](Self::new_by_ptr) that directly converts a C string pointer to a `Bytes` instance.
    /// If the pointer is null, it returns a zero-initialized `Bytes`. The function copies bytes from the C string into the fixed-size array, truncating if the source is longer than `SIZE`.
    ///
    /// # Parameters
    ///
    /// * `str` - A pointer to a null-terminated C string (`*const c_char`)
    ///
    /// # Safety
    ///
    /// This method uses `unsafe` code to dereference the pointer. The caller must ensure that:
    /// - If not null, the pointer points to a valid null-terminated C string
    /// - The memory the pointer references remains valid for the duration of the call
    ///
    /// - The byte array can be safely interpreted as UTF-8 if the conversion is expected to succeed. If the byte array contains invalid UTF-8, the resulting `Bytes` instance will contain the raw bytes, and the `Display` implementation will show "Conversion error" when attempting to display it as a string.
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
    /// let bytes = Bytes::<16>::from_cstr(c_string.as_ptr());
    /// 
    /// // From a null pointer
    /// let null_bytes = Bytes::<16>::from_cstr(core::ptr::null());
    /// // Returns zero-initialized Bytes
    /// 
    /// // Truncation example
    /// let long_string = CString::new("This is a very long string").unwrap();
    /// let short_bytes = Bytes::<8>::from_cstr(long_string.as_ptr());
    /// // Only first 8 bytes are copied
    /// ```
    #[inline]
    pub fn from_cstr(str: *const c_char) -> Self {
        Self::from_bytes(unsafe { CStr::from_ptr(str) }.to_bytes())
    }

    /// Converts the byte array to a C string reference.
    ///
    /// Creates a `CStr` reference from the internal byte array, treating it as
    /// a null-terminated C string. This is useful for passing strings to C FFI
    /// functions that expect `*const c_char` or `&CStr`.
    ///
    /// # Safety
    ///
    /// This method assumes the byte array is already null-terminated. All
    /// constructors (`new()`, `from_str()`, `from_char_ptr()`, etc.) guarantee
    /// this property by initializing with `[0u8; SIZE]`.
    ///
    /// However, if you've manually modified the array via `DerefMut`,
    /// you must ensure the last byte remains 0.
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
    /// let c_str = bytes.as_cstr();
    /// 
    /// extern "C" {
    ///     fn print_string(s: *const core::ffi::c_char);
    /// }
    /// 
    /// unsafe {
    ///     print_string(c_str.as_ptr());
    /// }
    /// ```
    #[inline]
    pub fn as_cstr(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(self.0.as_ptr() as *const c_char)
        }
    }

    /// Converts the byte array to a C string reference, ensuring null-termination.
    ///
    /// This is a safer version of `as_cstr()` that explicitly guarantees
    /// null-termination by modifying the last byte. Use this if you've
    /// manually modified the array and want to ensure it's null-terminated.
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
    /// let mut bytes = Bytes::<16>::new();
    /// bytes[0] = b'H';
    /// bytes[1] = b'i';
    /// // After manual modification, ensure null-termination
    /// let c_str = bytes.as_cstr_mut();
    /// ```
    #[inline]
    pub fn as_cstr_mut(&mut self) -> &CStr {
        unsafe {
            self.0[SIZE - 1] = 0; // Ensure null-termination
            CStr::from_ptr(self.0.as_ptr() as *const c_char)
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
    /// * `OTHER_SIZE` - The size of the source `Bytes` buffer (can be different from `SIZE`)
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
    pub fn append<const OTHER_SIZE: usize>(&mut self, other: &Bytes<OTHER_SIZE>) {
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


    /// Prepends a string slice to the existing content in the `Bytes` buffer.
    ///
    /// This method inserts the provided string at the beginning of the buffer,
    /// shifting the existing content to the right. If the combined length exceeds
    /// `SIZE`, the existing content is truncated to fit within the buffer.
    ///
    /// # Parameters
    ///
    /// * `str` - The string slice to prepend
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("World");
    /// bytes.prepend_str("Hello ");
    /// assert_eq!(bytes.as_str(), "Hello World");
    ///
    /// // Truncation when exceeding buffer size
    /// let mut small = Bytes::<8>::new_by_str("World");
    /// small.prepend_str("Hello ");
    /// assert_eq!(small.as_str(), "Hello Wo");
    /// ```
    pub fn prepend_str(&mut self, str: &str) {
        let current_len = self.0.iter().position(|&b| b == 0).unwrap_or(SIZE);
        let prefix = str.as_bytes();
        let prefix_len = prefix.len().min(SIZE);
        let keep_len = (SIZE - prefix_len).min(current_len);
        if keep_len > 0 {
            self.0.copy_within(0..keep_len, prefix_len);
        }
        self.0[..prefix_len].copy_from_slice(&prefix[..prefix_len]);
        let new_len = prefix_len + keep_len;
        if new_len < SIZE {
            self.0[new_len] = 0;
        }
    }

    /// Prepends content from any type implementing `AsSyncStr` to the buffer.
    ///
    /// This method accepts any type that implements the `AsSyncStr` trait, converts
    /// it to a string slice, and prepends it to the existing content. If the combined
    /// length exceeds `SIZE`, the existing content is truncated to fit.
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
    /// let mut bytes = Bytes::<16>::new_by_str("World");
    /// let prefix = Bytes::<8>::new_by_str("Hello ");
    /// bytes.prepend_as_sync_str(&prefix);
    /// assert_eq!(bytes.as_str(), "Hello World");
    /// ```
    pub fn prepend_as_sync_str(&mut self, c_str: & impl AsSyncStr) {
        self.prepend_str(c_str.as_str());
    }

    /// Prepends raw bytes to the existing content in the `Bytes` buffer.
    ///
    /// This method inserts the provided byte slice at the beginning of the buffer,
    /// shifting the existing content to the right. If the combined length exceeds
    /// `SIZE`, the existing content is truncated to fit within the buffer.
    ///
    /// # Parameters
    ///
    /// * `bytes` - The byte slice to prepend
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("World");
    /// bytes.prepend_bytes(b"Hello ");
    /// assert_eq!(bytes.as_str(), "Hello World");
    ///
    /// // Prepending arbitrary bytes
    /// let mut data = Bytes::<16>::new_by_str("BC");
    /// data.prepend_bytes(&[0x41]); // 'A'
    /// assert_eq!(data.as_str(), "ABC");
    /// ```
    pub fn prepend_bytes(&mut self, bytes: &[u8]) {
        let current_len = self.0.iter().position(|&b| b == 0).unwrap_or(SIZE);
        let prefix_len = bytes.len().min(SIZE);
        let keep_len = (SIZE - prefix_len).min(current_len);
        if keep_len > 0 {
            self.0.copy_within(0..keep_len, prefix_len);
        }
        self.0[..prefix_len].copy_from_slice(&bytes[..prefix_len]);
        let new_len = prefix_len + keep_len;
        if new_len < SIZE {
            self.0[new_len] = 0;
        }
    }

    /// Prepends the content of another `Bytes` instance to this buffer.
    ///
    /// This method allows prepending content from a `Bytes` instance of a different
    /// size (specified by the generic parameter `OTHER_SIZE`). The method inserts the
    /// content of the other `Bytes` at the beginning, shifting existing content to the
    /// right. If the combined length exceeds `SIZE`, the existing content is truncated.
    ///
    /// # Type Parameters
    ///
    /// * `OTHER_SIZE` - The size of the source `Bytes` buffer (can be different from `SIZE`)
    ///
    /// # Parameters
    ///
    /// * `other` - A reference to the `Bytes` instance to prepend
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("World");
    /// let prefix = Bytes::<8>::new_by_str("Hello ");
    /// bytes.prepend(&prefix);
    /// assert_eq!(bytes.as_str(), "Hello World");
    ///
    /// // Prepending from a larger buffer with truncation
    /// let mut small = Bytes::<8>::new_by_str("end");
    /// let large = Bytes::<32>::new_by_str("begin_");
    /// small.prepend(&large);
    /// assert_eq!(small.as_str(), "begin_en");
    /// ```
    pub fn prepend<const OTHER_SIZE: usize>(&mut self, other: &Bytes<OTHER_SIZE>) {
        let other_len = other.0.iter().position(|&b| b == 0).unwrap_or(OTHER_SIZE);
        self.prepend_bytes(&other.0[..other_len]);
    }

    /// Clears all content from the buffer, filling it with zeros.
    ///
    /// This method resets the entire internal byte array to zeros, effectively
    /// clearing any stored data. After calling this method, the buffer will be
    /// empty and ready for new content.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// assert!(!bytes.is_empty());
    ///
    /// bytes.clear();
    /// assert!(bytes.is_empty());
    /// assert_eq!(bytes.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        for byte in self.0.iter_mut() {
            *byte = 0;
        }
    }

    /// Returns the length of the content in the buffer.
    ///
    /// The length is determined by finding the position of the first null byte (0).
    /// If no null byte is found, returns `SIZE`, indicating the buffer is completely
    /// filled with non-zero data.
    ///
    /// # Returns
    ///
    /// The number of bytes before the first null terminator, or `SIZE` if the
    /// buffer is completely filled.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// let bytes = Bytes::<16>::new_by_str("Hello");
    /// assert_eq!(bytes.len(), 5);
    ///
    /// let empty = Bytes::<16>::new();
    /// assert_eq!(empty.len(), 0);
    ///
    /// // Buffer completely filled (no null terminator)
    /// let mut full = Bytes::<4>::new();
    /// full[0] = b'A';
    /// full[1] = b'B';
    /// full[2] = b'C';
    /// full[3] = b'D';
    /// assert_eq!(full.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.iter().position(|&b| b == 0).unwrap_or(SIZE)
    }

    /// Returns a byte slice of the content in the buffer.
    /// 
    /// This method returns a slice of the internal byte array up to the first null byte (0). If no null byte is found, it returns a slice of the entire array. This allows you to access the valid content stored in the buffer without including any trailing zeros.
    /// 
    /// # Returns
    /// A byte slice containing the content of the buffer up to the first null terminator.
    /// 
    /// # Examples
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes = Bytes::<16>::new_by_str("Hello");
    /// assert_eq!(bytes.as_raw_bytes(), b"Hello");
    /// 
    /// let empty = Bytes::<16>::new();
    /// assert_eq!(empty.as_raw_bytes(), b"");
    /// 
    /// let full = Bytes::<4>::new_by_str("ABCD");
    /// assert_eq!(full.as_raw_bytes(), b"ABCD");
    /// ``` 
    #[inline]
    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.0[..self.len()]
    }

    /// Returns the fixed size of the buffer.
    /// 
    /// This method returns the compile-time constant `SIZE`, which represents the total capacity of the internal byte array. The size is determined by the generic parameter `SIZE` specified when creating the `Bytes` instance. This value is fixed and does not change during the lifetime of the instance.
    /// # Returns
    /// The fixed size of the buffer in bytes (`SIZE`).
    /// # Examples
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let bytes = Bytes::<32>::new();
    /// assert_eq!(bytes.size(), 32);
    /// 
    /// let other = Bytes::<128>::new_by_str("Hello");
    /// assert_eq!(other.size(), 128);
    /// ```
    #[inline]
    pub const fn size(&self) -> usize {
        SIZE
    }

    /// Checks if the buffer is empty.
    ///
    /// A buffer is considered empty if all bytes are zero. This method searches
    /// for the first non-zero byte to determine emptiness.
    ///
    /// # Returns
    ///
    /// `true` if all bytes are zero, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// let empty = Bytes::<16>::new();
    /// assert!(empty.is_empty());
    ///
    /// let bytes = Bytes::<16>::new_by_str("Hello");
    /// assert!(!bytes.is_empty());
    ///
    /// let mut cleared = Bytes::<16>::new_by_str("Test");
    /// cleared.clear();
    /// assert!(cleared.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.iter().position(|&b| b != 0).is_none()
    }

    /// Returns the total capacity of the buffer.
    ///
    /// This is the fixed size of the internal byte array, determined at compile
    /// time by the generic `SIZE` parameter. The capacity never changes during
    /// the lifetime of the `Bytes` instance.
    ///
    /// # Returns
    ///
    /// The total capacity in bytes (`SIZE`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// let bytes = Bytes::<32>::new();
    /// assert_eq!(bytes.capacity(), 32);
    ///
    /// let other = Bytes::<128>::new_by_str("Hello");
    /// assert_eq!(other.capacity(), 128);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        SIZE
    }

    /// Replaces all occurrences of a byte pattern with another pattern.
    ///
    /// This method searches for all occurrences of the `find` byte sequence within
    /// the buffer and replaces them with the `replace` byte sequence. The replacement
    /// is performed in a single pass, and the method handles cases where the replacement
    /// is larger, smaller, or equal in size to the pattern being searched for.
    ///
    /// # Parameters
    ///
    /// * `find` - The byte pattern to search for
    /// * `replace` - The byte pattern to replace with
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all replacements were successful
    /// * `Err(Error::StringConversionError)` - If the replacement would exceed the buffer capacity
    ///
    /// # Behavior
    ///
    /// - Empty `find` patterns are ignored (returns `Ok(())` immediately)
    /// - Multiple occurrences are replaced in a single pass
    /// - Content is properly shifted when replacement size differs from find size
    /// - Null terminators and trailing bytes are correctly maintained
    /// - Overlapping patterns are not re-matched (avoids infinite loops)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// // Same length replacement
    /// let mut bytes = Bytes::<16>::new_by_str("Hello World");
    /// bytes.replace(b"World", b"Rust!").unwrap();
    /// assert_eq!(bytes.as_str(), "Hello Rust!");
    ///
    /// // Shorter replacement
    /// let mut bytes2 = Bytes::<16>::new_by_str("aabbcc");
    /// bytes2.replace(b"bb", b"X").unwrap();
    /// assert_eq!(bytes2.as_str(), "aaXcc");
    ///
    /// // Longer replacement
    /// let mut bytes3 = Bytes::<16>::new_by_str("Hi");
    /// bytes3.replace(b"Hi", b"Hello").unwrap();
    /// assert_eq!(bytes3.as_str(), "Hello");
    ///
    /// // Multiple occurrences
    /// let mut bytes4 = Bytes::<32>::new_by_str("foo bar foo");
    /// bytes4.replace(b"foo", b"baz").unwrap();
    /// assert_eq!(bytes4.as_str(), "baz bar baz");
    ///
    /// // Buffer overflow error
    /// let mut small = Bytes::<8>::new_by_str("Hello");
    /// assert!(small.replace(b"Hello", b"Hello World").is_err());
    /// ```
    pub fn replace(&mut self, find: &[u8], replace: &[u8]) -> Result<()> {
        if find.is_empty() {
            return Ok(());
        }
        
        let mut i = 0;
        loop {
            let current_len = self.len();
            
            // Exit if we've reached the end
            if i >= current_len {
                break;
            }
            
            // Check if pattern starts at position i
            if i + find.len() <= current_len && self.0[i..i + find.len()] == *find {
                let remaining_len = current_len - (i + find.len());
                let new_len = i + replace.len() + remaining_len;
                
                // Check if replacement fits in buffer
                if new_len > SIZE {
                    return Err(Error::StringConversionError);
                }
                
                // Shift remaining content if sizes differ
                if replace.len() != find.len() {
                    self.0.copy_within(
                        i + find.len()..i + find.len() + remaining_len,
                        i + replace.len()
                    );
                }
                
                // Insert replacement bytes
                self.0[i..i + replace.len()].copy_from_slice(replace);
                
                // Update null terminator position
                if new_len < SIZE {
                    self.0[new_len] = 0;
                }
                
                // Clear trailing bytes if content shrunk
                if new_len < current_len {
                    for j in (new_len + 1)..=current_len {
                        if j < SIZE {
                            self.0[j] = 0;
                        }
                    }
                }
                
                // Move past the replacement to avoid infinite loops
                i += replace.len();
            } else {
                i += 1;
            }
        }
        
        Ok(())
    }

    /// Converts the `Bytes` instance to a byte slice.
    ///
    /// This method provides a convenient way to access the internal byte array
    /// as a slice, which can be useful for C FFI or other operations that
    /// require byte slices.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::{Bytes, ToBytes};
    /// 
    /// let bytes = Bytes::<8>::new_by_str("example");
    /// let byte_slice = bytes.to_bytes();
    /// assert_eq!(byte_slice, b"example\0\0");
    /// ```
    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Pops the last byte from the buffer and returns it.
    ///
    /// This method removes the last byte of content (before the first null terminator)
    /// and returns it. If the buffer is empty, it returns `None`. After popping, the last byte is set to zero to maintain the null-terminated property.
    ///
    /// # Returns
    ///
    /// * `Some(u8)` - The last byte of content if the buffer is not empty
    /// * `None` - If the buffer is empty
    ///
    /// # Examples
    //// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// assert_eq!(bytes.pop(), Some(b'o'));
    /// assert_eq!(bytes.as_str(), "Hell");
    /// 
    /// // Pop until empty
    /// assert_eq!(bytes.pop(), Some(b'l'));
    /// assert_eq!(bytes.pop(), Some(b'l'));
    /// assert_eq!(bytes.pop(), Some(b'e'));
    /// assert_eq!(bytes.pop(), Some(b'H'));
    /// assert_eq!(bytes.pop(), None);
    /// ``` 
    pub fn pop(&mut self) -> Option<u8> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            let byte = self.0[len - 1];
            self.0[len - 1] = 0; // Clear the popped byte
            Some(byte)
        }
    }

    /// Pushes a byte to the end of the content in the buffer.
    ///
    /// # Parameters
    ///
    /// * `byte` - The byte to push into the buffer
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the byte was successfully pushed
    /// * `Err(Error::StringConversionError)` - If the buffer is full
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// assert_eq!(bytes.push(b'!'), Ok(()));
    /// assert_eq!(bytes.as_str(), "Hello!");
    /// ```
    pub fn push(&mut self, byte: u8) -> Result<()> {
        let len = self.len();
        if len >= SIZE {
            Err(Error::StringConversionError) // Buffer is full
        } else {
            self.0[len] = byte;
            Ok(())
        }
    }

    /// Pops the last byte from the buffer and returns it as a character.
    ///
    /// This method removes the last byte of content (before the first null terminator)
    /// and attempts to convert it to a `char`. If the buffer is empty or if the byte cannot be converted to a valid `char`, it returns `None`. After popping, the last byte is set to zero to maintain the null-terminated property.
    ///
    /// # Returns
    ///
    /// * `Some(char)` - The last byte of content as a character if the buffer is not empty and the byte is a valid character
    /// * `None` - If the buffer is empty or if the byte cannot be converted to a valid character
    ///
    /// # Examples
    //// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// assert_eq!(bytes.pop_char(), Some('o'));
    /// assert_eq!(bytes.as_str(), "Hell");
    /// 
    /// // Pop until empty
    /// assert_eq!(bytes.pop_char(), Some('l'));
    /// assert_eq!(bytes.pop_char(), Some('l'));
    /// assert_eq!(bytes.pop_char(), Some('e'));
    /// assert_eq!(bytes.pop_char(), Some('H'));
    /// assert_eq!(bytes.pop_char(), None);
    /// ```
    #[inline]
    pub fn pop_char(&mut self) -> Option<char> {
        self.pop().and_then(|byte| char::from_u32(byte as u32))
    }

    /// Pushes a character to the end of the content in the buffer.
    ///
    /// This method attempts to convert the provided `char` to a byte and push it into the buffer. If the character is not a valid ASCII character (i.e., its code point is greater than 127), it returns an error since it cannot be represented as a single byte. If the buffer is full, it also returns an error.
    ///
    /// # Parameters
    ///
    /// * `ch` - The character to push into the buffer
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the character was successfully pushed
    /// * `Err(Error::StringConversionError)` - If the character is not a valid ASCII character or if the buffer is full
    ///
    /// # Examples
    //// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let mut bytes = Bytes::<16>::new_by_str("Hello");
    /// assert_eq!(bytes.push_char('!'), Ok(()));
    /// assert_eq!(bytes.as_str(), "Hello!");
    /// 
    /// // Attempt to push a non-ASCII character
    /// assert!(bytes.push_char('é').is_err());
    /// ```
    pub fn push_char(&mut self, ch: char) -> Result<()> {
        if ch.is_ascii() {
            self.push(ch as u8)
        } else {
            Err(Error::StringConversionError) // Non-ASCII characters not supported
        }
    }

    /// Checks if the content of the buffer can be interpreted as a valid UTF-8 string.
    ///
    /// This method attempts to convert the internal byte array to a UTF-8 string. If the conversion is successful, it returns `true`, indicating that the content can be treated as a valid string. If the conversion fails due to invalid UTF-8 sequences, it returns `false`.
    ///
    /// # Returns
    ///
    /// * `true` - If the content can be interpreted as a valid UTF-8 string
    /// * `false` - If the content contains invalid UTF-8 sequences
    ///
    /// # Examples
    //// ```ignore
    /// use osal_rs::utils::Bytes;
    /// 
    /// let valid_bytes = Bytes::<16>::new_by_str("Hello");
    /// assert!(valid_bytes.is_string());
    /// 
    /// let mut invalid_bytes = Bytes::<16>::new();
    /// invalid_bytes[0] = 0xFF; // Invalid UTF-8 byte
    /// assert!(!invalid_bytes.is_string());
    /// ```
    #[inline]
    pub fn is_string(&self) -> bool {
        String::from_utf8(self.0.to_vec()).is_ok()
    }

    /// Returns the buffer content as a UTF-8 string slice.
    ///
    /// Interprets the byte array as a null-terminated C string and returns
    /// a `&str`. If the bytes contain invalid UTF-8, returns `"Conversion error"`.
    ///
    /// This is an inherent method (no trait import required at the call site).
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.as_ptr() as *const c_char)
                .to_str()
                .unwrap_or("Bytes::as_str() Conversion error - invalid UTF-8")
        }
    }

    /// Overwrites the buffer with a formatted string, behaving like `alloc::format!`.
    ///
    /// Clears the current content and fills the buffer with the result of formatting
    /// `args`. Content that exceeds `SIZE` is silently truncated.
    ///
    /// # Parameters
    ///
    /// * `args` - A [`core::fmt::Arguments`] value, typically created with [`format_args!`]
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::utils::Bytes;
    ///
    /// let mut b = Bytes::<32>::new();
    /// b.format(format_args!("Hello {}", 42));
    /// assert_eq!(b.as_str(), "Hello 42");
    ///
    /// let mut b2 = Bytes::<8>::new();
    /// b2.format(format_args!("{:.2}", 3.14159));
    /// assert_eq!(b2.as_str(), "3.14");
    /// ```
    #[inline]
    pub fn format(&mut self, args: core::fmt::Arguments<'_>) {
        self.clear();
        let _ = core::fmt::write(self, args);
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
#[inline]
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
