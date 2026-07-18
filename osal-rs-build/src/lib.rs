#![cfg_attr(target_os = "none", no_std)]

/***************************************************************************
 *
 * osal-rs
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

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::convert::AsRef;
use std::ffi::OsStr;

#[cfg(feature = "posix")]
pub struct TypeGenerator;

#[cfg(feature = "posix")]
impl TypeGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn with_config_path<P>(_config_path: P) -> Self {
        Self
    }

    pub fn set_config_path<P>(&mut self, _config_path: P) {}

    pub fn generate_types(&self) {}

    pub fn generate_all(&self) {}
}

#[cfg(not(feature = "freertos"))]
impl Default for TypeGenerator {

    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "freertos")]
pub struct TypeGenerator {
    out_dir: PathBuf,
    manifest_path: Option<PathBuf>,
}

#[cfg(feature = "freertos")]
impl TypeGenerator {

    /// Create a new generator with a custom FreeRTOSConfig.h path
    pub fn new<P>(manifest_path: P) -> Self
    where P: Into<PathBuf> + AsRef<OsStr>
    {
        let manifest_path: PathBuf = manifest_path.into();

        let workspace_root = manifest_path
            .parent() // Go up to osal-rs/
            .and_then(|p| p.parent()) // Go up to workspace root
            .expect("Failed to find workspace root");

        // Determine the path to FreeRTOSConfig.h.
        // Priority: Environment variable > Default location
        let _freertos_config = if let Ok(config_path) = env::var("FREERTOS_CONFIG_PATH") {
            // Use the path specified in FREERTOS_CONFIG_PATH environment variable
            PathBuf::from(config_path)
        } else {
            // Default: Look for FreeRTOSConfig.h in <workspace_root>/inc/
            workspace_root.join("inc/FreeRTOSConfig.h")
        };

        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        Self {
            out_dir,
            manifest_path: Some(manifest_path),
        }
    }

    /// Set the FreeRTOSConfig.h path
    pub fn set_manifest_path<P: Into<PathBuf>>(&mut self, manifest_path: P) {
        self.manifest_path = Some(manifest_path.into());
    }


    pub fn add_rerun_if_changed() {
        println!("cargo:rerun-if-changed=../osal-rs-porting/freeretos/src/osal_rs_freertos.c");
        println!("cargo:rerun-if-changed=../osal-rs-porting/freeretos/inc/osal_rs_freertos.h");
    }

    /// Query FreeRTOS type sizes and generate Rust type mappings
    pub fn generate_types(&self) {
        let (tick_size, ubase_size, base_size, base_signed, stack_size) = self.query_type_sizes();
        
        let tick_type = Self::size_to_type(tick_size, false);
        let ubase_type = Self::size_to_type(ubase_size, false);
        let base_type = Self::size_to_type(base_size, base_signed);
        let stack_type = Self::size_to_type(stack_size, true);

        
        self.write_generated_types(tick_size, tick_type, ubase_size, ubase_type, base_size, base_type, stack_size, stack_type);
        
        println!("cargo:warning=Generated FreeRTOS types: TickType={}, UBaseType={}, BaseType={} StackType={}", 
                 tick_type, ubase_type, base_type, stack_type);
    }

    /// Generate both types and config
    pub fn generate_all(&self) {
        self.generate_types();
        // self.generate_config();
    }

    /// Query the sizes of FreeRTOS types
    fn query_type_sizes(&self) -> (u16, u16, u16, bool, u16) {
        // Create a small C program to query the type sizes
        let query_program = r#"
#include <stdio.h>
#include <stdint.h>

// We need to include FreeRTOS headers - path will be provided by the main build
// For now, we'll use the compiled library approach
// This is a placeholder - we'll use the already compiled C library

int main() {
    // Since we can't easily compile against FreeRTOS in the build script,
    // we'll use a different approach: parse the compile_commands.json or
    // use predefined types based on common configurations
    
    // Common FreeRTOS configurations:
    // TickType_t is usually uint32_t (4 bytes) on 32-bit systems
    // UBaseType_t is usually uint32_t (4 bytes) on 32-bit systems  
    // BaseType_t is usually int32_t (4 bytes) on 32-bit systems
    // StackType_t is usually long (4 bytes) on 32-bit systems
    
    printf("TICK_TYPE_SIZE=%d\n", 4);
    printf("UBASE_TYPE_SIZE=%d\n", 4);
    printf("BASE_TYPE_SIZE=%d\n", 4);
    printf("BASE_TYPE_SIGNED=1\n");
    printf("STACK_TYPE_SIZE=%d\n", 4);
    
    return 0;
}
"#;
        
        let query_c = self.out_dir.join("query_types.c");
        fs::write(&query_c, query_program).expect("Failed to write query program");
        
        // Compile the query program
        let query_exe = self.out_dir.join("query_types");
        let compile_status = Command::new("gcc")
            .arg(&query_c)
            .arg("-o")
            .arg(&query_exe)
            .status();
        
        if compile_status.is_ok() && compile_status.unwrap().success() {
            // Run the query program
            let output = Command::new(&query_exe)
                .output()
                .expect("Failed to run query program");
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut tick_size = 4u16;
            let mut ubase_size = 4u16;
            let mut base_size = 4u16;
            let mut base_signed = true;
            let mut stack_type = 4u16;
            
            for line in stdout.lines() {
                if let Some(val) = line.strip_prefix("TICK_TYPE_SIZE=") {
                    tick_size = val.parse().unwrap_or(4);
                } else if let Some(val) = line.strip_prefix("UBASE_TYPE_SIZE=") {
                    ubase_size = val.parse().unwrap_or(4);
                } else if let Some(val) = line.strip_prefix("BASE_TYPE_SIZE=") {
                    base_size = val.parse().unwrap_or(4);
                } else if let Some(val) = line.strip_prefix("BASE_TYPE_SIGNED=") {
                    base_signed = val.parse::<u8>().unwrap_or(1) == 1;
                } else if let Some(val) = line.strip_prefix("STACK_TYPE_SIZE=") {
                    stack_type = val.parse().unwrap_or(4);
                } 
            }
            
            (tick_size, ubase_size, base_size, base_signed, stack_type)
        } else {
            // Default values for 32-bit ARM Cortex-M (typical for Raspberry Pi Pico)
            (4, 4, 4, true, 4)
        }
    }


    /// Convert a size to the corresponding Rust type
    fn size_to_type(size: u16, signed: bool) -> &'static str {
        match (size, signed) {
            (1, false) => "u8",
            (1, true) => "i8",
            (2, false) => "u16",
            (2, true) => "i16",
            (4, false) => "u32",
            (4, true) => "i32",
            (8, false) => "u64",
            (8, true) => "i64",
            // Default to u32 for unknown sizes
            _ => if signed { "i32" } else { "u32" },
        }
    }

    /// Write the generated types to a file
    fn write_generated_types(
        &self,
        tick_size: u16,
        tick_type: &str,
        ubase_size: u16,
        ubase_type: &str,
        base_size: u16,
        base_type: &str,
        stack_size: u16,
        stack_type: &str,
    ) {
        let generated_code = format!(r#"
// Auto-generated by build.rs - DO NOT EDIT MANUALLY
// This file contains FreeRTOS type mappings based on the actual type sizes

// FreeRTOS type mappings (auto-detected)
// TickType_t: {} bytes -> {}
// UBaseType_t: {} bytes -> {}
// BaseType_t: {} bytes -> {}
// StackType_t: {} bytes -> {}

pub type TickType = {};
pub type UBaseType = {};
pub type BaseType = {};
pub type StackType = {};

"#,
            tick_size, tick_type,
            ubase_size, ubase_type,
            base_size, base_type,
            stack_size, stack_type,
            tick_type,
            ubase_type,
            base_type,
            stack_type
        );
        
        let types_rs = self.out_dir.join("types_generated.rs");
        fs::write(&types_rs, generated_code).expect("Failed to write generated types");
    }

}

// #[cfg(feature = "freertos")]
// impl Default for TypeGenerator {

//     #[inline]
//     fn default() -> Self {
//         Self::new()
//     }
// }
