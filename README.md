# OSAL-RS

Operating System Abstraction Layer for Rust - A cross-platform compatibility layer for embedded and real-time systems development.  

[![Crates.io](https://img.shields.io/crates/v/osal-rs.svg)](https://crates.io/crates/osal-rs)
[![Documentation](https://docs.rs/osal-rs/badge.svg)](https://docs.rs/osal-rs)
[![License: LGPL-2.1](https://img.shields.io/badge/License-LGPL%202.1-blue.svg)](LICENSE)  
[![Tests](https://github.com/HiHappyGarden/osal-rs/actions/workflows/test.yml/badge.svg)](https://github.com/HiHappyGarden/osal-rs/actions/workflows/test.yml)

## Overview

OSAL-RS provides a unified API for developing multi-platform embedded applications in Rust. It abstracts operating system-specific functionality, allowing you to write portable code that can run on different platforms with minimal changes.

### Workspace Components

- **osal-rs**: Main Operating System Abstraction Layer, with **FreeRTOS** and **POSIX** backends
- **osal-rs-build**: Build configuration tools and helpers (FreeRTOS type generation, POSIX porting shim compilation)
- **osal-rs-porting**: C FFI bridge layer for the FreeRTOS and POSIX backends
- **osal-rs-tests**: Comprehensive test suite for all components
- **osal-rs-serde**: ✨ Extensible serialization/deserialization framework with derive macros

## Current Implementation Status

- ✅ **FreeRTOS**: Fully implemented and tested
- ✅ **POSIX**: Fully implemented and tested (glibc/Linux) - native host backend for running, testing and simulating OSAL-RS applications without embedded hardware
- ✅ **Serialization**: Complete osal-rs-serde implementation with derive macros
- 🧪 **Async/Await**: Experimental, backend-agnostic, works on both FreeRTOS and POSIX
- 🚧 **Other RTOSes**: Under consideration

## Supported Backends

OSAL-RS selects its implementation at compile time via Cargo features. There is **no default backend** - exactly one of `freertos` / `posix` must be enabled explicitly, or the crate fails to build. Enabling neither trips a `compile_error!`; enabling both is equally unsupported, since the two backends are mutually exclusive by design.

### FreeRTOS Backend (`freertos`)

For bare-metal embedded targets (`no_std`). FreeRTOS provides preemptive multitasking with priority-based scheduling, mutexes with priority inheritance, semaphores, queues and software timers.

**Requirements:**
1. FreeRTOS kernel properly configured and linked into your project
2. C porting layer from `osal-rs-porting/freeretos/` compiled and linked (an FFI bridge between Rust and FreeRTOS)
3. CMake build system set up for your embedded project (see [CMake Integration](#cmake-integration))
4. Rust toolchain with the appropriate embedded target installed

**Configuration** - ensure your `FreeRTOSConfig.h` includes:

```c
#define configTICK_RATE_HZ               1000
#define configUSE_MUTEXES                1
#define configUSE_RECURSIVE_MUTEXES      1
#define configUSE_COUNTING_SEMAPHORES    1
#define configUSE_TIMERS                 1
#define configUSE_QUEUE_SETS             1
#define configSUPPORT_DYNAMIC_ALLOCATION 1
```

**Build:**

```bash
# Install Rust target (example for ARM Cortex-M33)
rustup target add thumbv8m.main-none-eabi

# Build with FreeRTOS support
cargo build --release --target thumbv8m.main-none-eabi --features freertos
```

### POSIX Backend (`posix`)

Runs on any POSIX/pthreads host so OSAL-RS applications - and their tests and doc examples - can execute for real on Linux/macOS without embedded hardware or a cross toolchain. Enabling `posix` disables `no_std` and builds the crate against `std`.

**Requirements:**
- A **glibc**/Linux host (see note below)
- No special build steps: unlike `freertos`, the small POSIX porting shim in `osal-rs-porting/posix/` is compiled and linked automatically by `osal-rs-build` - no CMake, cross toolchain or RTOS kernel sources required

**Build:**

```bash
# Native development / host testing
cargo build --features posix
```

#### POSIX Backend: glibc Requirement

The `posix` backend links directly against **glibc** (the GNU C Library), not just any C compiler. It relies on glibc-specific internals - struct layouts (`pthread_attr_t`, `pthread_mutex_t`, `pthread_cond_t`, `sigset_t`, etc.) and the `__libc_current_sigrtmin()` extension used to implement thread suspend/resume via real-time signals.

This means:
- The **compiler** doesn't matter - gcc or clang both work fine.
- The **C library** does matter - targets linking against **musl** (e.g. `x86_64-unknown-linux-musl`) or non-glibc platforms (e.g. macOS/BSD libc) are **not supported** by the `posix` backend.

#### Real-Time Scheduling (`real_time`)

Threads spawned by the `posix` backend normally inherit the creating thread's scheduling policy/priority. The `real_time` feature switches them to the real-time `SCHED_FIFO` policy instead.

You don't need to request this feature yourself: `osal-rs-build`'s build script probes the host at compile time and automatically turns `real_time` on whenever the OS/kernel supports `SCHED_FIFO`. It's a plain Cargo feature only so it can be inspected via `cfg(feature = "real_time")`; a plain `cargo build --features posix` is enough to get it on a capable host.

### Caveats of the POSIX Backend

- `System::start()` simply spins until [`System::stop()`] is called from another thread - there is no scheduler to hand control to, unlike FreeRTOS where it never returns.
- Timers each spawn their own background thread and permanently block `SIGALRM` on the thread that creates them; create a new `Timer` rather than reusing one that already fired as a one-shot.

## Quick Start

```rust
use osal_rs::os::*;

fn main() {
    // Create a thread
    let mut thread = Thread::new(
        "worker",
        4096,  // stack size
        5,     // priority
    );

    thread.spawn_simple(|| {
        loop {
            println!("Working...");
            System::delay(1000);
        }
    }).unwrap();

    // Start the scheduler (never returns on FreeRTOS; spins until `System::stop()` on POSIX)
    System::start();
}
```

```rust
use osal_rs::os::*;
use std::sync::Arc;

let counter = Arc::new(Mutex::new(0));
let counter_clone = counter.clone();

let mut thread = Thread::new("incrementer", 2048, 5);
thread.spawn_simple(move || {
    let mut guard = counter_clone.lock().unwrap();
    *guard += 1;
    Ok(Arc::new(()))
}).unwrap();
```

```rust
use osal_rs::os::*;

let queue = Queue::new(10, 4).unwrap();

// Send data
let data = [1u8, 2, 3, 4];
queue.post(&data, 100).unwrap();

// Receive data
let mut buffer = [0u8; 4];
queue.fetch(&mut buffer, 100).unwrap();
```

The same code compiles and runs unchanged against either backend - just switch the `freertos`/`posix` feature flags.

## Core OSAL Features

- **Thread Management**: Create, manage, and synchronize threads with priorities
- **Synchronization Primitives**: Mutexes (recursive & non-recursive), binary & counting semaphores, event groups
- **Message Queues**: Type-safe inter-thread communication with blocking/non-blocking operations
- **Software Timers**: Periodic and one-shot timers with callbacks
- **Memory Allocation**: Custom allocator integration for heap management (`freertos`) or the system allocator (`posix`)
- **Time Management**: Duration handling and tick-based timing
- **System Control**: Scheduler control, task notifications, and system information
- **No-std Support**: Fully compatible with bare-metal embedded systems (`freertos` backend)
- **Host Testing**: Native `std` execution for tests, examples and simulation (`posix` backend)
- **🧪 _EXPERIMENTAL_ Async/Await**: Backend-agnostic `async`/`await` support without Tokio (see below)

## Cargo Features

OSAL-RS provides several Cargo features to customize the build configuration for different platforms and use cases:

### Available Features

| Feature | Default | Description |
|---------|---------|-------------|
| `freertos` | ❌ | Enable the FreeRTOS backend implementation for embedded RTOS development. Mutually exclusive with `posix` - exactly one of the two is required. |
| `posix` | ❌ | Enable the POSIX/native backend implementation for host environments. Requires **glibc** (see note above). Mutually exclusive with `freertos` - exactly one of the two is required. |
| `real_time` | ❌ | POSIX only: schedules spawned threads with the real-time `SCHED_FIFO` policy instead of inheriting the creating thread's policy/priority. Not meant to be requested by hand - `osal-rs-build`'s build script enables it automatically when the host OS/kernel supports `SCHED_FIFO`. |
| `async` | ❌ | Enable backend-agnostic async/await support (`block_on`, `AsyncQueue`, `AsyncSemaphore`, `AsyncMutex`). Works with both `freertos` and `posix`. No Tokio required. |
| `serde` | ❌ | Enable serialization/deserialization support via `osal-rs-serde`. Includes derive macros for automatic implementation. |

There is no default feature set: you must explicitly pick `freertos` or `posix` or the build fails.

### Feature Combinations

```bash
# FreeRTOS embedded development
cargo build --target thumbv8m.main-none-eabi --features freertos

# FreeRTOS with async support
cargo build --target thumbv8m.main-none-eabi --features freertos,async

# FreeRTOS with serialization support
cargo build --target thumbv8m.main-none-eabi --features freertos,serde

# Native development (POSIX) - real_time is auto-detected, no need to request it
cargo build --features posix

# Native development with async support
cargo build --features posix,async

# Native development with serialization
cargo build --features posix,serde
```

### Using Features in Cargo.toml

To use OSAL-RS in your project with specific features (exactly one of `freertos`/`posix` is required):

```toml
[dependencies]
osal-rs = { version = "1.0", features = ["freertos"] }

# Or for host development/testing
osal-rs = { version = "1.0", features = ["posix"] }

# Or with serialization support
osal-rs = { version = "1.0", features = ["freertos", "serde"] }
```

## 🧪 _EXPERIMENTAL_ Async/Await Support (feature `async`)

OSAL-RS includes a **backend-agnostic async runtime** that works on both FreeRTOS and POSIX
without Tokio or any external async runtime.

### Design

| Component | Description |
|-----------|-------------|
| `block_on(future)` | Drives a `Future` to completion on the calling RTOS task |
| `AsyncQueue` | Queue with `fetch_async` / `post_async` methods |
| `AsyncSemaphore` | Semaphore with `wait_async` |
| `AsyncMutex<T>` | Mutex whose `lock()` returns a `Future` |

- **No Tokio, no `std`**: built on `core::future::Future` + OSAL semaphores as the blocking primitive.
- **Per-task executor**: `block_on` runs on the calling RTOS task; no thread pool is needed.
- **Lock-free waker storage**: `WakerSlot` uses `AtomicPtr<Waker>` - no RTOS overhead for waker updates.
- **Race-condition safe**: the classic *store-waker-then-retry* double-check pattern is used in every `poll` implementation.

### Quick Example

```rust
use osal_rs::os::{block_on, AsyncMutex, AsyncQueue, AsyncSemaphore};

// Run async code inside any RTOS task — no runtime setup required
block_on(async {
    // Async mutex
    let mutex = AsyncMutex::new(0u32);
    {
        let mut guard = mutex.lock().await;
        *guard += 1;
    }

    // Async semaphore (signal from another task or ISR)
    let sem = AsyncSemaphore::new(1, 0).unwrap();
    sem.signal();
    sem.wait_async().await;

    // Async queue
    let queue = AsyncQueue::new(8, 4).unwrap();
    queue.post_async(&[1, 2, 3, 4]).await.unwrap();
    let mut buf = [0u8; 4];
    queue.fetch_async(&mut buf).await.unwrap();
});
```

### Enable the feature

```toml
# Cargo.toml
[dependencies]
osal-rs = { version = "1.0", features = ["freertos", "async"] }
# or for host development
osal-rs = { version = "1.0", features = ["posix", "async"] }
```

```bash
# FreeRTOS embedded target
cargo build --release --target thumbv8m.main-none-eabi --features freertos,async

# POSIX host (for tests / simulation)
cargo build --features posix,async
```

## osal-rs-serde Features

A complete serialization framework designed specifically for embedded systems:

- **No-std Compatible**: Works in bare-metal environments without standard library
- **Zero-Copy**: Direct buffer operations with no intermediate allocations
- **Derive Macros**: Automatic `#[derive(Serialize, Deserialize)]` implementation
- **Rich Type Support**: Primitives, arrays, tuples, Option<T>, Vec<T>, nested structs
- **Extensible Architecture**: Create custom serializers for any format (JSON, MessagePack, CBOR, etc.)
- **Memory Efficient**: Little-endian binary format with predictable sizes
- **Compile-Time Guarantees**: Type-safe serialization with static checks
- **Standalone**: Can be used independently in any Rust project

### osal-rs-serde Quick Example

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

### Integration with OSAL Queues

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

## CMake Integration

CMake integration is only needed for the **FreeRTOS** backend, since it must link against your project's FreeRTOS kernel and C porting layer. The **POSIX** backend needs no CMake step - `cargo build --features posix` is enough (see [Supported Backends](#supported-backends)).

**Important**: Always ensure that the C porting layer files from `osal-rs-porting/freeretos/` are compiled and linked to your project, as they provide the necessary FFI bridge between Rust and FreeRTOS.

### Basic CMake Integration

Add OSAL-RS to your existing CMake project:

```cmake
cmake_minimum_required(VERSION 3.20)
project(my_embedded_project C CXX)

# Configure FreeRTOS (assuming it's already in your project)
add_subdirectory(freertos)

# Add OSAL-RS porting layer
add_library(osal_rs_porting STATIC
    osal-rs-porting/freeretos/src/osal_rs.c
)

target_include_directories(osal_rs_porting PUBLIC
    osal-rs-porting/freeretos/inc
    ${FREERTOS_INCLUDE_DIRS}
)

target_link_libraries(osal_rs_porting PUBLIC
    freertos
)

# Configure Rust library
set(RUST_TARGET "thumbv8m.main-none-eabi")  # Adjust for your target
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
add_osal_rs_library(osal_rs "thumbv8m.main-none-eabi" "release")
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
set(RUST_TARGET "thumbv8m.main-none-eabi")
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
cargo build --release --target thumbv8m.main-none-eabi --features freertos
```

**Note**: The build system will automatically regenerate Rust type bindings from the specified `FreeRTOSConfig.h` during the build process.

## Project Structure

```
osal-rs/
├── osal-rs/              # Main library crate (freertos + posix backends)
├── osal-rs-build/        # Build utilities
├── osal-rs-tests/        # Test suite
├── osal-rs-serde/        # Serialization framework
└── osal-rs-porting/      # Platform-specific C/C++ code
    ├── freeretos/        # FreeRTOS porting layer
    │   ├── inc/          # Header files
    │   └── src/          # Implementation
    └── posix/            # POSIX porting layer (glibc shim, built automatically)
        ├── inc/          # Header files
        └── src/          # Implementation
```

## License

This project is licensed under the LGPL-2.1-or-later License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## Author

Antonio Salsi - [passy.linux@zresa.it](mailto:passy.linux@zresa.it)

## Links

- [Repository](https://github.com/HiHappyGarden/osal-rs)
- [Documentation](https://docs.rs/osal-rs)
- [Crates.io](https://crates.io/crates/osal-rs)

## Example implementation

[https://github.com/HiHappyGarden/hi-happy-garden-rs](https://github.com/HiHappyGarden/hi-happy-garden-rs)
