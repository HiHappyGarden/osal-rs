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

//! Byte conversion traits for serialization and deserialization.
//!
//! This module provides traits for converting types to and from byte arrays,
//! enabling type-safe serialization for queue and communication operations.

#[cfg(feature = "serde")]
use osal_rs_serde::Serialize;

#[cfg(not(feature = "serde"))]
use crate::utils::Result;

/// Trait for types that have a known byte length.
///
/// Used to determine the size of data structures when working with byte arrays.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
///
/// struct Reading {
///     temperature: i16,
///     humidity: u8,
/// }
///
/// impl BytesHasLen for Reading {
///     fn len(&self) -> usize {
///         core::mem::size_of::<i16>() + core::mem::size_of::<u8>()
///     }
/// }
///
/// let reading = Reading { temperature: 235, humidity: 65 };
/// assert_eq!(reading.len(), 3);
/// assert!(!reading.is_empty());
/// ```
pub trait BytesHasLen {
    /// Returns the length in bytes.
    ///
    /// # Returns
    ///
    /// Number of bytes in the data structure
    fn len(&self) -> usize;

    /// Returns `true` if the length is zero.
    ///
    /// # Returns
    ///
    /// `true` if empty, `false` otherwise
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Automatic implementation of `BytesHasLen` for fixed-size arrays.
///
/// This allows arrays of types implementing `Serialize` to automatically
/// report their size.
impl<T, const N: usize> BytesHasLen for [T; N] 
where 
    T: Serialize + Sized {
    fn len(&self) -> usize {
        N
    }
}

/// Trait for converting types to byte slices.
///
/// Enables serialization of structured data for transmission through
/// queues or other byte-oriented communication channels.
///
/// # Safety
///
/// When implementing this trait, ensure that the returned byte slice
/// is a valid representation of the type and lives at least as long
/// as the value itself.
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
///
/// // `repr(C)` pins the field order, so the bytes handed out below are a
/// // stable representation rather than whatever layout the compiler picks.
/// #[repr(C)]
/// struct SensorData {
///     temperature: i16,
///     humidity: u8,
/// }
///
/// impl Serialize for SensorData {
///     fn to_bytes(&self) -> &[u8] {
///         // Safety: the slice borrows `self`, so it cannot outlive it, and
///         // `size_of::<Self>()` bytes starting at `self` are always readable.
///         unsafe {
///             core::slice::from_raw_parts(
///                 self as *const Self as *const u8,
///                 core::mem::size_of::<Self>()
///             )
///         }
///     }
/// }
///
/// let data = SensorData { temperature: 235, humidity: 65 };
/// let bytes = data.to_bytes();
///
/// assert_eq!(bytes.len(), core::mem::size_of::<SensorData>());
/// assert_eq!(&bytes[..2], &235i16.to_ne_bytes());
/// assert_eq!(bytes[2], 65);
/// ```
#[cfg(not(feature = "serde"))]
pub trait Serialize {
    /// Converts this value to a byte slice.
    ///
    /// # Returns
    ///
    /// A reference to the byte representation of this value
    fn to_bytes(&self) -> &[u8];
}

/// Trait for deserializing types from byte slices.
///
/// Enables reconstruction of structured data from byte arrays received
/// from queues or communication channels.
///
/// # Errors
///
/// Implementations should return an error if:
/// - The byte slice is too small or too large
/// - The data is invalid or corrupted
/// - The conversion fails for any other reason
///
/// # Examples
///
/// ```
/// use osal_rs::os::*;
/// use osal_rs::utils::{Error, Result};
///
/// #[derive(Debug, PartialEq)]
/// struct SensorData {
///     temperature: i16,
///     humidity: u8,
/// }
///
/// impl Deserialize for SensorData {
///     fn from_bytes(bytes: &[u8]) -> Result<Self> {
///         if bytes.len() < 3 {
///             return Err(Error::OutOfIndex);
///         }
///         Ok(SensorData {
///             temperature: i16::from_le_bytes([bytes[0], bytes[1]]),
///             humidity: bytes[2],
///         })
///     }
/// }
///
/// let data = SensorData::from_bytes(&[0xEB, 0x00, 65]).unwrap();
/// assert_eq!(data, SensorData { temperature: 235, humidity: 65 });
///
/// // Too short to hold both fields.
/// assert!(SensorData::from_bytes(&[0xEB, 0x00]).is_err());
/// ```
#[cfg(not(feature = "serde"))]
pub trait Deserialize: Sized
where
    Self: Sized {
    /// Creates a new instance from a byte slice.
    ///
    /// # Parameters
    ///
    /// * `bytes` - The byte slice to deserialize from
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully deserialized value
    /// * `Err(Error)` - Deserialization failed (invalid data, wrong size, etc.)
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}



