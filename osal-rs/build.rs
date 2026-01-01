/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
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

use osal_rs_build::FreeRtosTypeGenerator;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../osal-rs-build/osal-rs-ffi-freertos/src/osal_rs_ffi_freertos.c");
    println!("cargo:rerun-if-changed=../osal-rs-build/osal-rs-ffi-freertos/inc/osal_rs_ffi_freertos.h");
    
    // Get the workspace root
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(manifest_dir);
    let workspace_root = manifest_path
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find workspace root");
    
    // Set the FreeRTOSConfig.h path
    let freertos_config = workspace_root.join("inc/hhg-config/pico/FreeRTOSConfig.h");
    
    // Generate FreeRTOS type mappings and configuration constants
    let generator = FreeRtosTypeGenerator::with_config_path(freertos_config);
    generator.generate_all();
}
