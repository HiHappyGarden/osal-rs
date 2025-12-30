# OSAL-RS

Operating System Abstraction Layer for Rust - A cross-platform compatibility layer for embedded and real-time systems development.

## Overview

OSAL-RS provides a unified API for developing multi-platform embedded applications in Rust. It abstracts operating system-specific functionality, allowing you to write portable code that can run on different platforms with minimal changes.

## Current Implementation Status

- ✅ **FreeRTOS**: Fully implemented
- 🚧 **POSIX**: Planned
- 🚧 **Other RTOSes**: Future consideration

## Features

- **Thread Management**: Create, manage, and synchronize threads
- **Synchronization Primitives**: Mutexes, semaphores, event groups
- **Message Queues**: Inter-thread communication
- **Timers**: Software timers for periodic and one-shot operations
- **Memory Allocation**: Custom allocator support
- **Time Management**: Duration and tick handling
- **No-std Support**: Suitable for bare-metal embedded systems

## Prerequisites

Before using OSAL-RS in your project, ensure that:

1. **FreeRTOS is properly configured** in your project
2. **FreeRTOS is linked** to your final binary
3. **CMake build system** is set up for your embedded project
4. **Rust toolchain** with appropriate target support is installed

## CMake Integration

OSAL-RS is designed to be integrated into CMake-based projects. Here are several integration examples:

### Basic CMake Integration

Add OSAL-RS to your existing CMake project:

```cmake
cmake_minimum_required(VERSION 3.20)
project(my_embedded_project C CXX)

# Configure FreeRTOS (assuming it's already in your project)
add_subdirectory(freertos)

# Add OSAL-RS porting layer
add_library(osal_rs_porting STATIC
    osal-rs-porting/freertos/src/osal_rs_freertos.c
)

target_include_directories(osal_rs_porting PUBLIC
    osal-rs-porting/freertos/inc
    ${FREERTOS_INCLUDE_DIRS}
)

target_link_libraries(osal_rs_porting PUBLIC
    freertos
)

# Configure Rust library
set(RUST_TARGET "thumbv7em-none-eabihf")  # Adjust for your target
set(OSAL_RS_LIB "${CMAKE_CURRENT_SOURCE_DIR}/osal-rs/target/${RUST_TARGET}/release/libosal_rs.a")

# Custom command to build Rust library
add_custom_command(
    OUTPUT ${OSAL_RS_LIB}
    COMMAND cargo build --release --target ${RUST_TARGET} --features freertos
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/osal-rs
    COMMENT "Building OSAL-RS library"
)

add_custom_target(osal_rs_build DEPENDS ${OSAL_RS_LIB})

# Create imported library for OSAL-RS
add_library(osal_rs STATIC IMPORTED GLOBAL)
set_target_properties(osal_rs PROPERTIES
    IMPORTED_LOCATION ${OSAL_RS_LIB}
)
add_dependencies(osal_rs osal_rs_build)

# Your main application
add_executable(my_app
    src/main.c
)

target_link_libraries(my_app PRIVATE
    osal_rs
    osal_rs_porting
    freertos
)
```

### Advanced CMake Integration with Multiple Configurations

```cmake
# Function to build OSAL-RS for different configurations
function(add_osal_rs_library TARGET_NAME RUST_TARGET CARGO_PROFILE)
    set(PROFILE_DIR ${CARGO_PROFILE})
    if(CARGO_PROFILE STREQUAL "release")
        set(CARGO_FLAGS "--release")
    else()
        set(CARGO_FLAGS "")
    endif()

    set(LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/osal-rs/target/${RUST_TARGET}/${PROFILE_DIR}/libosal_rs.a")

    add_custom_command(
        OUTPUT ${LIB_PATH}
        COMMAND cargo build ${CARGO_FLAGS} --target ${RUST_TARGET} --features freertos
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/osal-rs
        COMMENT "Building OSAL-RS (${CARGO_PROFILE}) for ${RUST_TARGET}"
    )

    add_custom_target(${TARGET_NAME}_build DEPENDS ${LIB_PATH})

    add_library(${TARGET_NAME} STATIC IMPORTED GLOBAL)
    set_target_properties(${TARGET_NAME} PROPERTIES
        IMPORTED_LOCATION ${LIB_PATH}
    )
    add_dependencies(${TARGET_NAME} ${TARGET_NAME}_build)
endfunction()

# Use it in your project
add_osal_rs_library(osal_rs "thumbv7em-none-eabihf" "release")
```

### Integration with Corrosion (Recommended)

For a more seamless Rust-CMake integration, use [Corrosion](https://github.com/corrosion-rs/corrosion):

```cmake
cmake_minimum_required(VERSION 3.20)
project(my_embedded_project C CXX)

# Include Corrosion
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.4
)
FetchContent_MakeAvailable(Corrosion)

# Import OSAL-RS crate
corrosion_import_crate(
    MANIFEST_PATH osal-rs/Cargo.toml
    FEATURES freertos
)

# Configure FreeRTOS
add_subdirectory(freertos)

# OSAL-RS porting layer
add_library(osal_rs_porting STATIC
    osal-rs-porting/freertos/src/osal_rs_freertos.c
)

target_include_directories(osal_rs_porting PUBLIC
    osal-rs-porting/freertos/inc
    ${FREERTOS_INCLUDE_DIRS}
)

target_link_libraries(osal_rs_porting PUBLIC
    freertos
)

# Your application
add_executable(my_app src/main.c)

target_link_libraries(my_app PRIVATE
    osal-rs
    osal_rs_porting
    freertos
)
```

### Cross-Compilation Setup

Example CMake toolchain file for ARM Cortex-M:

```cmake
# toolchain-arm-none-eabi.cmake
set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR arm)

set(CMAKE_C_COMPILER arm-none-eabi-gcc)
set(CMAKE_CXX_COMPILER arm-none-eabi-g++)
set(CMAKE_ASM_COMPILER arm-none-eabi-gcc)

set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)

# Rust target
set(RUST_TARGET "thumbv7em-none-eabihf")
```

Use it with:

```bash
cmake -DCMAKE_TOOLCHAIN_FILE=toolchain-arm-none-eabi.cmake -B build
cmake --build build
```

## Usage Example

```rust
use osal_rs::os::*;

fn main() {
    // Create a thread
    let thread = Thread::new(
        "my_thread",
        4096,
        ThreadPriority::Normal,
        || {
            loop {
                println!("Hello from thread!");
                Duration::from_millis(1000).sleep();
            }
        }
    );

    // Create a mutex
    let mutex = Mutex::new().unwrap();
    
    // Use synchronization
    {
        let _guard = mutex.lock();
        // Critical section
    }

    // Create a queue
    let queue: Queue<u32> = Queue::new(10).unwrap();
    queue.send(42, Duration::from_millis(100)).unwrap();
    
    let value = queue.receive(Duration::from_millis(100)).unwrap();
    println!("Received: {}", value);
}
```

## Building

### For FreeRTOS targets:

```bash
# Install Rust target (example for ARM Cortex-M4F)
rustup target add thumbv7em-none-eabihf

# Build with FreeRTOS support
cargo build --release --target thumbv7em-none-eabihf --features freertos
```

### For native development/testing:

```bash
# Build with POSIX support (when implemented)
cargo build --features posix,std
```

## Project Structure

```
osal-rs/
├── osal-rs/              # Main library crate
├── osal-rs-build/        # Build utilities
├── osal-rs-tests/        # Test suite
└── osal-rs-porting/      # Platform-specific C/C++ code
    └── freertos/         # FreeRTOS porting layer
        ├── inc/          # Header files
        └── src/          # Implementation
```

## Configuration

OSAL-RS requires proper FreeRTOS configuration. Ensure your `FreeRTOSConfig.h` includes:

```c
#define configUSE_MUTEXES                1
#define configUSE_RECURSIVE_MUTEXES      1
#define configUSE_COUNTING_SEMAPHORES    1
#define configUSE_TIMERS                 1
#define configUSE_QUEUE_SETS             1
#define configSUPPORT_DYNAMIC_ALLOCATION 1
```

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## Author

Antonio Salsi

## Repository

[https://github.com/antoniosalsi/osal-rs](https://github.com/antoniosalsi/osal-rs)
