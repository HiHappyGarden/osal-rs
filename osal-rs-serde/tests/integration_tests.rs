/***************************************************************************
 *
 * osal-rs-serde - Tests
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

use osal_rs_serde::{to_bytes, from_bytes, Serialize, Deserialize, ByteDeserializer, Serializer, Deserializer};

#[test]
fn test_u8_serialization() {
    let value = 42u8;
    let mut buffer = [0u8; 1];
    let len = to_bytes(&value, &mut buffer).unwrap();
    assert_eq!(len, 1);
    assert_eq!(buffer[0], 42);
    
    let restored: u8 = from_bytes(&buffer).unwrap();
    assert_eq!(restored, 42);
}

#[test]
fn test_i32_serialization() {
    let value = -12345i32;
    let mut buffer = [0u8; 4];
    let len = to_bytes(&value, &mut buffer).unwrap();
    assert_eq!(len, 4);
    
    let restored: i32 = from_bytes(&buffer).unwrap();
    assert_eq!(restored, -12345);
}

#[test]
fn test_bool_serialization() {
    let mut buffer = [0u8; 1];
    
    let len = to_bytes(&true, &mut buffer).unwrap();
    assert_eq!(len, 1);
    let restored: bool = from_bytes(&buffer).unwrap();
    assert_eq!(restored, true);
    
    let len = to_bytes(&false, &mut buffer).unwrap();
    assert_eq!(len, 1);
    let restored: bool = from_bytes(&buffer).unwrap();
    assert_eq!(restored, false);
}

#[test]
fn test_tuple_serialization() {
    let tuple = (100u16, 200u16);
    let mut buffer = [0u8; 8];
    let len = to_bytes(&tuple, &mut buffer).unwrap();
    assert_eq!(len, 4); // 2 bytes + 2 bytes
    
    let restored: (u16, u16) = from_bytes(&buffer).unwrap();
    assert_eq!(restored, (100, 200));
}

#[test]
fn test_option_some_serialization() {
    let value: Option<u32> = Some(42);
    let mut buffer = [0u8; 8];
    let len = to_bytes(&value, &mut buffer).unwrap();
    assert_eq!(len, 5); // 1 byte tag + 4 bytes value
    
    let restored: Option<u32> = from_bytes(&buffer).unwrap();
    assert_eq!(restored, Some(42));
}

#[test]
fn test_option_none_serialization() {
    let value: Option<u32> = None;
    let mut buffer = [0u8; 8];
    let len = to_bytes(&value, &mut buffer).unwrap();
    assert_eq!(len, 1); // Just the tag
    
    let restored: Option<u32> = from_bytes(&buffer).unwrap();
    assert_eq!(restored, None);
}

#[test]
fn test_array_serialization() {
    let array = [1u8, 2, 3, 4, 5];
    let mut buffer = [0u8; 8];
    let len = to_bytes(&array, &mut buffer).unwrap();
    assert_eq!(len, 5);
    
    let restored: [u8; 5] = from_bytes(&buffer).unwrap();
    assert_eq!(restored, [1, 2, 3, 4, 5]);
}

#[test]
fn test_f32_serialization() {
    let value = 3.14159f32;
    let mut buffer = [0u8; 4];
    let len = to_bytes(&value, &mut buffer).unwrap();
    assert_eq!(len, 4);
    
    let restored: f32 = from_bytes(&buffer).unwrap();
    assert!((restored - 3.14159).abs() < 0.00001);
}

#[test]
fn test_buffer_too_small() {
    let value = 12345u32;
    let mut buffer = [0u8; 2]; // Too small for u32
    let result = to_bytes(&value, &mut buffer);
    assert!(result.is_err());
}

#[test]
fn test_manual_serialize() {
    struct Point {
        x: i32,
        y: i32,
    }

    impl Serialize for Point {
        fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
            serializer.serialize_i32(self.x)?;
            serializer.serialize_i32(self.y)?;
            Ok(())
        }
    }

    impl Deserialize for Point {
        fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
            Ok(Point {
                x: deserializer.deserialize_i32()?,
                y: deserializer.deserialize_i32()?,
            })
        }
    }

    let point = Point { x: 10, y: 20 };
    let mut buffer = [0u8; 8];
    let len = to_bytes(&point, &mut buffer).unwrap();
    assert_eq!(len, 8);

    let restored: Point = from_bytes(&buffer).unwrap();
    assert_eq!(restored.x, 10);
    assert_eq!(restored.y, 20);
}

#[test]
fn test_unexpected_eof() {
    let buffer = [1u8, 2]; // Only 2 bytes
    let mut deserializer = ByteDeserializer::new(&buffer);
    
    // Try to read u8 - should succeed
    let _value = deserializer.deserialize_u8().unwrap();
    
    // Try to read u32 - should fail with UnexpectedEof
    let result = deserializer.deserialize_u32();
    assert!(result.is_err());
}
