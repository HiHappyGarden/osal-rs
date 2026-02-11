/***************************************************************************
 *
 * osal-rs-serde - Binary Serialization Example
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This example demonstrates binary serialization/deserialization
 * using to_bytes and from_bytes with arrays of structs.
 *
 ***************************************************************************/

use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
struct UserConfig {
    user_id: u32,
    role: u8,
    active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
struct Config {
    version: u8,
    timezone: i16,
    users: [UserConfig; 3],  // Array of user configurations
    flags: u32,
}

fn main() {
    println!("=== Binary Serialization with to_bytes/from_bytes ===\n");
    
    // Create a configuration with array of users
    let config = Config {
        version: 1,
        timezone: 120,  // +2 hours
        users: [
            UserConfig { user_id: 1001, role: 1, active: true },
            UserConfig { user_id: 2002, role: 2, active: false },
            UserConfig { user_id: 3003, role: 3, active: true },
        ],
        flags: 0xFF00AA55,
    };

    println!("Original Config:");
    println!("  Version: {}", config.version);
    println!("  Timezone: {} minutes", config.timezone);
    println!("  Users:");
    for (i, user) in config.users.iter().enumerate() {
        println!("    [{}] ID: {}, Role: {}, Active: {}", 
                 i, user.user_id, user.role, user.active);
    }
    println!("  Flags: 0x{:08X}", config.flags);
    
    // Serialize to binary buffer
    let mut buffer = [0u8; 128];
    let len = to_bytes(&config, &mut buffer)
        .expect("Failed to serialize");
    
    println!("\n--- Binary Serialization ---");
    println!("Serialized {} bytes (little-endian format)", len);
    println!("Binary data (hex):");
    
    // Display binary data
    for chunk in buffer[..len].chunks(16) {
        print!("  ");
        for byte in chunk {
            print!("{:02X} ", byte);
        }
        println!();
    }
    
    // Calculate expected size
    // version: 1 byte
    // timezone: 2 bytes (i16)
    // users[0]: 4 (u32) + 1 (u8) + 1 (bool) = 6 bytes
    // users[1]: 6 bytes
    // users[2]: 6 bytes
    // flags: 4 bytes
    // Total: 1 + 2 + 18 + 4 = 25 bytes
    println!("\nExpected size: 25 bytes");
    println!("Actual size: {} bytes", len);
    assert_eq!(len, 25, "Size mismatch!");
    
    // Deserialize from binary
    println!("\n--- Binary Deserialization ---");
    let decoded: Config = from_bytes(&buffer[..len])
        .expect("Failed to deserialize");
    
    println!("Decoded Config:");
    println!("  Version: {}", decoded.version);
    println!("  Timezone: {} minutes", decoded.timezone);
    println!("  Users:");
    for (i, user) in decoded.users.iter().enumerate() {
        println!("    [{}] ID: {}, Role: {}, Active: {}", 
                 i, user.user_id, user.role, user.active);
    }
    println!("  Flags: 0x{:08X}", decoded.flags);
    
    // Verify data integrity
    println!("\n--- Verification ---");
    if decoded == config {
        println!("✓ Data integrity verified - perfect match!");
    } else {
        println!("✗ Data mismatch!");
    }
    
    assert_eq!(decoded.version, config.version);
    assert_eq!(decoded.timezone, config.timezone);
    assert_eq!(decoded.users, config.users);
    assert_eq!(decoded.flags, config.flags);
    
    println!("\n=== Binary Serialization Complete ===");
}
