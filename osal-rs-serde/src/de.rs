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

//! Deserialization trait and implementation.

use crate::error::{Error, Result};

/// Trait for types that can be deserialized.
///
/// This trait should be implemented for any type that needs to be deserialized.
/// The implementation defines how the type should be read from a deserializer.
///
/// # Examples
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
///     fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
///         Ok(Point {
///             x: deserializer.deserialize_i32()?,
///             y: deserializer.deserialize_i32()?,
///         })
///     }
/// }
/// ```
pub trait Deserialize: Sized {
    /// Deserialize this value using the given deserializer.
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error>;
}

/// Trait that defines how to deserialize various types.
///
/// Implementations of this trait determine how to read data from a specific format.
/// For example, `ByteDeserializer` reads data in little-endian binary format.
pub trait Deserializer: Sized {
    /// The error type that can be returned during deserialization.
    type Error: From<Error>;

    /// Deserialize a `bool` value.
    fn deserialize_bool(&mut self) -> core::result::Result<bool, Self::Error>;

    /// Deserialize a `u8` value.
    fn deserialize_u8(&mut self) -> core::result::Result<u8, Self::Error>;

    /// Deserialize an `i8` value.
    fn deserialize_i8(&mut self) -> core::result::Result<i8, Self::Error>;

    /// Deserialize a `u16` value.
    fn deserialize_u16(&mut self) -> core::result::Result<u16, Self::Error>;

    /// Deserialize an `i16` value.
    fn deserialize_i16(&mut self) -> core::result::Result<i16, Self::Error>;

    /// Deserialize a `u32` value.
    fn deserialize_u32(&mut self) -> core::result::Result<u32, Self::Error>;

    /// Deserialize an `i32` value.
    fn deserialize_i32(&mut self) -> core::result::Result<i32, Self::Error>;

    /// Deserialize a `u64` value.
    fn deserialize_u64(&mut self) -> core::result::Result<u64, Self::Error>;

    /// Deserialize an `i64` value.
    fn deserialize_i64(&mut self) -> core::result::Result<i64, Self::Error>;

    /// Deserialize a `u128` value.
    fn deserialize_u128(&mut self) -> core::result::Result<u128, Self::Error>;

    /// Deserialize an `i128` value.
    fn deserialize_i128(&mut self) -> core::result::Result<i128, Self::Error>;

    /// Deserialize an `f32` value.
    fn deserialize_f32(&mut self) -> core::result::Result<f32, Self::Error>;

    /// Deserialize an `f64` value.
    fn deserialize_f64(&mut self) -> core::result::Result<f64, Self::Error>;

    /// Deserialize bytes into a buffer. Returns the number of bytes read.
    fn deserialize_bytes(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, Self::Error>;
}

/// A deserializer that reads data from a byte buffer in little-endian format.
///
/// This is a concrete implementation of the `Deserializer` trait that reads
/// binary data in a compact, little-endian format.
///
/// # Examples
///
/// ```ignore
/// use osal_rs_serde::{ByteDeserializer, Deserializer};
///
/// let buffer = [42u8, 0, 0, 0, 100, 0, 0, 0];
/// let mut deserializer = ByteDeserializer::new(&buffer);
///
/// let value1 = deserializer.deserialize_u32().unwrap();
/// let value2 = deserializer.deserialize_u32().unwrap();
/// assert_eq!(value1, 42);
/// assert_eq!(value2, 100);
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

    fn deserialize_bool(&mut self) -> Result<bool> {
        Ok(self.deserialize_u8()? != 0)
    }

    fn deserialize_u8(&mut self) -> Result<u8> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    fn deserialize_i8(&mut self) -> Result<i8> {
        let bytes = self.read_bytes(1)?;
        Ok(i8::from_le_bytes([bytes[0]]))
    }

    fn deserialize_u16(&mut self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(buf))
    }

    fn deserialize_i16(&mut self) -> Result<i16> {
        let bytes = self.read_bytes(2)?;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(bytes);
        Ok(i16::from_le_bytes(buf))
    }

    fn deserialize_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(buf))
    }

    fn deserialize_i32(&mut self) -> Result<i32> {
        let bytes = self.read_bytes(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        Ok(i32::from_le_bytes(buf))
    }

    fn deserialize_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(buf))
    }

    fn deserialize_i64(&mut self) -> Result<i64> {
        let bytes = self.read_bytes(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(buf))
    }

    fn deserialize_u128(&mut self) -> Result<u128> {
        let bytes = self.read_bytes(16)?;
        let mut buf = [0u8; 16];
        buf.copy_from_slice(bytes);
        Ok(u128::from_le_bytes(buf))
    }

    fn deserialize_i128(&mut self) -> Result<i128> {
        let bytes = self.read_bytes(16)?;
        let mut buf = [0u8; 16];
        buf.copy_from_slice(bytes);
        Ok(i128::from_le_bytes(buf))
    }

    fn deserialize_f32(&mut self) -> Result<f32> {
        let bytes = self.read_bytes(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        Ok(f32::from_le_bytes(buf))
    }

    fn deserialize_f64(&mut self) -> Result<f64> {
        let bytes = self.read_bytes(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        Ok(f64::from_le_bytes(buf))
    }

    fn deserialize_bytes(&mut self, buffer: &mut [u8]) -> Result<usize> {
        // First read the length
        let len = self.deserialize_u32()? as usize;
        if len > buffer.len() {
            return Err(Error::BufferTooSmall);
        }
        let bytes = self.read_bytes(len)?;
        buffer[..len].copy_from_slice(bytes);
        Ok(len)
    }
}

// Implementations for primitive types

impl Deserialize for bool {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_bool()
    }
}

impl Deserialize for u8 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u8()
    }
}

impl Deserialize for i8 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i8()
    }
}

impl Deserialize for u16 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u16()
    }
}

impl Deserialize for i16 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i16()
    }
}

impl Deserialize for u32 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u32()
    }
}

impl Deserialize for i32 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i32()
    }
}

impl Deserialize for u64 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u64()
    }
}

impl Deserialize for i64 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i64()
    }
}

impl Deserialize for u128 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_u128()
    }
}

impl Deserialize for i128 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_i128()
    }
}

impl Deserialize for f32 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_f32()
    }
}

impl Deserialize for f64 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        deserializer.deserialize_f64()
    }
}

// Array implementation
impl<T, const N: usize> Deserialize for [T; N] 
where 
    T: Deserialize 
{
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        let mut result = core::mem::MaybeUninit::<[T; N]>::uninit();
        let result_ptr = result.as_mut_ptr() as *mut T;
        
        for i in 0..N {
            unsafe {
                result_ptr.add(i).write(T::deserialize(deserializer)?);
            }
        }
        
        Ok(unsafe { result.assume_init() })
    }
}

// Tuple implementations
impl<T1: Deserialize, T2: Deserialize> Deserialize for (T1, T2) 
where
    T1: Deserialize,
    T2: Deserialize
{
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        Ok((
            T1::deserialize(deserializer)?,
            T2::deserialize(deserializer)?,
        ))
    }
}

impl<T1: Deserialize, T2: Deserialize, T3: Deserialize> Deserialize for (T1, T2, T3)
where
    T1: Deserialize,
    T2: Deserialize,
    T3: Deserialize,
{
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        Ok((
            T1::deserialize(deserializer)?,
            T2::deserialize(deserializer)?,
            T3::deserialize(deserializer)?,
        ))
    }
}

// Option implementation
impl<T> Deserialize for Option<T> 
where T: Deserialize
{
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        let tag = deserializer.deserialize_u8()?;
        match tag {
            0 => Ok(None),
            1 => Ok(Some(T::deserialize(deserializer)?)),
            _ => Err(Error::InvalidData.into()),
        }
    }
}
