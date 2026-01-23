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

use osal_rs_serde::{to_bytes, from_bytes, ByteDeserializer, Serializer, Deserializer, Serialize, Deserialize};

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
            serializer.serialize_i32("x", self.x)?;
            serializer.serialize_i32("y", self.y)?;
            Ok(())
        }
    }

    impl Deserialize for Point {
        fn deserialize<D: Deserializer>(deserializer: &mut D, _name: &str) -> Result<Self, D::Error> {
            Ok(Point {
                x: deserializer.deserialize_i32("x")?,
                y: deserializer.deserialize_i32("y")?,
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
    let _value = deserializer.deserialize_u8("").unwrap();
    
    // Try to read u32 - should fail with UnexpectedEof
    let result = deserializer.deserialize_u32("");
    assert!(result.is_err());
}

// ============================================================================
// Tests with #[derive(Serialize, Deserialize)]
// ============================================================================

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct SensorData {
    temperature: i16,
    humidity: u8,
    pressure: u32,
}

#[test]
fn test_derive_basic_struct() {
    let data = SensorData {
        temperature: 25,
        humidity: 60,
        pressure: 1013,
    };

    let mut buffer = [0u8; 32];
    let len = to_bytes(&data, &mut buffer).unwrap();
    assert!(len > 0);

    let decoded: SensorData = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(data.temperature, decoded.temperature);
    assert_eq!(data.humidity, decoded.humidity);
    assert_eq!(data.pressure, decoded.pressure);
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Config {
    device_id: u32,
    name: Option<u8>,
    enabled: bool,
    timeout: Option<u16>,
}

#[test]
fn test_derive_optional_fields_all_some() {
    let config = Config {
        device_id: 100,
        name: Some(42),
        enabled: true,
        timeout: Some(5000),
    };

    let mut buffer = [0u8; 64];
    let len = to_bytes(&config, &mut buffer).unwrap();

    let decoded: Config = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(config, decoded);
}

#[test]
fn test_derive_optional_fields_all_none() {
    let config = Config {
        device_id: 300,
        name: None,
        enabled: true,
        timeout: None,
    };

    let mut buffer = [0u8; 64];
    let len = to_bytes(&config, &mut buffer).unwrap();

    let decoded: Config = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(config, decoded);
    assert!(decoded.name.is_none());
    assert!(decoded.timeout.is_none());
}

#[test]
fn test_derive_optional_fields_mixed() {
    let config = Config {
        device_id: 200,
        name: None,
        enabled: false,
        timeout: Some(1000),
    };

    let mut buffer = [0u8; 64];
    let len = to_bytes(&config, &mut buffer).unwrap();

    let decoded: Config = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(config, decoded);
    assert!(decoded.name.is_none());
    assert_eq!(decoded.timeout, Some(1000));
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TelemetryPacket {
    timestamp: u64,
    coordinates: (i32, i32, i32),
    samples: [u16; 8],
    status: u8,
}

#[test]
fn test_derive_arrays_and_tuples() {
    let packet = TelemetryPacket {
        timestamp: 1642857600,
        coordinates: (100, 200, 50),
        samples: [10, 20, 30, 40, 50, 60, 70, 80],
        status: 0xFF,
    };

    let mut buffer = [0u8; 128];
    let len = to_bytes(&packet, &mut buffer).unwrap();

    let decoded: TelemetryPacket = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(packet, decoded);
    assert_eq!(decoded.coordinates, (100, 200, 50));
    assert_eq!(decoded.samples, [10, 20, 30, 40, 50, 60, 70, 80]);
}

#[test]
fn test_derive_arrays_with_negative_values() {
    let packet = TelemetryPacket {
        timestamp: 1642857700,
        coordinates: (-50, -100, 150),
        samples: [100, 110, 120, 130, 140, 150, 160, 170],
        status: 0x01,
    };

    let mut buffer = [0u8; 128];
    let len = to_bytes(&packet, &mut buffer).unwrap();

    let decoded: TelemetryPacket = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(packet, decoded);
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Location {
    latitude: i32,
    longitude: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Device {
    id: u32,
    battery: u8,
    location: Location,
    active: bool,
}

#[test]
fn test_derive_nested_structs() {
    let device = Device {
        id: 42,
        battery: 85,
        location: Location {
            latitude: 45500000,
            longitude: 9200000,
        },
        active: true,
    };

    let mut buffer = [0u8; 64];
    let len = to_bytes(&device, &mut buffer).unwrap();

    let decoded: Device = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(device, decoded);
    assert_eq!(decoded.location.latitude, 45500000);
    assert_eq!(decoded.location.longitude, 9200000);
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Network {
    network_id: u16,
    devices: [Device; 2],
    timestamp: u64,
}

#[test]
fn test_derive_nested_arrays_of_structs() {
    let network = Network {
        network_id: 100,
        devices: [
            Device {
                id: 1,
                battery: 95,
                location: Location { latitude: 45500000, longitude: 9200000 },
                active: true,
            },
            Device {
                id: 2,
                battery: 60,
                location: Location { latitude: 45510000, longitude: 9210000 },
                active: false,
            },
        ],
        timestamp: 1642857600,
    };

    let mut buffer = [0u8; 256];
    let len = to_bytes(&network, &mut buffer).unwrap();

    let decoded: Network = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(network, decoded);
    assert_eq!(decoded.devices.len(), 2);
    assert_eq!(decoded.devices[0].id, 1);
    assert_eq!(decoded.devices[1].id, 2);
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct MotorControl {
    motor_id: u8,
    speed: i16,
    direction: bool,
    current: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct RobotState {
    timestamp: u64,
    motors: [MotorControl; 4],
    battery_voltage: u16,
    temperature: i8,
    error_flags: u32,
}

#[test]
fn test_derive_complex_robot_state() {
    let state = RobotState {
        timestamp: 1000000,
        motors: [
            MotorControl { motor_id: 0, speed: 500, direction: true, current: 1200 },
            MotorControl { motor_id: 1, speed: 500, direction: true, current: 1150 },
            MotorControl { motor_id: 2, speed: -300, direction: false, current: 800 },
            MotorControl { motor_id: 3, speed: -300, direction: false, current: 850 },
        ],
        battery_voltage: 12400,
        temperature: 35,
        error_flags: 0,
    };

    let mut buffer = [0u8; 256];
    let len = to_bytes(&state, &mut buffer).unwrap();
    assert!(len > 0);

    let decoded: RobotState = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(state, decoded);
    assert_eq!(decoded.motors.len(), 4);
    assert_eq!(decoded.battery_voltage, 12400);
    assert_eq!(decoded.temperature, 35);
}

#[test]
fn test_derive_robot_state_with_errors() {
    let error_state = RobotState {
        timestamp: 1001000,
        motors: [
            MotorControl { motor_id: 0, speed: 0, direction: true, current: 0 },
            MotorControl { motor_id: 1, speed: 0, direction: true, current: 0 },
            MotorControl { motor_id: 2, speed: 0, direction: true, current: 0 },
            MotorControl { motor_id: 3, speed: 0, direction: true, current: 0 },
        ],
        battery_voltage: 9500,
        temperature: 65,
        error_flags: 0x0000_0101,
    };

    let mut buffer = [0u8; 256];
    let len = to_bytes(&error_state, &mut buffer).unwrap();

    let decoded: RobotState = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(error_state, decoded);
    assert_eq!(decoded.error_flags, 0x0000_0101);
    assert!(decoded.error_flags & 0x01 != 0); // Battery low flag
    assert!(decoded.error_flags & 0x100 != 0); // Temperature high flag
}
