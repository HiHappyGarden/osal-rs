/***************************************************************************
 *
 * osal-rs-serde - Arrays and Tuples Example
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

//! Example showing arrays and tuples in derived structures.

use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

/// Telemetry packet with coordinates, samples and status
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TelemetryPacket {
    timestamp: u64,
    coordinates: (i32, i32, i32),  // x, y, z
    samples: [u16; 8],              // 8 sensor readings
    status: u8,
}

fn main() {
    println!("=== OSAL-RS-Serde Arrays and Tuples Example ===\n");

    let packet = TelemetryPacket {
        timestamp: 1642857600,
        coordinates: (100, 200, 50),
        samples: [10, 20, 30, 40, 50, 60, 70, 80],
        status: 0xFF,
    };

    println!("Original packet:");
    println!("  Timestamp: {}", packet.timestamp);
    println!("  Coordinates: {:?}", packet.coordinates);
    println!("  Samples: {:?}", packet.samples);
    println!("  Status: 0x{:02X}", packet.status);

    let mut buffer = [0u8; 128];
    let len = to_bytes(&packet, &mut buffer).unwrap();
    println!("\nTelemetry packet serialized: {} bytes", len);
    
    // Show the byte representation
    print!("Bytes: ");
    for b in &buffer[..len] {
        print!("{:02X} ", b);
    }
    println!();

    let decoded: TelemetryPacket = from_bytes(&buffer[..len]).unwrap();
    println!("\nDecoded packet:");
    println!("  Timestamp: {}", decoded.timestamp);
    println!("  Coordinates: {:?}", decoded.coordinates);
    println!("  Samples: {:?}", decoded.samples);
    println!("  Status: 0x{:02X}", decoded.status);

    assert_eq!(packet, decoded);

    // Test with different data
    println!("\n=== Second packet ===");
    
    let packet2 = TelemetryPacket {
        timestamp: 1642857700,
        coordinates: (-50, -100, 150),
        samples: [100, 110, 120, 130, 140, 150, 160, 170],
        status: 0x01,
    };

    println!("Packet 2 coordinates: {:?}", packet2.coordinates);
    
    let len = to_bytes(&packet2, &mut buffer).unwrap();
    println!("Serialized {} bytes", len);
    
    let decoded: TelemetryPacket = from_bytes(&buffer[..len]).unwrap();
    println!("Decoded coordinates: {:?}", decoded.coordinates);
    assert_eq!(packet2, decoded);

    println!("\n=== Example completed successfully! ===");
}
