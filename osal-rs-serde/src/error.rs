/***************************************************************************
 *
 * osal-rs-serde
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
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 *
 ***************************************************************************/

//! Error types for serialization and deserialization operations.

use core::fmt::{self, Display, Debug};

/// Result type for serialization/deserialization operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Error types that can occur during serialization or deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Buffer is too small for the operation
    BufferTooSmall,
    
    /// Unexpected end of data
    UnexpectedEof,
    
    /// Invalid data format encountered
    InvalidData,
    
    /// Type mismatch during deserialization
    TypeMismatch,
    
    /// Value out of valid range
    OutOfRange,
    
    /// Custom error with a static message
    Custom(&'static str),
    
    /// Unsupported operation
    Unsupported,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BufferTooSmall => write!(f, "Buffer too small"),
            Error::UnexpectedEof => write!(f, "Unexpected end of data"),
            Error::InvalidData => write!(f, "Invalid data format"),
            Error::TypeMismatch => write!(f, "Type mismatch"),
            Error::OutOfRange => write!(f, "Value out of range"),
            Error::Custom(msg) => write!(f, "Custom error: {}", msg),
            Error::Unsupported => write!(f, "Unsupported operation"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
