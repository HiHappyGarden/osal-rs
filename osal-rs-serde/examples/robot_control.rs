/***************************************************************************
 *
 * osal-rs-serde - Robot Control Example
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

//! Complex embedded system example showing robot control state.

use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

/// Motor control parameters
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct MotorControl {
    motor_id: u8,
    speed: i16,        // -1000 to 1000
    direction: bool,   // true = forward, false = reverse
    current: u16,      // mA
}

/// Complete robot state
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct RobotState {
    timestamp: u64,
    motors: [MotorControl; 4],  // 4 motors
    battery_voltage: u16,        // mV
    temperature: i8,             // °C
    error_flags: u32,
}

fn main() {
    println!("=== OSAL-RS-Serde Robot Control Example ===\n");

    let state = RobotState {
        timestamp: 1000000,
        motors: [
            MotorControl { motor_id: 0, speed: 500, direction: true, current: 1200 },
            MotorControl { motor_id: 1, speed: 500, direction: true, current: 1150 },
            MotorControl { motor_id: 2, speed: -300, direction: false, current: 800 },
            MotorControl { motor_id: 3, speed: -300, direction: false, current: 850 },
        ],
        battery_voltage: 12400,  // 12.4V
        temperature: 35,
        error_flags: 0,
    };

    println!("Robot State:");
    println!("  Timestamp: {}", state.timestamp);
    println!("  Battery: {:.2}V", state.battery_voltage as f32 / 1000.0);
    println!("  Temperature: {}°C", state.temperature);
    println!("  Error flags: 0x{:08X}", state.error_flags);
    println!("\n  Motors:");
    for motor in &state.motors {
        println!("    Motor {}: speed={:5}, dir={}, current={}mA",
                 motor.motor_id,
                 motor.speed,
                 if motor.direction { "FWD" } else { "REV" },
                 motor.current);
    }

    let mut buffer = [0u8; 256];
    let len = to_bytes(&state, &mut buffer).unwrap();
    println!("\nRobot state serialized: {} bytes", len);

    // Show byte representation (first 32 bytes)
    print!("First 32 bytes: ");
    for i in 0..32.min(len) {
        print!("{:02X} ", buffer[i]);
        if (i + 1) % 16 == 0 {
            print!("\n                ");
        }
    }
    println!();

    // Deserialize and check
    let decoded: RobotState = from_bytes(&buffer[..len]).unwrap();
    println!("\nDecoded Robot State:");
    println!("  Battery: {:.2}V", decoded.battery_voltage as f32 / 1000.0);
    println!("  Temperature: {}°C", decoded.temperature);
    
    assert_eq!(state, decoded);

    // Simulate error condition
    println!("\n=== Robot with error condition ===");
    
    let error_state = RobotState {
        timestamp: 1001000,
        motors: [
            MotorControl { motor_id: 0, speed: 0, direction: true, current: 0 },
            MotorControl { motor_id: 1, speed: 0, direction: true, current: 0 },
            MotorControl { motor_id: 2, speed: 0, direction: true, current: 0 },
            MotorControl { motor_id: 3, speed: 0, direction: true, current: 0 },
        ],
        battery_voltage: 9500,   // Low battery!
        temperature: 65,          // High temperature!
        error_flags: 0x0000_0101, // Battery low + temperature high
    };

    println!("Error State:");
    println!("  Battery: {:.2}V (LOW!)", error_state.battery_voltage as f32 / 1000.0);
    println!("  Temperature: {}°C (HIGH!)", error_state.temperature);
    println!("  Error flags: 0x{:08X}", error_state.error_flags);
    if error_state.error_flags & 0x01 != 0 {
        println!("    - Battery low warning");
    }
    if error_state.error_flags & 0x100 != 0 {
        println!("    - Temperature high warning");
    }

    let len = to_bytes(&error_state, &mut buffer).unwrap();
    println!("\nError state serialized: {} bytes", len);

    let decoded: RobotState = from_bytes(&buffer[..len]).unwrap();
    assert_eq!(error_state, decoded);

    println!("\n=== Example completed successfully! ===");
}
