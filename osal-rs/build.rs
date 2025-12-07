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
