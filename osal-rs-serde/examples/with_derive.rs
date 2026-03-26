/***************************************************************************
 *
 * osal-rs-serde - Derive Example
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

//! Example showing automatic derive of Serialize and Deserialize traits.

use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

/// Sensor data structure with derive macros
#[derive(Serialize, Deserialize, Debug)]
struct SensorData {
    temperature: i16,
    humidity: u8,
    pressure: u32,
    valid: bool,
}

/// GPS coordinates
#[derive(Serialize, Deserialize, Debug)]
struct GpsCoordinates {
    latitude: f64,
    longitude: f64,
    altitude: f32,
}

/// Message with nested structures
#[derive(Serialize, Deserialize, Debug)]
struct TelemetryMessage {
    timestamp: u64,
    sensor: SensorData,
    location: GpsCoordinates,
}

fn main() {
    println!("=== OSAL-RS-Serde Derive Example ===\n");

    // Create sensor data
    let sensor = SensorData {
        temperature: 25,
        humidity: 60,
        pressure: 1013,
        valid: true,
    };

    println!("Original sensor data: {:?}", sensor);

    // Serialize
    let mut buffer = [0u8; 128];
    let len = to_bytes(&sensor, &mut buffer).unwrap();
    println!("Serialized {} bytes", len);

    // Deserialize
    let restored: SensorData = from_bytes(&buffer[..len]).unwrap();
    println!("Restored sensor data: {:?}", restored);

    // Test with nested structures
    println!("\n=== Nested Structures ===");

    let telemetry = TelemetryMessage {
        timestamp: 1234567890,
        sensor: SensorData {
            temperature: 22,
            humidity: 55,
            pressure: 1015,
            valid: true,
        },
        location: GpsCoordinates {
            latitude: 45.4642,
            longitude: 9.1900,
            altitude: 122.5,
        },
    };

    println!("Original telemetry: {:?}", telemetry);

    let len = to_bytes(&telemetry, &mut buffer).unwrap();
    println!("Serialized {} bytes", len);

    let restored: TelemetryMessage = from_bytes(&buffer[..len]).unwrap();
    println!("Restored telemetry: {:?}", restored);

    println!("\n=== Example completed successfully! ===");
}
