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

//! Build script for OSAL-RS library.
//!
//! This build script runs at compile time to perform several critical setup tasks:
//!
//! # Purpose
//!
//! 1. **Generate FreeRTOS type mappings**: Creates Rust bindings for FreeRTOS C types
//! 2. **Extract configuration constants**: Reads `FreeRTOSConfig.h` and exposes constants to Rust
//! 3. **Set up rebuild triggers**: Ensures the library rebuilds when FFI code changes
//!
//! # What It Generates
//!
//! The script generates Rust source files containing:
//! - Type aliases for FreeRTOS types (e.g., `TickType`, `BaseType`)
//! - Configuration constants (e.g., `configTICK_RATE_HZ`, `configMAX_PRIORITIES`)
//! - Platform-specific type sizes and layouts
//!
//! These generated files are included by the `osal-rs` crate at compile time via
//! `include!` macros.
//!
//! # Configuration
//!
//! ## FreeRTOSConfig.h Location
//!
//! The script searches for `FreeRTOSConfig.h` in the following order:
//!
//! 1. **Environment variable**: `FREERTOS_CONFIG_PATH` (if set)
//! 2. **Default location**: `<workspace_root>/inc/FreeRTOSConfig.h`
//!
//! ### Setting Custom Config Path
//!
//! To use a custom configuration file location, set the environment variable:
//!
//! ```bash
//! export FREERTOS_CONFIG_PATH=/path/to/FreeRTOSConfig.h
//! cargo build
//! ```
//!
//! Or in `.cargo/config.toml`:
//!
//! ```toml
//! [env]
//! FREERTOS_CONFIG_PATH = { value = "/path/to/FreeRTOSConfig.h" }
//! ```
//!
//! # Rebuild Triggers
//!
//! The library will rebuild if any of these files change:
//! - `build.rs` (this file)
//! - `osal_rs_ffi_freertos.c` (FFI implementation)
//! - `osal_rs_ffi_freertos.h` (FFI header)
//!
//! # Build Dependencies
//!
//! Requires:
//! - `osal-rs-build` crate (provides `FreeRtosTypeGenerator`)
//! - Valid `FreeRTOSConfig.h` file
//! - C compiler for parsing FreeRTOS headers
//!
//! # Troubleshooting
//!
//! If the build fails:
//! 1. Verify `FreeRTOSConfig.h` exists at the expected location
//! 2. Check that `FREERTOS_CONFIG_PATH` is correct (if set)
//! 3. Ensure FreeRTOS headers are accessible
//! 4. Check build output for specific error messages
//!
//! # See Also
//!
//! - `osal-rs-build` crate for the type generation implementation
//! - `FreeRTOSConfig.h` for FreeRTOS configuration options

use osal_rs_build::TypeGenerator;
use std::env;
use std::path::PathBuf;

/// Main entry point for the build script.
///
/// This function performs the following tasks in order:
///
/// 1. **Sets rebuild triggers**: Configures cargo to rebuild when FFI files change
/// 2. **Locates workspace root**: Determines the workspace root directory
/// 3. **Finds FreeRTOSConfig.h**: Searches for the configuration file
/// 4. **Generates type bindings**: Creates Rust type mappings from FreeRTOS C types
///
/// # Rebuild Triggers
///
/// The script tells cargo to rebuild if these files change:
/// - `build.rs` - This build script itself
/// - `osal_rs_ffi_freertos.c` - FFI implementation in C
/// - `osal_rs_ffi_freertos.h` - FFI header declarations
///
/// # FreeRTOSConfig.h Discovery
///
/// The configuration file is located using this precedence:
/// 1. Environment variable `FREERTOS_CONFIG_PATH` (if set)
/// 2. Default path: `<workspace_root>/inc/FreeRTOSConfig.h`
///
/// # Generation Process
///
/// Uses `FreeRtosTypeGenerator` to:
/// - Parse FreeRTOSConfig.h for configuration constants
/// - Generate Rust type aliases for FreeRTOS types
/// - Create platform-specific type definitions
/// - Write generated code to source files in the build output directory
///
/// # Panics
///
/// - If `CARGO_MANIFEST_DIR` environment variable is not set (cargo always sets this)
/// - If workspace root cannot be determined from manifest path
/// - If FreeRTOSConfig.h cannot be found or parsed (handled by `FreeRtosTypeGenerator`)
///
/// # Environment Variables
///
/// - `CARGO_MANIFEST_DIR` - Set by cargo, points to the crate's directory
/// - `FREERTOS_CONFIG_PATH` - Optional, custom path to FreeRTOSConfig.h
fn main() {
    // Tell cargo to rerun this build script if any of these files change.
    // This ensures the generated bindings stay synchronized with the FFI implementation.
    println!("cargo:rerun-if-changed=build.rs");

    // Get the workspace root directory by navigating up from the manifest directory.
    // Manifest dir is typically: <workspace>/osal-rs/osal-rs
    // Workspace root is: <workspace>
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    

    // Initialize the type generator with the FreeRTOS configuration file path.
    // This will parse FreeRTOSConfig.h and generate Rust type definitions and constants.
    let generator = TypeGenerator::new(&PathBuf::from(manifest_dir));

    #[cfg(feature = "freertos")]
    {
        TypeGenerator::add_rerun_if_changed();

        // Generate all type mappings, configuration constants, and FFI bindings.
        // Generated files are written to the OUT_DIR and included by the main crate.
        generator.generate_all();
    }

    #[cfg(all(feature = "posix", not(feature = "freertos")))]
    {
        // Generate all type mappings, configuration constants, and FFI bindings.
        // Generated files are written to the OUT_DIR and included by the main crate.
        generator.generate_all();
    }

    #[cfg(all(not(feature = "posix"), not(feature = "freertos")))]
    compile_error!("Either the \"posix\" or the \"freertos\" feature must be enabled");
}
