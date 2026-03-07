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
/// ```ignore
/// use osal_rs::traits::BytesHasLen;
/// 
/// let data: [u8; 4] = [1, 2, 3, 4];
/// assert_eq!(data.len(), 4);
/// assert!(!data.is_empty());
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
/// ```ignore
/// use osal_rs::traits::Serialize;
/// 
/// struct SensorData {
///     temperature: i16,
///     humidity: u8,
/// }
/// 
/// impl Serialize for SensorData {
///     fn to_bytes(&self) -> &[u8] {
///         // Convert struct to bytes
///         // Safety: ensure proper memory layout
///         unsafe {
///             core::slice::from_raw_parts(
///                 self as *const Self as *const u8,
///                 core::mem::size_of::<Self>()
///             )
///         }
///     }
/// }
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
/// ```ignore
/// use osal_rs::traits::Deserialize;
/// use osal_rs::utils::Result;
/// 
/// struct SensorData {
///     temperature: i16,
///     humidity: u8,
/// }
/// 
/// impl Deserialize for SensorData {
///     fn from_bytes(bytes: &[u8]) -> Result<Self> {
///         if bytes.len() < 3 {
///             return Err(Error::InvalidParameter);
///         }
///         Ok(SensorData {
///             temperature: i16::from_le_bytes([bytes[0], bytes[1]]),
///             humidity: bytes[2],
///         })
///     }
/// }
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



