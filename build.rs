use std::env;
use std::path::PathBuf;

fn main() {
    let _out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile FreeRTOS only if the "freertos" feature is enabled
    #[cfg(feature = "freertos")]
    {
        // Configure CMake to download and compile FreeRTOS
        let mut cmake_config = cmake::Config::new(".");
        cmake_config.define("FREERTOS_VERSION", "V11.2.0");

        // Add FREERTOS_PORT and FREERTOS_HEAP parameters if specified
        if let Ok(port) = env::var("FREERTOS_PORT") {
            cmake_config.define("FREERTOS_PORT", &port);
            println!("cargo:warning=Using custom FreeRTOS port: {}", port);
        }
        
        if let Ok(heap) = env::var("FREERTOS_HEAP") {
            cmake_config.define("FREERTOS_HEAP", &heap);
            println!("cargo:warning=Using custom FreeRTOS heap: heap_{}.c", heap);
        }

        let dst = cmake_config.build();

        println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
        println!("cargo:rustc-link-lib=static=freertos");
        
        // Export FreeRTOS include directories for use in Rust code
        let freertos_include = dst.join("include/freertos");
        let freertos_config_include = PathBuf::from("include");

        // Print include paths for debugging
        println!("cargo:warning=FreeRTOS headers available at: {}", freertos_include.display());
        println!("cargo:warning=FreeRTOS kernel built at: {}", dst.display());

        // Generate Rust bindings for FreeRTOS headers using bindgen (if available)
        #[cfg(feature = "bindgen")]
        {
            let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
            let bindings = bindgen::Builder::default()
                .header("wrapper.h")
                .clang_arg(format!("-I{}", freertos_include.display()))
                .clang_arg(format!("-I{}", freertos_config_include.display()))
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                .generate()
                .expect("Unable to generate bindings");

            bindings
                .write_to_file(out_dir.join("freertos_bindings.rs"))
                .expect("Couldn't write bindings!");

            println!("cargo:warning=Generated FreeRTOS bindings at: {}/freertos_bindings.rs", out_dir.display());
        }

        // Export include directory as environment variable for manual FFI declarations
        println!("cargo:rustc-env=FREERTOS_INCLUDE_DIR={}", freertos_include.display());
        println!("cargo:rustc-env=FREERTOS_CONFIG_DIR={}", freertos_config_include.display());
    }
    
    // For POSIX no external libraries need to be compiled
    #[cfg(feature = "posix")]
    {
        println!("cargo:warning=Building with POSIX backend");
    }
    
    // Rebuild if CMake files change (only for FreeRTOS)
    #[cfg(feature = "freertos")]
    {
        println!("cargo:rerun-if-changed=CMakeLists.txt");
        println!("cargo:rerun-if-changed=cmake/FreeRTOS.cmake");
        println!("cargo:rerun-if-changed=include/FreeRTOSConfig.h");
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}
