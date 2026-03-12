# OSAL-RS

Operating System Abstraction Layer for Rust - A cross-platform compatibility layer for embedded and real-time systems development.

[![Crates.io](https://img.shields.io/crates/v/osal-rs.svg)](https://crates.io/crates/osal-rs)
[![Documentation](https://docs.rs/osal-rs/badge.svg)](https://docs.rs/osal-rs)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](LICENSE)

## Overview

OSAL-RS provides a unified API for developing multi-platform embedded applications in Rust. It abstracts operating system-specific functionality, allowing you to write portable code that can run on different platforms with minimal changes.

### Workspace Components

- **osal-rs**: Main Operating System Abstraction Layer with FreeRTOS support
- **osal-rs-build**: Build configuration tools and helpers
- **osal-rs-porting**: C FFI bridge layer for FreeRTOS integration
- **osal-rs-tests**: Comprehensive test suite for all components
- **osal-rs-serde**: ✨ Extensible serialization/deserialization framework with derive macros

## Current Implementation Status

- ✅ **FreeRTOS**: Fully implemented and tested
- ✅ **Serialization**: Complete osal-rs-serde implementation with derive macros
- 🚧 **POSIX**: Planned for future releases
- 🚧 **Other RTOSes**: Under consideration

## Features

### Core OSAL Features
- **Thread Management**: Create, manage, and synchronize threads with priorities
- **Synchronization Primitives**: Mutexes (recursive & non-recursive), binary & counting semaphores, event groups
- **Message Queues**: Type-safe inter-thread communication with blocking/non-blocking operations
- **Software Timers**: Periodic and one-shot timers with callbacks
- **Memory Allocation**: Custom allocator integration for heap management
- **Time Management**: Duration handling and tick-based timing
- **System Control**: Scheduler control, task notifications, and system information
- **No-std Support**: Fully compatible with bare-metal embedded systems

### 🆕 osal-rs-serde Features

A complete serialization framework designed specifically for embedded systems:

- **No-std Compatible**: Works in bare-metal environments without standard library
- **Zero-Copy**: Direct buffer operations with no intermediate allocations
- **Derive Macros**: Automatic `#[derive(Serialize, Deserialize)]` implementation
- **Rich Type Support**: Primitives, arrays, tuples, Option<T>, Vec<T>, nested structs
- **Extensible Architecture**: Create custom serializers for any format (JSON, MessagePack, CBOR, etc.)
- **Memory Efficient**: Little-endian binary format with predictable sizes
- **Compile-Time Guarantees**: Type-safe serialization with static checks
- **Standalone**: Can be used independently in any Rust project

#### osal-rs-serde Quick Example

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct SensorData {
    temperature: i16,
    humidity: u8,
    pressure: u32,
    status: Option<u8>,
}

let data = SensorData { 
    temperature: 25, 
    humidity: 60, 
    pressure: 1013,
    status: Some(0xFF),
};

// Serialize to stack buffer
let mut buffer = [0u8; 32];
let len = to_bytes(&data, &mut buffer).unwrap();

// Deserialize from buffer
let restored: SensorData = from_bytes(&buffer[..len]).unwrap();
assert_eq!(data, restored);
```

#### Integration with OSAL Queues

Perfect for inter-task communication:

```rust
use osal_rs::os::{Queue, QueueFn};
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct Command {
    id: u32,
    params: [u16; 4],
}

fn sender_task(queue: &Queue) {
    let cmd = Command { id: 42, params: [1, 2, 3, 4] };
    let mut buffer = [0u8; 32];
    let len = to_bytes(&cmd, &mut buffer).unwrap();
    queue.post(&buffer[..len], 100).unwrap();
}

fn receiver_task(queue: &Queue) {
    let mut buffer = [0u8; 32];
    queue.fetch(&mut buffer, 100).unwrap();
    let cmd: Command = from_bytes(&buffer).unwrap();
}
```

For comprehensive documentation, examples, and advanced features, see:
- [osal-rs-serde README](osal-rs-serde/README.md) - Complete feature documentation
- [osal-rs-serde/derive README](osal-rs-serde/derive/README.md) - Derive macro guide
- `osal-rs-serde/examples/` - Working code examples

## Prerequisites

Before using OSAL-RS in your project, ensure that:

1. **FreeRTOS is properly configured** in your project
2. **FreeRTOS is linked** to your final binary
3. **C porting layer files** from `osal-rs-porting` must be compiled and linked to your project
4. **CMake build system** is set up for your embedded project
5. **Rust toolchain** with appropriate target support is installed

### Configuration

OSAL-RS requires proper FreeRTOS configuration. Ensure your `FreeRTOSConfig.h` includes:

```c
#define configUSE_MUTEXES                1
#define configUSE_RECURSIVE_MUTEXES      1
#define configUSE_COUNTING_SEMAPHORES    1
#define configUSE_TIMERS                 1
#define configUSE_QUEUE_SETS             1
#define configSUPPORT_DYNAMIC_ALLOCATION 1
```

## CMake Integration

OSAL-RS is designed to be integrated into CMake-based projects. Here are several integration examples:

**Important**: Always ensure that the C porting layer files from `osal-rs-porting/freertos/` are compiled and linked to your project, as they provide the necessary FFI bridge between Rust and FreeRTOS.

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

### Custom FreeRTOS Configuration Path

By default, OSAL-RS looks for `FreeRTOSConfig.h` at `<workspace_root>/inc/FreeRTOSConfig.h`. You can override this path using the `FREERTOS_CONFIG_PATH` environment variable.

#### Setting via CMake

```cmake
# Set custom path to FreeRTOSConfig.h
set(FREERTOS_CONFIG_PATH "${CMAKE_SOURCE_DIR}/inc/hhg-config/pico/FreeRTOSConfig.h")

# Pass to Cargo build via environment variable
add_custom_command(
    OUTPUT ${OSAL_RS_LIB}
    COMMAND ${CMAKE_COMMAND} -E env FREERTOS_CONFIG_PATH=${FREERTOS_CONFIG_PATH}
            cargo build --release --target ${RUST_TARGET} --features freertos
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/osal-rs
    COMMENT "Building OSAL-RS library"
)
```

#### Setting via Environment Variable

```bash
# Set environment variable before building
export FREERTOS_CONFIG_PATH="/path/to/your/FreeRTOSConfig.h"
cargo build --release --target thumbv7em-none-eabihf --features freertos
```

#### Using with Corrosion

```cmake
# Set environment variable for Corrosion
set(FREERTOS_CONFIG_PATH "${CMAKE_SOURCE_DIR}/inc/custom/FreeRTOSConfig.h")

corrosion_import_crate(
    MANIFEST_PATH osal-rs/Cargo.toml
    FEATURES freertos
)

# Set environment for the build
corrosion_set_env_vars(osal-rs
    FREERTOS_CONFIG_PATH=${FREERTOS_CONFIG_PATH}
)
```

**Note**: The build system will automatically regenerate Rust type bindings from the specified `FreeRTOSConfig.h` during the build process.


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

## Cargo Features

OSAL-RS provides several Cargo features to customize the build configuration for different platforms and use cases:

### Available Features

| Feature | Default | Description |
|---------|---------|-------------|
| `freertos` | ✅ | Enable FreeRTOS backend implementation. This is the default and fully implemented feature for embedded RTOS development. |
| `posix` | ❌ | Enable POSIX backend implementation. Currently planned for future releases to support Linux/Unix-like systems. |
| `std` | ❌ | Enable standard library support. Automatically enables `disable_panic`. Use this for native development and testing environments. |
| `disable_panic` | ❌ | Disable custom panic handler. Enabled automatically when `std` feature is active. Useful when you want to use the default panic behavior. |
| `serde` | ❌ | Enable serialization/deserialization support via `osal-rs-serde`. Includes derive macros for automatic implementation. |

### Feature Combinations

#### FreeRTOS Embedded Development (Default)
```bash
cargo build --target thumbv7em-none-eabihf --features freertos
```

#### FreeRTOS with Serialization Support
```bash
cargo build --target thumbv7em-none-eabihf --features freertos,serde
```

#### Native Development with Standard Library
```bash
cargo build --features posix,std
```

#### Native Development with Serialization
```bash
cargo build --features posix,std,serde
```

### Using Features in Cargo.toml

To use OSAL-RS in your project with specific features:

```toml
[dependencies]
osal-rs = { version = "0.3", features = ["freertos"] }

# Or with serialization support
osal-rs = { version = "0.3", features = ["freertos", "serde"] }
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

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## Author

Antonio Salsi - [passy.linux@zresa.it](mailto:passy.linux@zresa.it)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Links

- [Repository](https://github.com/HiHappyGarden/osal-rs)
- [Documentation](https://docs.rs/osal-rs)
- [Crates.io](https://crates.io/crates/osal-rs)

## Example implementation

[https://github.com/HiHappyGarden/hi-happy-garden-rs](https://github.com/HiHappyGarden/hi-happy-garden-rs)
