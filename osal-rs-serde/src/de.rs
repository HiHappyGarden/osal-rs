/***************************************************************************
 *
 * osal-rs-serde
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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

//! Deserialization traits and implementations.
//!
//! This module provides the core deserialization functionality for osal-rs-serde.
//! It defines the [`Deserialize`] trait for types that can be deserialized and the
//! [`Deserializer`] trait for implementing custom deserialization formats.
//!
//! # Overview
//!
//! - [`Deserialize`]: Trait implemented by types that can be deserialized
//! - [`Deserializer`]: Trait for implementing custom deserialization formats  
//! - [`ByteDeserializer`]: Concrete implementation that reads little-endian binary data
//!
//! # Usage with Derive Macro
//!
//! The easiest way to implement deserialization is using the derive macro:
//!
//! ```ignore
//! use osal_rs_serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct SensorData {
//!     temperature: i16,
//!     humidity: u8,
//!     pressure: u32,
//! }
//! ```
//!
//! # Manual Implementation
//!
//! For custom deserialization logic, implement the trait manually:
//!
//! ```ignore
//! use osal_rs_serde::{Deserialize, Deserializer};
//!
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! impl Deserialize for Point {
//!     fn deserialize<D: Deserializer>(deserializer: &mut D, _name: &str) -> Result<Self, D::Error> {
//!         Ok(Point {
//!             x: deserializer.deserialize_i32("x")?,
//!             y: deserializer.deserialize_i32("y")?,
//!         })
//!     }
//! }
//! ```
//!
//! # Supported Types
//!
//! The deserialization framework supports:
//! - All primitive types (bool, integers, floats)
//! - Arrays `[T; N]` where T: Deserialize
//! - Tuples (up to 3 elements)
//! - `Option<T>` where T: Deserialize
//! - `Vec<T>` where T: Deserialize (requires `alloc`)
//! - `String` (requires `alloc`)
//! - Custom types implementing `Deserialize`

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::error::{Error, Result};

/// Trait for types that can be deserialized.
///
/// This trait should be implemented (or derived) for any type that needs to be deserialized.
/// The implementation defines how the type should be read from a deserializer.
///
/// # Derive Macro
///
/// The easiest way to implement this trait is using the derive macro (requires `derive` feature):
///
/// ```ignore
/// use osal_rs_serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     id: u32,
///     enabled: bool,
///     timeout: Option<u16>,
/// }
/// ```
///
/// # Manual Implementation
///
/// For custom deserialization logic or types not supported by the derive macro:
///
/// ```ignore
/// use osal_rs_serde::{Deserialize, Deserializer};
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl Deserialize for Point {
///     fn deserialize<D: Deserializer>(deserializer: &mut D, _name: &str) -> core::result::Result<Self, D::Error> {
///         Ok(Point {
///             x: deserializer.deserialize_i32("x")?,
///             y: deserializer.deserialize_i32("y")?,
///         })
///     }
/// }
/// ```
///
/// # Built-in Implementations
///
/// This trait is already implemented for:
/// - All primitive types (bool, u8-u128, i8-i128, f32, f64)
/// - Arrays `[T; N]` where T: Deserialize
/// - Tuples (T1, T2) and (T1, T2, T3) where all T: Deserialize
/// - `Option<T>` where T: Deserialize
/// - `Vec<T>` where T: Deserialize (requires `alloc`)
/// - `String` (requires `alloc`)
pub trait Deserialize: Sized {
    /// Deserialize this value using the given deserializer.
    /// The `name` parameter contains the field name or struct name for context.
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error>;
}

/// Trait that defines how to deserialize various types.
///
/// Implementations of this trait determine how to read data from a specific format.
/// For example, `ByteDeserializer` reads data in little-endian binary format.
pub trait Deserializer: Sized {
    /// The error type that can be returned during deserialization.
    type Error: From<Error>;

    /// Deserialize a `bool` value.
    fn deserialize_bool(&mut self, name: &str) -> core::result::Result<bool, Self::Error>;

    /// Deserialize a `u8` value.
    fn deserialize_u8(&mut self, name: &str) -> core::result::Result<u8, Self::Error>;

    /// Deserialize an `i8` value.
    fn deserialize_i8(&mut self, name: &str) -> core::result::Result<i8, Self::Error>;

    /// Deserialize a `u16` value.
    fn deserialize_u16(&mut self, name: &str) -> core::result::Result<u16, Self::Error>;

    /// Deserialize an `i16` value.
    fn deserialize_i16(&mut self, name: &str) -> core::result::Result<i16, Self::Error>;

    /// Deserialize a `u32` value.
    fn deserialize_u32(&mut self, name: &str) -> core::result::Result<u32, Self::Error>;

    /// Deserialize an `i32` value.
    fn deserialize_i32(&mut self, name: &str) -> core::result::Result<i32, Self::Error>;

    /// Deserialize a `u64` value.
    fn deserialize_u64(&mut self, name: &str) -> core::result::Result<u64, Self::Error>;

    /// Deserialize an `i64` value.
    fn deserialize_i64(&mut self, name: &str) -> core::result::Result<i64, Self::Error>;

    /// Deserialize a `u128` value.
    fn deserialize_u128(&mut self, name: &str) -> core::result::Result<u128, Self::Error>;

    /// Deserialize an `i128` value.
    fn deserialize_i128(&mut self, name: &str) -> core::result::Result<i128, Self::Error>;

    /// Deserialize an `f32` value.
    fn deserialize_f32(&mut self, name: &str) -> core::result::Result<f32, Self::Error>;

    /// Deserialize an `f64` value.
    fn deserialize_f64(&mut self, name: &str) -> core::result::Result<f64, Self::Error>;

    /// Deserialize bytes into a buffer. Returns the number of bytes read.
    fn deserialize_bytes(&mut self, name: &str, buffer: &mut [u8]) -> core::result::Result<usize, Self::Error>;

    /// Deserialize a string.
    /// Default implementation reads length and bytes.
    #[cfg(feature = "alloc")]
    fn deserialize_string(&mut self, name: &str) -> core::result::Result<alloc::string::String, Self::Error> {
        let len = self.deserialize_u32(name)? as usize;
        let mut buffer = alloc::vec![0u8; len];
        for i in 0..len {
            buffer[i] = self.deserialize_u8("")?;
        }
        alloc::string::String::from_utf8(buffer)
            .map_err(|_| Error::InvalidData.into())
    }

    /// Deserialize a vector of deserializable items.
    /// Default implementation reads length then deserializes each item.
    #[cfg(feature = "alloc")]
    fn deserialize_vec<T: Deserialize>(&mut self, name: &str) -> core::result::Result<alloc::vec::Vec<T>, Self::Error> {
        let len = self.deserialize_u32(name)? as usize;
        let mut vec = alloc::vec::Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(self, "")?);
        }
        Ok(vec)
    }

    /// Deserialize an array of deserializable items.
    /// Default implementation deserializes each item in sequence.
    fn deserialize_array<T: Deserialize, const N: usize>(&mut self, name: &str) -> core::result::Result<[T; N], Self::Error> {
        let mut result = core::mem::MaybeUninit::<[T; N]>::uninit();
        let result_ptr = result.as_mut_ptr() as *mut T;
        
        for i in 0..N {
            unsafe {
                result_ptr.add(i).write(T::deserialize(self, name)?);
            }
        }
        
        Ok(unsafe { result.assume_init() })
    }

    /// Begin deserializing a struct with the given name.
    /// Default implementation does nothing (suitable for binary formats).
    fn deserialize_struct_start(&mut self, _name: &str) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    /// Deserialize a struct field with name.
    /// Default implementation just deserializes the value.
    fn deserialize_field<T: Deserialize>(&mut self, name: &str) -> core::result::Result<T, Self::Error> {
        T::deserialize(self, name)
    }

    /// End deserializing a struct.
    /// Default implementation does nothing (suitable for binary formats).
    fn deserialize_struct_end(&mut self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }
}

/// A deserializer that reads data from a byte buffer in little-endian format.
///
/// This is a concrete implementation of the `Deserializer` trait that reads
/// binary data in a compact, little-endian format. This is the default deserializer
/// used by the [`crate::from_bytes`] convenience function.
///
/// # Format
///
/// This deserializer expects data in the same format as [`crate::ser::ByteSerializer`]:
/// - All integers in little-endian byte order
/// - Floating-point numbers in IEEE 754 representation
/// - `bool` as a single byte (0 or 1)
/// - `Option<T>`: 1 byte tag (0=None, 1=Some) followed by T if Some
/// - Arrays: Elements deserialized sequentially (no length prefix expected)
/// - Tuples: Elements deserialized sequentially
/// - Strings/Vec: u32 length prefix followed by data
///
/// # Examples
///
/// ## Basic Usage
///
/// ```ignore
/// use osal_rs_serde::{ByteDeserializer, Deserializer};
///
/// let buffer = [42u8, 0, 0, 0, 1, 156, 255];
/// let mut deserializer = ByteDeserializer::new(&buffer);
///
/// let value1 = deserializer.deserialize_u32("").unwrap();
/// let value2 = deserializer.deserialize_bool("").unwrap();
/// let value3 = deserializer.deserialize_i16("").unwrap();
///
/// assert_eq!(value1, 42);
/// assert_eq!(value2, true);
/// assert_eq!(value3, -100);
/// ```
///
/// ## With Structs
///
/// ```ignore
/// use osal_rs_serde::{ByteDeserializer, Deserialize, from_bytes};
///
/// #[derive(Deserialize)]
/// struct Message {
///     id: u32,
///     value: i16,
/// }
///
/// let buffer = [100, 0, 0, 0, 206, 255]; // id=100, value=-50 in little-endian
/// let msg: Message = from_bytes(&buffer).unwrap();
///
/// assert_eq!(msg.id, 100);
/// assert_eq!(msg.value, -50);
/// ```
///
/// # Error Handling
///
/// Returns [`Error::UnexpectedEof`](crate::error::Error::UnexpectedEof) if the buffer
/// doesn't contain enough data for the requested type.
///
/// # Memory Layout
///
/// The deserializer reads data sequentially without expecting padding or alignment:
///
/// ```text
/// struct Data { a: u16, b: u32 }
/// Memory: [a_lo, a_hi, b0, b1, b2, b3]
/// ```
pub struct ByteDeserializer<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> ByteDeserializer<'a> {
    /// Create a new ByteDeserializer with the given buffer.
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Get the current position in the buffer.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Read bytes from the buffer.
    fn read_bytes(&mut self, len: usize) -> Result<&[u8]> {
        if self.position + len > self.buffer.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = &self.buffer[self.position..self.position + len];
        self.position += len;
        Ok(bytes)
    }
}

impl<'a> Deserializer for ByteDeserializer<'a> {
    type Error = Error;

    fn deserialize_bool(&mut self, _name: &str) -> Result<bool> {
        Ok(self.deserialize_u8("")? != 0)
    }

    fn deserialize_u8(&mut self, _name: &str) -> Result<u8> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    fn deserialize_i8(&mut self, _name: &str) -> Result<i8> {
        let bytes = self.read_bytes(1)?;
        Ok(i8::from_le_bytes([bytes[0]]))
    }

    fn deserialize_u16(&mut self, _name: &str) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(buf))
    }

    fn deserialize_i16(&mut self, _name: &str) -> Result<i16> {
        let bytes = self.read_bytes(2)?;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(bytes);
        Ok(i16::from_le_bytes(buf))
    }

    fn deserialize_u32(&mut self, _name: &str) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(buf))
    }

    fn deserialize_i32(&mut self, _name: &str) -> Result<i32> {
        let bytes = self.read_bytes(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        Ok(i32::from_le_bytes(buf))
    }

    fn deserialize_u64(&mut self, _name: &str) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(buf))
    }

    fn deserialize_i64(&mut self, _name: &str) -> Result<i64> {
        let bytes = self.read_bytes(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(buf))
    }

    fn deserialize_u128(&mut self, _name: &str) -> Result<u128> {
        let bytes = self.read_bytes(16)?;
        let mut buf = [0u8; 16];
        buf.copy_from_slice(bytes);
        Ok(u128::from_le_bytes(buf))
    }

    fn deserialize_i128(&mut self, _name: &str) -> Result<i128> {
        let bytes = self.read_bytes(16)?;
        let mut buf = [0u8; 16];
        buf.copy_from_slice(bytes);
        Ok(i128::from_le_bytes(buf))
    }

    fn deserialize_f32(&mut self, _name: &str) -> Result<f32> {
        let bytes = self.read_bytes(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        Ok(f32::from_le_bytes(buf))
    }

    fn deserialize_f64(&mut self, _name: &str) -> Result<f64> {
        let bytes = self.read_bytes(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(f64::from_le_bytes(buf))
    }

    fn deserialize_bytes(&mut self, _name: &str, buffer: &mut [u8]) -> Result<usize> {
        // First read the length
        let len = self.deserialize_u32("")? as usize;
        if len > buffer.len() {
            return Err(Error::BufferTooSmall);
        }
        let bytes = self.read_bytes(len)?;
        buffer[..len].copy_from_slice(bytes);
        Ok(len)
    }

    #[cfg(feature = "alloc")]
    fn deserialize_string(&mut self, _name: &str) -> Result<String> {
        let len = self.deserialize_u32("")? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|_| Error::InvalidData)
    }

    #[cfg(feature = "alloc")]
    fn deserialize_vec<T: Deserialize>(&mut self, _name: &str) -> Result<alloc::vec::Vec<T>> {
        let len = self.deserialize_u32("")? as usize;
        let mut vec = alloc::vec::Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(self, "")?);
        }
        Ok(vec)
    }

    fn deserialize_array<T: Deserialize, const N: usize>(&mut self, _name: &str) -> Result<[T; N]> {
        let mut result = core::mem::MaybeUninit::<[T; N]>::uninit();
        let result_ptr = result.as_mut_ptr() as *mut T;
        
        for i in 0..N {
            unsafe {
                result_ptr.add(i).write(T::deserialize(self, "")?);
            }
        }
        
        Ok(unsafe { result.assume_init() })
    }
}

// Implementations for primitive types

impl Deserialize for bool {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_bool(name)
    }
}

impl Deserialize for u8 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u8(name)
    }
}

impl Deserialize for i8 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i8(name)
    }
}

impl Deserialize for u16 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u16(name)
    }
}

impl Deserialize for i16 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i16(name)
    }
}

impl Deserialize for u32 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u32(name)
    }
}

impl Deserialize for i32 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i32(name)
    }
}

impl Deserialize for u64 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u64(name)
    }
}

impl Deserialize for i64 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i64(name)
    }
}

impl Deserialize for u128 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u128(name)
    }
}

impl Deserialize for i128 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i128(name)
    }
}

impl Deserialize for f32 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_f32(name)
    }
}

impl Deserialize for f64 {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_f64(name)
    }
}

// String implementations
impl Deserialize for &str {
    fn deserialize<D: Deserializer>(_deserializer: &mut D, _name: &str) -> core::result::Result<Self, D::Error> {
        // Cannot deserialize into &str directly - use String instead
        Err(Error::InvalidData.into())
    }
}

#[cfg(feature = "alloc")]
impl Deserialize for String {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_string(name)
    }
}

// Array implementation
impl<T, const N: usize> Deserialize for [T; N] 
where 
    T: Deserialize 
{
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_array::<T, N>(name)
    }
}

// Vec implementation
#[cfg(feature = "alloc")]
impl<T> Deserialize for alloc::vec::Vec<T>
where
    T: Deserialize
{
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_vec::<T>(name)
    }
}

// Tuple implementations
impl<T1: Deserialize, T2: Deserialize> Deserialize for (T1, T2) 
where
    T1: Deserialize,
    T2: Deserialize
{
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        Ok((
            T1::deserialize(deserializer, name)?,
            T2::deserialize(deserializer, name)?,
        ))
    }
}

impl<T1: Deserialize, T2: Deserialize, T3: Deserialize> Deserialize for (T1, T2, T3)
where
    T1: Deserialize,
    T2: Deserialize,
    T3: Deserialize,
{
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        Ok((
            T1::deserialize(deserializer, name)?,
            T2::deserialize(deserializer, name)?,
            T3::deserialize(deserializer, name)?,
        ))
    }
}

// Option implementation
impl<T> Deserialize for Option<T> 
where T: Deserialize
{
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        let tag = deserializer.deserialize_u8(name)?;
        match tag {
            0 => Ok(None),
            1 => Ok(Some(T::deserialize(deserializer, name)?)),
            _ => Err(Error::InvalidData.into()),
        }
    }
}
