# OSAL-RS

Operating System Abstraction Layer in Rust with support for FreeRTOS kernel v11.2.0 and POSIX.

## Features

- **Dual Backend**: Support for FreeRTOS kernel v11.2.0 and POSIX systems
- **Feature Flags**: Backend selection via Cargo feature flags
- **Automated Build**: Uses CMake to download and compile FreeRTOS from GitHub (when needed)
- **Unified API**: Common interface for both backends
- **No_std**: Compatible with embedded environments without standard library

## Supported Backends

### FreeRTOS (default)
- **Feature flag**: `freertos`
- **Description**: Complete integration of FreeRTOS kernel v11.2.0
- **Usage**: Embedded and real-time systems

### POSIX
- **Feature flag**: `posix`
- **Description**: Implementation based on POSIX APIs
- **Usage**: Unix-like systems (Linux, macOS, BSD)

## Project Structure

```
osal-rs/
├── Cargo.toml              # Rust project configuration
├── build.rs                # Conditional build script
├── CMakeLists.txt          # CMake configuration for FreeRTOS
├── cmake/
│   └── FreeRTOS.cmake      # Script to download FreeRTOS v11.2.0
├── include/
│   └── FreeRTOSConfig.h    # FreeRTOS configuration
└── src/
    ├── lib.rs              # Main module with feature flags
    ├── freertos.rs         # FreeRTOS module
    ├── posix.rs            # POSIX module
    ├── freertos/           # FreeRTOS implementations
    │   ├── task.rs
    │   ├── queue.rs
    │   └── semaphore.rs
    └── posix/              # POSIX implementations
        ├── task.rs
        ├── queue.rs
        └── semaphore.rs
```

## Build

The project supports two compilation modes via feature flags.

### Prerequisites

#### For FreeRTOS
- Rust (edition 2021 or higher)
- CMake 3.15+
- C compiler (gcc, clang, etc.)
- Git

#### For POSIX
- Rust (edition 2021 or higher)
- Unix-like system (Linux, macOS, BSD)

### Compilation

#### Con FreeRTOS (default)
```bash
cargo build
# oppure esplicitamente
cargo build --features freertos
```

##### Custom FreeRTOS Configuration

You can customize the hardware port and heap implementation of FreeRTOS via environment variables:

```bash
# Custom port (default: auto-detection based on system)
export FREERTOS_PORT="GCC/ARM_CM4F"

# Custom heap implementation (default: 4)
export FREERTOS_HEAP="2"

# Build with custom configuration
cargo build --features freertos
```

**Common values for FREERTOS_PORT:**
- `ThirdParty/GCC/Posix` - Simulation on POSIX systems
- `GCC/ARM_CM4F` - ARM Cortex-M4F
- `GCC/ARM_CM3` - ARM Cortex-M3
- `GCC/ARM_CM0` - ARM Cortex-M0

**Values for FREERTOS_HEAP:**
- `1` - Simple allocation, no deallocation
- `2` - Simple allocation/deallocation
- `3` - Wrapper for standard malloc/free
- `4` - With coalescence (default)
- `5` - Multiple regions

See [FREERTOS_CONFIG.md](FREERTOS_CONFIG.md) for complete details.

#### With POSIX
```bash
cargo build --no-default-features --features posix
```

#### Examples
```bash
# Example with FreeRTOS
cargo run --example basic

# Example with POSIX
cargo run --example basic --no-default-features --features posix
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
osal-rs = { path = "../osal-rs" }

# For POSIX only
osal-rs = { path = "../osal-rs", default-features = false, features = ["posix"] }
```

### Unified API

```rust
use osal_rs::{os, os_version, init};

fn main() {
    println!("System: {}", os_version());
    init();
    
    // The API is the same for both backends
    let task = os::task::Task::new();
}
```

### Conditional Feature Flags

```rust
#[cfg(feature = "freertos")]
fn freertos_specific() {
    // FreeRTOS specific code
}

#[cfg(feature = "posix")]
fn posix_specific() {
    // POSIX specific code
}
```

### Reading FreeRTOS Configuration

You can read FreeRTOS configuration constants directly from Rust. **The values are automatically synchronized** from `FreeRTOSConfig.h` during compilation:

```rust
use osal_rs::constants::*;

fn main() {
    // Read CONFIG_TICK_RATE_HZ (auto-synced from FreeRTOSConfig.h)
    println!("Tick rate: {} Hz", CONFIG_TICK_RATE_HZ);
    
    // Convert milliseconds to ticks
    let delay_ticks = ms_to_ticks(500);
    println!("500ms = {} ticks", delay_ticks);
    
    // Other constants
    println!("Max priorities: {}", CONFIG_MAX_PRIORITIES);
    println!("Heap size: {} bytes", CONFIG_TOTAL_HEAP_SIZE);
}
```

**Automatic Synchronization**: When you modify `include/FreeRTOSConfig.h`, the Rust constants are automatically updated during the next `cargo build`. No manual synchronization needed! ✨

See [AUTO_SYNC.md](AUTO_SYNC.md), [FREERTOS_CONSTANTS.md](FREERTOS_CONSTANTS.md) and [QUICK_START_CONFIG.md](QUICK_START_CONFIG.md) for more details.

## Testing

OSAL-RS includes comprehensive tests for verifying code correctness. Due to the nature of FreeRTOS requiring an embedded environment, tests are categorized into:

- **Unit Tests**: Logic verification without requiring RTOS runtime (can run on host)
- **Integration Tests**: Require actual FreeRTOS or POSIX runtime

For detailed testing instructions, see [TESTING.md](TESTING.md).

Quick start:
```bash
# Check compilation (no execution needed)
cargo check --lib --features freertos

# Run unit tests (logic only, no RTOS execution)
cargo test --lib --features freertos --doc
```

## FreeRTOS Kernel

When using the `freertos` feature, the project automatically downloads the FreeRTOS kernel from the official repository:
- **Repository**: https://github.com/FreeRTOS/FreeRTOS-Kernel
- **Version**: v11.2.0
- **Git Tag**: V11.2.0

## Additional Documentation

- [FEATURES.md](FEATURES.md) - Detailed guide to using feature flags
- [FREERTOS_CONFIG.md](FREERTOS_CONFIG.md) - Advanced FreeRTOS configuration
- [TESTING.md](TESTING.md) - Complete testing guide

## License

GPL-3

## References

- [FreeRTOS Official Website](https://www.freertos.org/)
- [FreeRTOS Kernel GitHub](https://github.com/FreeRTOS/FreeRTOS-Kernel)
- [POSIX Standard](https://pubs.opengroup.org/onlinepubs/9699919799/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
