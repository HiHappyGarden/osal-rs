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
//! Delegates to [`osal_rs_build::TypeGenerator`] to generate the `TickType`/
//! `UBaseType`/`BaseType`/`StackType` aliases the crate `include!`s from
//! `OUT_DIR/types_generated.rs`, and wire up `cargo:rerun-if-changed` for the
//! active backend's C porting sources. See `osal-rs-build`'s own docs for
//! exactly what each backend does at build time (POSIX probes the host
//! architecture and `SCHED_FIFO` support for real; the FreeRTOS path
//! currently always emits the common 32-bit type mapping rather than parsing
//! `FreeRTOSConfig.h`).
//!
//! # FreeRTOSConfig.h Location (freertos backend)
//!
//! Determined by `TypeGenerator` from:
//! 1. **Environment variable**: `FREERTOS_CONFIG_PATH` (if set)
//! 2. **Default location**: `<workspace_root>/inc/FreeRTOSConfig.h`
//!
//! ```bash
//! export FREERTOS_CONFIG_PATH=/path/to/FreeRTOSConfig.h
//! cargo build --features freertos
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
//! Rebuilds on changes to `build.rs` itself and, for `freertos`, to its C
//! porting sources (`osal-rs-porting/freeretos/`); `posix` has no C porting
//! sources of its own to track.
//!
//! # Feature Requirements
//!
//! Exactly one of `freertos` / `posix` must be enabled: enabling neither or
//! both trips one of the two `compile_error!`s below.
//!
//! # Build Dependencies
//!
//! Requires the `osal-rs-build` crate (`TypeGenerator`) and, for `posix`,
//! `gcc` on `PATH` (used only to probe the host architecture).

use osal_rs_build::TypeGenerator;
use std::env;
use std::path::PathBuf;

/// Main entry point for the build script.
///
/// Builds a [`TypeGenerator`] from `CARGO_MANIFEST_DIR`, then - for whichever
/// of `freertos`/`posix` is active - registers rerun-if-changed triggers via
/// `TypeGenerator::add_rerun_if_changed()` and runs `generator.generate_all()`
/// to write `types_generated.rs` (and, for `posix`, compile and link the C
/// porting shim).
///
/// # Panics
///
/// - If `CARGO_MANIFEST_DIR` is not set (cargo always sets this)
/// - If neither or both of `freertos`/`posix` are enabled (`compile_error!`)
/// - If generation fails (handled by `TypeGenerator`, e.g. missing `gcc`)
///
/// # Environment Variables
///
/// - `CARGO_MANIFEST_DIR` - Set by cargo, points to the crate's directory
/// - `FREERTOS_CONFIG_PATH` - Optional, custom path to `FreeRTOSConfig.h` (freertos backend)
fn main() {
    // Tell cargo to rerun this build script if any of these files change.
    // This ensures the generated bindings stay synchronized with the FFI implementation.
    println!("cargo:rerun-if-changed=build.rs");

    // Get the workspace root directory by navigating up from the manifest directory.
    // Manifest dir is typically: <workspace>/osal-rs/osal-rs
    // Workspace root is: <workspace>
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    

    let mut generator = TypeGenerator::new(&PathBuf::from(manifest_dir));

    #[cfg(all(not(feature = "posix"), feature = "freertos"))]
    {
        TypeGenerator::add_rerun_if_changed();

        // Generate all type mappings, configuration constants, and FFI bindings.
        // Generated files are written to the OUT_DIR and included by the main crate.
        generator.generate_all();
    }

    #[cfg(all(feature = "posix", not(feature = "freertos")))]
    {
        TypeGenerator::add_rerun_if_changed();
        
        // Generate all type mappings, configuration constants, and FFI bindings.
        // Generated files are written to the OUT_DIR and included by the main crate.
        generator.generate_all();
    }

    #[cfg(all(not(feature = "posix"), not(feature = "freertos")))]
    compile_error!("Either the \"posix\" or the \"freertos\" feature must be enabled");

    #[cfg(all(feature = "posix", feature = "freertos"))]
    compile_error!("Only one backend must be enabled");
}
