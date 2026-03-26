/***************************************************************************
 *
 * osal-rs-serde - Nested Structs Example
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

//! Example showing nested structures with derive macros.

use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

/// Geographic location
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Location {
    latitude: i32,   // Scaled by 1,000,000
    longitude: i32,  // Scaled by 1,000,000
}

/// Device information with nested location
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Device {
    id: u32,
    battery: u8,
    location: Location,
    active: bool,
}

/// Network with multiple devices
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Network {
    network_id: u16,
    devices: [Device; 3],
    timestamp: u64,
}

fn main() {
    println!("=== OSAL-RS-Serde Nested Structs Example ===\n");

    // Single device
    let device = Device {
        id: 42,
        battery: 85,
        location: Location {
            latitude: 45500000,   // 45.500000°
            longitude: 9200000,   // 9.200000°
        },
        active: true,
    };

    println!("Device: {:?}", device);
    println!("Location: lat={:.6}°, lon={:.6}°",
             device.location.latitude as f64 / 1_000_000.0,
             device.location.longitude as f64 / 1_000_000.0);

    let mut buffer = [0u8; 64];
    let len = to_bytes(&device, &mut buffer).unwrap();
    println!("Serialized {} bytes", len);

    let decoded: Device = from_bytes(&buffer[..len]).unwrap();
    println!("Decoded device: {:?}", decoded);
    assert_eq!(device, decoded);

    // Network with multiple devices
    println!("\n=== Network with multiple devices ===");

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
                active: true,
            },
            Device {
                id: 3,
                battery: 30,
                location: Location { latitude: 45490000, longitude: 9190000 },
                active: false,
            },
        ],
        timestamp: 1642857600,
    };

    println!("Network ID: {}", network.network_id);
    println!("Number of devices: {}", network.devices.len());
    for (i, dev) in network.devices.iter().enumerate() {
        println!("  Device {}: ID={}, Battery={}%, Active={}", 
                 i + 1, dev.id, dev.battery, dev.active);
    }

    let mut large_buffer = [0u8; 256];
    let len = to_bytes(&network, &mut large_buffer).unwrap();
    println!("\nNetwork serialized: {} bytes", len);

    let decoded: Network = from_bytes(&large_buffer[..len]).unwrap();
    println!("Decoded network ID: {}", decoded.network_id);
    assert_eq!(network, decoded);

    println!("\n=== Example completed successfully! ===");
}
