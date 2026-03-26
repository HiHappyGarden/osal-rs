/***************************************************************************
 *
 * osal-rs-serde - Integration Example
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

//! Example showing advanced usage with custom types and nested structures.

use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

/// Device configuration
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct DeviceConfig {
    device_id: u32,
    enabled: bool,
    sample_rate: u16,
}

/// Sensor reading with timestamp
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct SensorReading {
    timestamp: u64,
    temperature: i16,
    humidity: u8,
}

/// Complete telemetry packet
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TelemetryPacket {
    config: DeviceConfig,
    readings: [SensorReading; 3],
    checksum: u32,
}


fn main() {
    println!("=== OSAL-RS-Serde Integration Example ===\n");

    // Create a complex data structure
    let packet = TelemetryPacket {
        config: DeviceConfig {
            device_id: 12345,
            enabled: true,
            sample_rate: 1000,
        },
        readings: [
            SensorReading {
                timestamp: 1000,
                temperature: 25,
                humidity: 60,
            },
            SensorReading {
                timestamp: 2000,
                temperature: 26,
                humidity: 58,
            },
            SensorReading {
                timestamp: 3000,
                temperature: 24,
                humidity: 62,
            },
        ],
        checksum: 0xDEADBEEF,
    };

    println!("Original packet:");
    println!("  Device ID: {}", packet.config.device_id);
    println!("  Enabled: {}", packet.config.enabled);
    println!("  Sample Rate: {}", packet.config.sample_rate);
    println!("  Readings: {} samples", packet.readings.len());
    println!("  Checksum: 0x{:08X}", packet.checksum);

    // Serialize
    let mut buffer = [0u8; 256];
    let len = to_bytes(&packet, &mut buffer).unwrap();
    println!("\nSerialized {} bytes", len);
    println!("First 32 bytes: {:02X?}", &buffer[..32.min(len)]);

    // Deserialize
    let restored: TelemetryPacket = from_bytes(&buffer[..len]).unwrap();
    println!("\nRestored packet:");
    println!("  Device ID: {}", restored.config.device_id);
    println!("  Enabled: {}", restored.config.enabled);
    println!("  Sample Rate: {}", restored.config.sample_rate);
    println!("  Readings: {} samples", restored.readings.len());
    println!("  Checksum: 0x{:08X}", restored.checksum);

    // Verify integrity
    if packet == restored {
        println!("\n✓ Data integrity verified!");
    } else {
        println!("\n✗ Data mismatch!");
    }

    // Example with Option
    println!("\n=== Optional Fields Example ===");
    
    #[derive(Serialize, Deserialize, Debug)]
    struct ConfigWithOptional {
        device_id: u32,
        network_address: Option<u32>,
        device_name: Option<u8>,  // Using u8 as simplified string
    }

    let config1 = ConfigWithOptional {
        device_id: 1,
        network_address: Some(0xC0A80101), // 192.168.1.1
        device_name: Some(65), // 'A'
    };

    let config2 = ConfigWithOptional {
        device_id: 2,
        network_address: None,
        device_name: None,
    };

    let mut buffer = [0u8; 32];
    let len1 = to_bytes(&config1, &mut buffer).unwrap();
    println!("Config with values: {} bytes", len1);

    let len2 = to_bytes(&config2, &mut buffer).unwrap();
    println!("Config without optional: {} bytes", len2);
    println!("Saved {} bytes by not including optional fields!", len1 - len2);

    println!("\n=== Example completed successfully! ===");
}
