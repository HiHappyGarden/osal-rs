use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Check which features are enabled via environment variables
    let freertos_enabled = env::var("CARGO_FEATURE_FREERTOS").is_ok();
    let posix_enabled = env::var("CARGO_FEATURE_POSIX").is_ok();

    // Compile FreeRTOS only if the "freertos" feature is enabled
    if freertos_enabled {
        println!("cargo:warning=Building with FreeRTOS backend");
        
        let mut cmake_config = cmake::Config::new(".");
        cmake_config.define("FREERTOS_VERSION", "V11.2.0");

        if let Ok(port) = env::var("FREERTOS_PORT") {
            cmake_config.define("FREERTOS_PORT", &port);
            println!("cargo:warning=Using custom FreeRTOS port: {}", port);
        }
        
        if let Ok(heap) = env::var("FREERTOS_HEAP") {
            cmake_config.define("FREERTOS_HEAP", &heap);
            println!("cargo:warning=Using custom FreeRTOS heap: heap_{}.c", heap);
        }

        let config_include = if let Ok(config_include) = env::var("FREERTOS_CONFIG_INCLUDE") {
            cmake_config.define("FREERTOS_CONFIG_INCLUDE", &config_include);
            config_include
        } else {
            "include".to_string()
        };
        

        let dst = cmake_config.build();

        println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
        println!("cargo:rustc-link-lib=static=freertos");
        
        let freertos_include = dst.join("include/freertos");
        let freertos_portable_include = dst.join("include/freertos/portable");
        let freertos_config_include = PathBuf::from(config_include);

        println!("cargo:warning=FreeRTOS headers available at: {}", freertos_include.display());
        println!("cargo:warning=FreeRTOS kernel built at: {}", dst.display());
        println!("cargo:warning=FreeRTOS config include built at: {}", freertos_config_include.display());

        // Compile the config wrapper to expose constants that bindgen can't parse
        cc::Build::new()
            .file("freertos_posix/freertos_config_wrapper.c")
            .include(&freertos_include)
            .include(&freertos_portable_include)
            .include(&freertos_config_include)
            .compile("freertos_config_wrapper");

        // Generate Rust bindings for FreeRTOS
        bindgen::Builder::default()
            .header_contents("wrapper.h",
                             r#"
#include "FreeRTOS.h"
#include "task.h"
#include "queue.h"
#include "semphr.h"
#include "timers.h"
#include "event_groups.h"
#include "stream_buffer.h"
#include "message_buffer.h"
#include "portmacro.h"

// External constants from wrapper
unsigned long get_freertos_cpu_clock_hz(void);
unsigned long get_freertos_tick_rate_hz(void);
unsigned long get_freertos_minimal_stack_size(void);
unsigned long get_freertos_total_heap_size(void);
unsigned long get_freertos_timer_task_stack_depth(void);
TickType_t port_tick_period_ms();
"#)
            .use_core()
            .clang_arg(format!("-I{}", freertos_include.display()))
            .clang_arg(format!("-I{}", freertos_portable_include.display()))
            .clang_arg(format!("-I{}", freertos_config_include.display()))
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(out_dir.join("freertos_bindings.rs"))
            .expect("Couldn't write bindings!");

        println!("cargo:warning=Generated FreeRTOS bindings at: {}/freertos_bindings.rs", out_dir.display());
        println!("cargo:rustc-env=FREERTOS_INCLUDE_DIR={}", freertos_include.display());
        println!("cargo:rustc-env=FREERTOS_CONFIG_DIR={}", freertos_config_include.display());

        println!("cargo:rerun-if-changed=CMakeLists.txt");
        println!("cargo:rerun-if-changed=cmake/FreeRTOS.cmake");
        println!("cargo:rerun-if-changed={}/FreeRTOSConfig.h", freertos_config_include.display());
        println!("cargo:rerun-if-changed=freertos_config_wrapper.c");
    }

    // Compile POSIX bindings only if the "posix" feature is enabled
    if posix_enabled {
        println!("cargo:warning=Building with POSIX backend");
        
        bindgen::Builder::default()
            .header_contents("wrapper.h",
r#"
#include <stdlib.h>
#include <pthread.h>
#include <time.h>
#include <sys/time.h>
#include <sys/resource.h>
"#)
            .clang_arg("-I/usr/include")
            .use_core()
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(out_dir.join("posix_bindings.rs"))
            .expect("Couldn't write bindings!");

        println!("cargo:warning=Generated POSIX bindings at: {}/posix_bindings.rs", out_dir.display());
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:warning=Linking pthread library for POSIX threads");
    }

    // Generate constants for FreeRTOS (if needed)
    // if freertos_enabled {
    //     if let Err(e) = generate_constants_from_config() {
    //         println!("cargo:warning=Failed to generate constants: {}", e);
    //     }
    // }
}