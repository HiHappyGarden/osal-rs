use osal_rs_build::FreeRtosTypeGenerator;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../osal-rs-build/osal-rs-ffi-freertos/src/osal_rs_ffi_freertos.c");
    println!("cargo:rerun-if-changed=../osal-rs-build/osal-rs-ffi-freertos/inc/osal_rs_ffi_freertos.h");
    
    // Generate FreeRTOS type mappings and configuration constants
    let generator = FreeRtosTypeGenerator::new();
    generator.generate_all();
}
