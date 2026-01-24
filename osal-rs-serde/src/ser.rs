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

//! Serialization traits and implementations.
//!
//! This module provides the core serialization functionality for osal-rs-serde.
//! It defines the [`Serialize`] trait for types that can be serialized and the
//! [`Serializer`] trait for implementing custom serialization formats.
//!
//! # Overview
//!
//! - [`Serialize`]: Trait implemented by types that can be serialized
//! - [`Serializer`]: Trait for implementing custom serialization formats  
//! - [`ByteSerializer`]: Concrete implementation that writes little-endian binary data
//!
//! # Usage with Derive Macro
//!
//! The easiest way to implement serialization is using the derive macro:
//!
//! ```ignore
//! use osal_rs_serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct SensorData {
//!     temperature: i16,
//!     humidity: u8,
//!     pressure: u32,
//! }
//! ```
//!
//! # Manual Implementation
//!
//! For custom serialization logic, implement the trait manually:
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Serializer};
//!
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! impl Serialize for Point {
//!     fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
//!         serializer.serialize_i32("x", self.x)?;
//!         serializer.serialize_i32("y", self.y)?;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Supported Types
//!
//! The serialization framework supports:
//! - All primitive types (bool, integers, floats)
//! - Arrays `[T; N]` where T: Serialize
//! - Tuples (up to 3 elements)
//! - `Option<T>` where T: Serialize
//! - `Vec<T>` where T: Serialize (requires `alloc`)
//! - `String` and `&str` (requires `alloc` for String)
//! - Custom types implementing `Serialize`

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::error::{Error, Result};

/// Trait for types that can be serialized.
///
/// This trait should be implemented (or derived) for any type that needs to be serialized.
/// The implementation defines how the type should be written to a serializer.
///
/// # Derive Macro
///
/// The easiest way to implement this trait is using the derive macro (requires `derive` feature):
///
/// ```ignore
/// use osal_rs_serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     id: u32,
///     enabled: bool,
///     timeout: Option<u16>,
/// }
/// ```
///
/// # Manual Implementation
///
/// For custom serialization logic or types not supported by the derive macro:
///
/// ```ignore
/// use osal_rs_serde::{Serialize, Serializer};
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl Serialize for Point {
///     fn serialize<S: Serializer>(&self, serializer: &mut S) -> core::result::Result<(), S::Error> {
///         serializer.serialize_i32("x", self.x)?;
///         serializer.serialize_i32("y", self.y)?;
///         Ok(())
///     }
/// }
/// ```
///
/// # Built-in Implementations
///
/// This trait is already implemented for:
/// - All primitive types (bool, u8-u128, i8-i128, f32, f64)
/// - Arrays `[T; N]` where T: Serialize
/// - Tuples (T1, T2) and (T1, T2, T3) where all T: Serialize
/// - `Option<T>` where T: Serialize
/// - `Vec<T>` where T: Serialize (requires `alloc`)
/// - `String` and `&str` (requires `alloc` for String)
pub trait Serialize {
    /// Serialize this value using the given serializer.
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error>
    where
        S: Serializer;
}

/// Trait that defines how to serialize various types.
///
/// Implementations of this trait determine the output format.
/// For example, `ByteSerializer` writes data in little-endian binary format.
pub trait Serializer: Sized {
    /// The error type that can be returned during serialization.
    type Error: From<Error>;

    /// Serialize a `bool` value.
    fn serialize_bool(&mut self, name: &str, v: bool) -> core::result::Result<(), Self::Error>;

    /// Serialize a `u8` value.
    fn serialize_u8(&mut self, name: &str, v: u8) -> core::result::Result<(), Self::Error>;

    /// Serialize an `i8` value.
    fn serialize_i8(&mut self, name: &str, v: i8) -> core::result::Result<(), Self::Error>;

    /// Serialize a `u16` value.
    fn serialize_u16(&mut self, name: &str, v: u16) -> core::result::Result<(), Self::Error>;

    /// Serialize an `i16` value.
    fn serialize_i16(&mut self, name: &str, v: i16) -> core::result::Result<(), Self::Error>;

    /// Serialize a `u32` value.
    fn serialize_u32(&mut self, name: &str, v: u32) -> core::result::Result<(), Self::Error>;

    /// Serialize an `i32` value.
    fn serialize_i32(&mut self, name: &str, v: i32) -> core::result::Result<(), Self::Error>;

    /// Serialize a `u64` value.
    fn serialize_u64(&mut self, name: &str, v: u64) -> core::result::Result<(), Self::Error>;

    /// Serialize an `i64` value.
    fn serialize_i64(&mut self, name: &str, v: i64) -> core::result::Result<(), Self::Error>;

    /// Serialize a `u128` value.
    fn serialize_u128(&mut self, name: &str, v: u128) -> core::result::Result<(), Self::Error>;

    /// Serialize an `i128` value.
    fn serialize_i128(&mut self, name: &str, v: i128) -> core::result::Result<(), Self::Error>;

    /// Serialize an `f32` value.
    fn serialize_f32(&mut self, name: &str, v: f32) -> core::result::Result<(), Self::Error>;

    /// Serialize an `f64` value.
    fn serialize_f64(&mut self, name: &str, v: f64) -> core::result::Result<(), Self::Error>;

    /// Serialize a byte slice.
    fn serialize_bytes(&mut self, name: &str, v: &[u8]) -> core::result::Result<(), Self::Error>;

    /// Serialize a string.
    fn serialize_string(&mut self, name: &str, v: &String) -> core::result::Result<(), Self::Error>;

    /// Serialize a string slice.
    fn serialize_str(&mut self, name: &str, v: &str) -> core::result::Result<(), Self::Error>;

    /// Serialize a vector of serializable items.
    fn serialize_vec<T>(&mut self, name: &str, v: &alloc::vec::Vec<T>) -> core::result::Result<(), Self::Error>
    where
        T: Serialize;

    /// Serialize an array of serializable items.
    fn serialize_array<T>(&mut self, name: &str, v: &[T]) -> core::result::Result<(), Self::Error>
    where
        T: Serialize;

    /// Begin serializing a struct with the given name and number of fields.
    /// Default implementation does nothing (suitable for binary formats).
    fn serialize_struct_start(&mut self, _name: &str, _len: usize) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    /// Serialize a struct field with name and value.
    /// Default implementation just serializes the value.
    fn serialize_field<T>(&mut self, name: &str, value: &T) -> core::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(name, self)
    }

    /// End serializing a struct.
    /// Default implementation does nothing (suitable for binary formats).
    fn serialize_struct_end(&mut self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }
}

/// A serializer that writes data to a byte buffer in little-endian format.
///
/// This is a concrete implementation of the `Serializer` trait that writes
/// binary data in a compact, little-endian format. This is the default serializer
/// used by the [`crate::to_bytes`] convenience function.
///
/// # Format
///
/// - All integers are written in little-endian byte order
/// - Floating-point numbers use IEEE 754 representation
/// - `bool` is written as a single byte (0 or 1)
/// - `Option<T>`: 1 byte tag (0=None, 1=Some) followed by T if Some
/// - Arrays: Elements serialized sequentially (no length prefix)
/// - Tuples: Elements serialized sequentially
/// - Strings/Vec: u32 length prefix followed by data
///
/// # Examples
///
/// ## Basic Usage
///
/// ```ignore
/// use osal_rs_serde::{ByteSerializer, Serializer, Serialize};
///
/// let mut buffer = [0u8; 16];
/// let mut serializer = ByteSerializer::new(&mut buffer);
///
/// serializer.serialize_u32("", 42).unwrap();
/// serializer.serialize_bool("", true).unwrap();
/// serializer.serialize_i16("", -100).unwrap();
///
/// let len = serializer.position();
/// println!("Serialized {} bytes", len);
/// ```
///
/// ## With Structs
///
/// ```ignore
/// use osal_rs_serde::{ByteSerializer, Serialize};
///
/// #[derive(Serialize)]
/// struct Message {
///     id: u32,
///     value: i16,
/// }
///
/// let msg = Message { id: 100, value: -50 };
/// let mut buffer = [0u8; 32];
/// let mut serializer = ByteSerializer::new(&mut buffer);
/// msg.serialize(&mut serializer).unwrap();
/// ```
///
/// # Memory Layout
///
/// The serializer writes data sequentially without padding or alignment:
///
/// ```text
/// struct Data { a: u16, b: u32 }
/// Memory: [a_lo, a_hi, b0, b1, b2, b3]
/// ```
pub struct ByteSerializer<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> ByteSerializer<'a> {
    /// Create a new ByteSerializer with the given buffer.
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Get the current position in the buffer.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Write bytes to the buffer.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        if self.position + bytes.len() > self.buffer.len() {
            return Err(Error::BufferTooSmall);
        }
        self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
        Ok(())
    }
}

impl<'a> Serializer for ByteSerializer<'a> {
    type Error = Error;

    fn serialize_bool(&mut self, _name: &str, v: bool) -> Result<()> {
        self.serialize_u8("", if v { 1 } else { 0 })
    }

    fn serialize_u8(&mut self, _name: &str, v: u8) -> Result<()> {
        self.write_bytes(&[v])
    }

    fn serialize_i8(&mut self, _name: &str, v: i8) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_u16(&mut self, _name: &str, v: u16) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_i16(&mut self, _name: &str, v: i16) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_u32(&mut self, _name: &str, v: u32) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_i32(&mut self, _name: &str, v: i32) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_u64(&mut self, _name: &str, v: u64) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_i64(&mut self, _name: &str, v: i64) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_u128(&mut self, _name: &str, v: u128) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_i128(&mut self, _name: &str, v: i128) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_f32(&mut self, _name: &str, v: f32) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_f64(&mut self, _name: &str, v: f64) -> Result<()> {
        self.write_bytes(&v.to_le_bytes())
    }

    fn serialize_bytes(&mut self, _name: &str, v: &[u8]) -> Result<()> {
        // First write the length as u32
        self.serialize_u32("", v.len() as u32)?;
        self.write_bytes(v)
    }

    fn serialize_string(&mut self, name: &str, v: &String) -> core::result::Result<(), Self::Error> {
        self.serialize_str(name, v.as_str())
    }

    fn serialize_str(&mut self, name: &str, v: &str) -> core::result::Result<(), Self::Error> {
        self.serialize_bytes(name, v.as_bytes())
    }

    fn serialize_vec<T>(&mut self, name: &str, v: &alloc::vec::Vec<T>) -> core::result::Result<(), Self::Error> 
    where
        T: Serialize {
        // First write the length as u32
        self.serialize_u32(name, v.len() as u32)?;
        for item in v.iter() {
            item.serialize(name, self)?;
        }
        Ok(())
    }

    /// Serialize an array of serializable items.
    fn serialize_array<T>(&mut self, name: &str, v: &[T]) -> core::result::Result<(), Self::Error> 
    where
        T: Serialize {
        for item in v.iter() {
            item.serialize(name, self)?;
        }
        Ok(())
    }

}

// Implementations for primitive types

impl Serialize for bool {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer
    {
    
        serializer.serialize_bool(name, *self)
    }
}

impl Serialize for u8 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_u8(name, *self)
    }
}

impl Serialize for i8 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_i8(name, *self)
    }
}

impl Serialize for u16 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_u16(name, *self)
    }
}

impl Serialize for i16 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_i16(name, *self)
    }
}

impl Serialize for u32 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_u32(name, *self)
    }
}

impl Serialize for i32 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_i32(name, *self)
    }
}

impl Serialize for u64 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_u64(name, *self)
    }
}

impl Serialize for i64 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_i64(name, *self)
    }
}

impl Serialize for u128 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_u128(name, *self)
    }
}

impl Serialize for i128 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_i128(name, *self)
    }
}

impl Serialize for f32 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_f32(name, *self)
    }
}

impl Serialize for f64 {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_f64(name, *self)
    }
}

// String implementations
impl Serialize for &str {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_str(name, self)
    }
}

#[cfg(feature = "alloc")]
impl Serialize for String {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        serializer.serialize_string(name, self)
    }
}

// Array implementation
impl<T: Serialize, const N: usize> Serialize for [T; N] {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer
    {
        for item in self.iter() {
            item.serialize(name, serializer)?;
        }
        Ok(())
    }
}

// Tuple implementations
impl<T1: Serialize, T2: Serialize> Serialize for (T1, T2) {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        self.0.serialize(name, serializer)?;
        self.1.serialize(name, serializer)?;
        Ok(())
    }
}

impl<T1: Serialize, T2: Serialize, T3: Serialize> Serialize for (T1, T2, T3) {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer
    {
        self.0.serialize(name, serializer)?;
        self.1.serialize(name, serializer)?;
        self.2.serialize(name, serializer)?;
        Ok(())
    }
}

// Option implementation
impl<T: Serialize> Serialize for Option<T> {
    fn serialize<S>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> 
    where
        S: Serializer,
    {
        match self {
            Some(value) => {
                serializer.serialize_u8(name, 1)?;
                value.serialize(name, serializer)?;
            }
            None => {
                serializer.serialize_u8(name, 0)?;
            }
        }
        Ok(())
    }
}
