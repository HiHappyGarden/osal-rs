/***************************************************************************
 *
 * osal-rs-serde - Optional Fields Example
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

//! Example showing optional fields in derived structures.

use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

/// Configuration structure with optional fields
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Config {
    device_id: u32,
    name: Option<u8>,      // Optional device name code
    enabled: bool,
    timeout: Option<u16>,  // Optional timeout in ms
}

fn main() {
    println!("=== OSAL-RS-Serde Optional Fields Example ===\n");

    // Config with all fields
    let config_full = Config {
        device_id: 100,
        name: Some(42),
        enabled: true,
        timeout: Some(5000),
    };

    println!("Config (all fields): {:?}", config_full);

    let mut buffer = [0u8; 64];
    let len = to_bytes(&config_full, &mut buffer).unwrap();
    println!("Serialized {} bytes", len);

    let decoded: Config = from_bytes(&buffer[..len]).unwrap();
    println!("Decoded: {:?}", decoded);
    assert_eq!(config_full.device_id, decoded.device_id);
    assert_eq!(config_full.name, decoded.name);
    assert_eq!(config_full.timeout, decoded.timeout);

    // Config with partial fields
    println!("\n=== Config with None values ===");
    
    let config_partial = Config {
        device_id: 200,
        name: None,
        enabled: false,
        timeout: Some(1000),
    };

    println!("Config (partial): {:?}", config_partial);

    let len = to_bytes(&config_partial, &mut buffer).unwrap();
    println!("Serialized {} bytes", len);

    let decoded: Config = from_bytes(&buffer[..len]).unwrap();
    println!("Decoded: {:?}", decoded);
    assert_eq!(config_partial.device_id, decoded.device_id);
    assert_eq!(config_partial.name, decoded.name);
    assert_eq!(config_partial.timeout, decoded.timeout);

    // Config with all None
    println!("\n=== Config with all optional None ===");
    
    let config_minimal = Config {
        device_id: 300,
        name: None,
        enabled: true,
        timeout: None,
    };

    println!("Config (minimal): {:?}", config_minimal);

    let len = to_bytes(&config_minimal, &mut buffer).unwrap();
    println!("Serialized {} bytes (notice smaller size due to None values)", len);

    let decoded: Config = from_bytes(&buffer[..len]).unwrap();
    println!("Decoded: {:?}", decoded);
    assert_eq!(config_minimal, decoded);

    println!("\n=== Example completed successfully! ===");
}
