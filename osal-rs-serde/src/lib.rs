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
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
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
//! - **No-std compatible**: Works perfectly in bare-metal embedded environments
//! - **Memory-efficient**: Optimized for resource-constrained systems  
//! - **Derive macro support**: `#[derive(Serialize, Deserialize)]` for automatic implementation
//! - **Extensible**: Create custom serializers for any format (binary, JSON, MessagePack, etc.)
//! - **Type-safe**: Leverages Rust's type system for compile-time guarantees
//! - **Standalone**: Can be used in any project, not just with osal-rs
//!
//! ## Supported Types
//!
//! - **Primitives**: `bool`, `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `f32`, `f64`
//! - **Compound types**: Arrays `[T; N]`, tuples `(T1, T2, T3)` (up to 3 elements), `Option<T>`
//! - **Collections**: `Vec<T>`, byte slices, strings (with `alloc` feature)
//! - **Custom types**: Any struct implementing `Serialize`/`Deserialize`
//! - **Nested structs**: Full support for struct composition
//!
//! ## Memory Layout
//!
//! The default `ByteSerializer` uses little-endian binary format:
//! - Primitives: Native sizes (1, 2, 4, 8, or 16 bytes)
//! - `bool`: 1 byte (0 or 1)
//! - `Option<T>`: 1 byte tag + sizeof(T) if Some, 1 byte if None
//! - Arrays `[T; N]`: sizeof(T) * N (no length prefix)
//! - Tuples: concatenation of all elements
//! - Structs: concatenation of all fields in declaration order
//!
//! ## Quick Start
//!
//! ### Using Derive Macros (Recommended)
//!
//! #### Basic Struct Example
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};
//!
//! #[derive(Serialize, Deserialize)]
//! struct SensorData {
//!     temperature: i16,
//!     humidity: u8,
//!     pressure: u32,
//! }
//!
//! fn main() {
//!     let data = SensorData {
//!         temperature: 25,
//!         humidity: 60,
//!         pressure: 1013,
//!     };
//!
//!     // Serialize
//!     let mut buffer = [0u8; 32];
//!     let len = to_bytes(&data, &mut buffer).unwrap();
//!     println!("Serialized {} bytes", len);
//!
//!     // Deserialize
//!     let read_data: SensorData = from_bytes(&buffer[..len]).unwrap();
//!     println!("Temperature: {}", read_data.temperature);
//! }
//! ```
//!
//! #### Struct with Optional Fields
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     device_id: u32,
//!     name: Option<u8>,      // Optional device name code
//!     enabled: bool,
//!     timeout: Option<u16>,  // Optional timeout in ms
//! }
//!
//! fn main() {
//!     let config = Config {
//!         device_id: 100,
//!         name: Some(42),
//!         enabled: true,
//!         timeout: None,
//!     };
//!
//!     let mut buffer = [0u8; 64];
//!     let len = to_bytes(&config, &mut buffer).unwrap();
//!     let decoded: Config = from_bytes(&buffer[..len]).unwrap();
//! }
//! ```
//!
//! #### Struct with Arrays and Tuples
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};
//!
//! #[derive(Serialize, Deserialize)]
//! struct TelemetryPacket {
//!     timestamp: u64,
//!     coordinates: (i32, i32, i32),  // x, y, z
//!     samples: [u16; 8],              // 8 sensor readings
//!     status: u8,
//! }
//!
//! fn main() {
//!     let packet = TelemetryPacket {
//!         timestamp: 1642857600,
//!         coordinates: (100, 200, 50),
//!         samples: [10, 20, 30, 40, 50, 60, 70, 80],
//!         status: 0xFF,
//!     };
//!
//!     let mut buffer = [0u8; 128];
//!     let len = to_bytes(&packet, &mut buffer).unwrap();
//!     println!("Telemetry packet: {} bytes", len);
//! }
//! ```
//!
//! #### Nested Structs
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Location {
//!     latitude: i32,
//!     longitude: i32,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct Device {
//!     id: u32,
//!     battery: u8,
//!     location: Location,
//!     active: bool,
//! }
//!
//! fn main() {
//!     let device = Device {
//!         id: 42,
//!         battery: 85,
//!         location: Location {
//!             latitude: 45500000,
//!             longitude: 9200000,
//!         },
//!         active: true,
//!     };
//!
//!     let mut buffer = [0u8; 64];
//!     let len = to_bytes(&device, &mut buffer).unwrap();
//!     let decoded: Device = from_bytes(&buffer[..len]).unwrap();
//!     println!("Device at {}, {}", 
//!              decoded.location.latitude, 
//!              decoded.location.longitude);
//! }
//! ```
//!
//! #### Complex Embedded System Example
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct MotorControl {
//!     motor_id: u8,
//!     speed: i16,        // -1000 to 1000
//!     direction: bool,   // true = forward, false = reverse
//!     current: u16,      // mA
//! }
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct RobotState {
//!     timestamp: u64,
//!     motors: [MotorControl; 4],  // 4 motors
//!     battery_voltage: u16,        // mV
//!     temperature: i8,             // °C
//!     error_flags: u32,
//! }
//!
//! fn main() {
//!     let state = RobotState {
//!         timestamp: 1000000,
//!         motors: [
//!             MotorControl { motor_id: 0, speed: 500, direction: true, current: 1200 },
//!             MotorControl { motor_id: 1, speed: 500, direction: true, current: 1150 },
//!             MotorControl { motor_id: 2, speed: -300, direction: false, current: 800 },
//!             MotorControl { motor_id: 3, speed: -300, direction: false, current: 850 },
//!         ],
//!         battery_voltage: 12400,  // 12.4V
//!         temperature: 35,
//!         error_flags: 0,
//!     };
//!
//!     let mut buffer = [0u8; 256];
//!     let len = to_bytes(&state, &mut buffer).unwrap();
//!     println!("Robot state serialized: {} bytes", len);
//!     
//!     // Deserialize and check
//!     let decoded: RobotState = from_bytes(&buffer[..len]).unwrap();
//!     assert_eq!(state, decoded);
//!     println!("Battery: {}mV, Temp: {}°C", 
//!              decoded.battery_voltage, 
//!              decoded.temperature);
//! }
//! ```
//!
//! ### Manual Implementation (For Custom Behavior)
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize, Serializer, Deserializer};
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
//! ## Integration with OSAL-RS
//!
//! Perfect for inter-task communication using queues:
//!
//! ```ignore
//! use osal_rs::os::{Queue, QueueFn};
//! use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Message {
//!     command: u8,
//!     data: [u16; 4],
//! }
//!
//! fn sender(queue: &Queue) {
//!     let msg = Message { command: 0x42, data: [1, 2, 3, 4] };
//!     let mut buffer = [0u8; 32];
//!     let len = to_bytes(&msg, &mut buffer).unwrap();
//!     queue.post(&buffer[..len], 100).unwrap();
//! }
//!
//! fn receiver(queue: &Queue) {
//!     let mut buffer = [0u8; 32];
//!     queue.fetch(&mut buffer, 100).unwrap();
//!     let msg: Message = from_bytes(&buffer).unwrap();
//! }
//! ```
//!
//! ## Supported Types
//!
//! - **Primitives**: All integer types (u8-u128, i8-i128), f32, f64, bool
//! - **Compound**: Arrays `[T; N]`, tuples (up to 3 elements), `Option<T>`
//! - **Collections**: `Vec<T>`, `String` (requires `alloc` feature)
//! - **Custom**: Any type implementing `Serialize`/`Deserialize`
//! - **Nested**: Full support for nested structs
//!
//! ## Creating Custom Serializers
//!
//! You can create custom serializers for different formats (JSON, MessagePack, CBOR, etc.)
//! by implementing the `Serializer` and `Deserializer` traits:
//!
//! ```ignore
//! use osal_rs_serde::{Serializer, Error};
//!
//! struct JsonSerializer<'a> {
//!     buffer: &'a mut [u8],
//!     position: usize,
//! }
//!
//! impl<'a> Serializer for JsonSerializer<'a> {
//!     type Error = Error;
//!     
//!     fn serialize_u32(&mut self, name: &str, v: u32) -> Result<(), Self::Error> {
//!         // Write JSON format: "name": value
//!         // Implementation here...
//!         Ok(())
//!     }
//!     
//!     // Implement other serialize_* methods...
//! }
//! ```
//!
//! See `examples/custom_serializer.rs` for a complete implementation example.
//!
//! ## Performance & Binary Size
//!
//! - **Zero-copy**: Reads/writes directly to/from buffers
//! - **No allocations**: Works entirely with stack buffers (or Vec with `alloc`)
//! - **Predictable**: Buffer size calculable at compile time
//! - **Small code size**: Minimal overhead, optimized for embedded targets
//!
//! ## Features
//!
//! - `default`: Includes `alloc` feature
//! - `alloc`: Enables Vec, String support
//! - `std`: Enables standard library (error traits, etc.)
//! - `derive`: Enables `#[derive(Serialize, Deserialize)]` macros (**recommended**)
//!
//! ## Examples
//!
//! The `examples/` directory contains complete working examples:
//! - `basic.rs` - Simple struct serialization
//! - `with_derive.rs` - Using derive macros
//! - `arrays_tuples.rs` - Arrays and tuples
//! - `nested_structs.rs` - Nested structures
//! - `optional_fields.rs` - Optional fields with Option<T>
//! - `robot_control.rs` - Complex embedded system
//! - `custom_serializer.rs` - Custom serializer implementation

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
    value.serialize("", &mut serializer)?;
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
    value.serialize("", &mut serializer)?;
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
    T::deserialize(&mut deserializer, "")
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
