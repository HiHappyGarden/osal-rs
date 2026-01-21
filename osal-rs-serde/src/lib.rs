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

//! # OSAL-RS-Serde - Serialization/Deserialization Framework
//!
//! A lightweight, extensible serialization framework inspired by Serde,
//! designed for embedded systems and no-std environments.
//!
//! ## Overview
//!
//! This library provides a flexible serialization/deserialization framework that:
//! - Works in no-std environments
//! - Is memory-efficient
//! - Supports custom serialization formats
//! - Is extensible for any data type
//! - Can be used standalone in other projects
//!
//! ## Quick Start
//!
//! ### Using Traits Directly
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize, ByteSerializer, ByteDeserializer};
//!
//! struct SensorData {
//!     temperature: i16,
//!     humidity: u8,
//!     pressure: u32,
//! }
//!
//! impl Serialize for SensorData {
//!     fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
//!         serializer.serialize_i16(self.temperature)?;
//!         serializer.serialize_u8(self.humidity)?;
//!         serializer.serialize_u32(self.pressure)?;
//!         Ok(())
//!     }
//! }
//!
//! impl Deserialize for SensorData {
//!     fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
//!         Ok(SensorData {
//!             temperature: deserializer.deserialize_i16()?,
//!             humidity: deserializer.deserialize_u8()?,
//!             pressure: deserializer.deserialize_u32()?,
//!         })
//!     }
//! }
//!
//! // Usage
//! let data = SensorData { temperature: 25, humidity: 60, pressure: 1013 };
//! let mut buffer = [0u8; 32];
//! let mut serializer = ByteSerializer::new(&mut buffer);
//! data.serialize(&mut serializer).unwrap();
//!
//! // Deserialize
//! let mut deserializer = ByteDeserializer::new(&buffer);
//! let read_data = SensorData::deserialize(&mut deserializer).unwrap();
//! ```
//!
//! ## Supported Types
//!
//! - All primitive integer types (u8, i8, u16, i16, u32, i32, u64, i64, u128, i128)
//! - Floating point types (f32, f64)
//! - bool
//! - Arrays and slices
//! - Tuples
//! - Optional types (Option<T>)
//! - Result types
//!
//! ## Custom Serializers
//!
//! You can create custom serializers for different formats (JSON, MessagePack, etc.)
//! by implementing the `Serializer` and `Deserializer` traits.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod ser;
pub mod de;

use alloc::vec::Vec;
pub use error::{Error, Result};
pub use ser::{Serialize, Serializer, ByteSerializer};
pub use de::{Deserialize, Deserializer, ByteDeserializer};

// Re-export derive macros when the derive feature is enabled
#[cfg(feature = "derive")]
pub use osal_rs_serde_derive::{Serialize, Deserialize};

/// Serialize a value to a byte buffer.
///
/// This is a convenience function that handles serialization in a single call.
///
/// # Examples
///
/// ```ignore
/// use osal_rs_serde::to_bytes;
///
/// let value = 42u32;
/// let mut buffer = [0u8; 4];
/// let len = to_bytes(&value, &mut buffer).unwrap();
/// assert_eq!(len, 4);
/// ```
pub fn to_bytes<T>(value: &T, buffer: &mut [u8]) -> Result<usize> 
where 
    T: Serialize
{
    let mut serializer = ByteSerializer::new(buffer);
    value.serialize(&mut serializer)?;
    Ok(serializer.position())
}

/// Serialize a value to a dynamically sized byte vector.
///
/// This is a convenience function that handles serialization to a growable Vec<u8>.
///
/// # Examples
///
/// ```ignore
/// use osal_rs_serde::to_dyn_bytes;
/// use alloc::vec::Vec;
///
/// let value = 42u32;
/// let mut buffer = Vec::new();
/// let len = to_dyn_bytes(&value, &mut buffer).unwrap();
/// assert_eq!(len, 4);
/// ```
pub fn to_dyn_bytes<T>(value: &T, buffer: &mut Vec<u8>) -> Result<usize> 
where 
    T: Serialize
{
    let mut serializer = ByteSerializer::new(buffer);
    value.serialize(&mut serializer)?;
    Ok(serializer.position())
}

/// Deserialize a value from a byte buffer.
///
/// This is a convenience function that handles deserialization in a single call.
///
/// # Examples
///
/// ```ignore
/// use osal_rs_serde::from_bytes;
///
/// let buffer = [42u8, 0, 0, 0];
/// let value: u32 = from_bytes(&buffer).unwrap();
/// assert_eq!(value, 42);
/// ```
pub fn from_bytes<T>(buffer: &[u8]) -> Result<T> 
where 
    T: Deserialize
{
    let mut deserializer = ByteDeserializer::new(buffer);
    T::deserialize(&mut deserializer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_serialization() {
        let mut buffer = [0u8; 4];
        let len = to_bytes(&42u32, &mut buffer).unwrap();
        assert_eq!(len, 4);
        
        let value: u32 = from_bytes(&buffer).unwrap();
        assert_eq!(value, 42);
    }
}
