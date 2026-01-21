/***************************************************************************
 *
 * osal-rs-serde - Basic Example
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

//! Basic example showing manual implementation of Serialize and Deserialize traits.

use osal_rs_serde::{Serialize, Deserialize, Serializer, Deserializer, to_bytes, from_bytes};

/// A simple 2D point structure
struct Point {
    x: i32,
    y: i32,
}

/// Manual implementation of Serialize
impl Serialize for Point {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_i32(self.x)?;
        serializer.serialize_i32(self.y)?;
        Ok(())
    }
}

/// Manual implementation of Deserialize
impl Deserialize for Point {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        Ok(Point {
            x: deserializer.deserialize_i32()?,
            y: deserializer.deserialize_i32()?,
        })
    }
}

fn main() {
    println!("=== OSAL-RS-Serde Basic Example ===\n");

    // Create a point
    let point = Point { x: 42, y: -17 };
    println!("Original point: x={}, y={}", point.x, point.y);

    // Serialize to bytes
    let mut buffer = [0u8; 32];
    let len = to_bytes(&point, &mut buffer).unwrap();
    println!("Serialized {} bytes: {:?}", len, &buffer[..len]);

    // Deserialize back
    let restored: Point = from_bytes(&buffer[..len]).unwrap();
    println!("Restored point: x={}, y={}", restored.x, restored.y);

    // Test with primitive types
    println!("\n=== Primitive Types ===");
    
    let value = 12345u32;
    let mut buffer = [0u8; 4];
    let len = to_bytes(&value, &mut buffer).unwrap();
    println!("u32 {} serialized to {} bytes: {:?}", value, len, &buffer[..len]);
    
    let restored: u32 = from_bytes(&buffer[..len]).unwrap();
    println!("Restored u32: {}", restored);

    // Test with tuples
    println!("\n=== Tuples ===");
    
    let tuple = (100u16, 200u16);
    let mut buffer = [0u8; 8];
    let len = to_bytes(&tuple, &mut buffer).unwrap();
    println!("Tuple {:?} serialized to {} bytes", tuple, len);
    
    let restored: (u16, u16) = from_bytes(&buffer[..len]).unwrap();
    println!("Restored tuple: {:?}", restored);

    // Test with Option
    println!("\n=== Option ===");
    
    let some_value: Option<i32> = Some(42);
    let mut buffer = [0u8; 8];
    let len = to_bytes(&some_value, &mut buffer).unwrap();
    println!("Some(42) serialized to {} bytes", len);
    
    let restored: Option<i32> = from_bytes(&buffer[..len]).unwrap();
    println!("Restored: {:?}", restored);

    let none_value: Option<i32> = None;
    let len = to_bytes(&none_value, &mut buffer).unwrap();
    println!("None serialized to {} bytes", len);
    
    let restored: Option<i32> = from_bytes(&buffer[..len]).unwrap();
    println!("Restored: {:?}", restored);

    println!("\n=== Example completed successfully! ===");
}
