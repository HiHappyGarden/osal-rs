# osal-rs-build

Build utilities for [osal-rs](https://github.com/HiHappyGarden/osal-rs) - FreeRTOS type generation at compile time.

[![Crates.io](https://img.shields.io/crates/v/osal-rs-build.svg)](https://crates.io/crates/osal-rs-build)
[![Documentation](https://docs.rs/osal-rs-build/badge.svg)](https://docs.rs/osal-rs-build)
[![License: LGPL-2.1](https://img.shields.io/badge/License-LGPL%202.1-blue.svg)](LICENSE)

## Overview

`osal-rs-build` is a build-time utility crate that automatically generates Rust type mappings for FreeRTOS primitives. It queries the sizes of FreeRTOS types and creates corresponding Rust type definitions, ensuring type safety and compatibility across different target platforms.

## Features

- **Automatic Type Detection**: Detects FreeRTOS type sizes at build time
- **Platform Agnostic**: Works with different architectures (ARM Cortex-M, RISC-V, etc.)
- **Compile-time Generation**: Creates type mappings during the build process
- **Zero Runtime Overhead**: All processing happens at build time

## Supported FreeRTOS Types

The crate generates Rust mappings for the following FreeRTOS types:

| FreeRTOS Type | Description | Rust Mapping |
|---------------|-------------|--------------|
| `TickType_t` | Timer tick counter | `u8`, `u16`, `u32`, or `u64` |
| `UBaseType_t` | Unsigned base type | `u8`, `u16`, `u32`, or `u64` |
| `BaseType_t` | Signed base type | `i8`, `i16`, `i32`, or `i64` |
| `StackType_t` | Stack element type | `u8`, `u16`, `u32`, or `u64` |

## Installation

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
osal-rs-build = "0.4"
```

## Usage

In your `build.rs` file:

```rust
use osal_rs_build::FreeRtosTypeGenerator;

fn main() {
    // Create a generator and generate types
    let generator = FreeRtosTypeGenerator::new();
    generator.generate_types();
    
    // Or generate everything (types and config)
    generator.generate_all();
}
```

### With Custom FreeRTOS Config

If you have a custom `FreeRTOSConfig.h` location:

```rust
use osal_rs_build::FreeRtosTypeGenerator;

fn main() {
    let generator = FreeRtosTypeGenerator::with_config_path("path/to/FreeRTOSConfig.h");
    generator.generate_all();
}
```

### In Your Rust Code

After running the build script, include the generated types in your code:

```rust
// In your lib.rs or main.rs
include!(concat!(env!("OUT_DIR"), "/types_generated.rs"));

// Now you can use the generated types
fn example_task() {
    let tick: TickType = 1000;
    let priority: UBaseType = 5;
}
```

## How It Works

1. **Build Time Detection**: The generator creates a small C program to query FreeRTOS type sizes
2. **Compilation**: Compiles and executes the query program using GCC
3. **Type Mapping**: Maps detected sizes to appropriate Rust types
4. **Code Generation**: Writes generated type aliases to `types_generated.rs` in the `OUT_DIR`

### Default Values

If type detection fails (e.g., no C compiler available), the generator falls back to default values suitable for 32-bit ARM Cortex-M platforms (like Raspberry Pi Pico):

- `TickType_t`: 4 bytes → `u32`
- `UBaseType_t`: 4 bytes → `u32`
- `BaseType_t`: 4 bytes (signed) → `i32`
- `StackType_t`: 4 bytes → `u32`

## Requirements

- Rust 1.85.0 or later
- GCC compiler (for type detection, optional)
- FreeRTOS headers (optional, for custom configurations)

## Example Projects

This crate is used by:

- [osal-rs](https://github.com/HiHappyGarden/osal-rs) - Operating System Abstraction Layer for Rust
- [hi-happy-garden-rs](https://github.com/HiHappyGarden/hi-happy-garden-rs) - Embedded Rust project for Raspberry Pi Pico

## Build Script Example

Complete `build.rs` example:

```rust
use osal_rs_build::FreeRtosTypeGenerator;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Create the generator
    let generator = FreeRtosTypeGenerator::new();
    
    // Generate FreeRTOS type mappings
    generator.generate_types();
    
    // The generated file will be available at:
    // ${OUT_DIR}/types_generated.rs
}
```

## API Documentation

### `FreeRtosTypeGenerator`

The main struct for generating FreeRTOS type mappings.

#### Methods

- `new() -> Self`: Creates a new generator with default settings
- `with_config_path<P: Into<PathBuf>>(config_path: P) -> Self`: Creates a generator with a custom FreeRTOS config path
- `set_config_path<P: Into<PathBuf>>(&mut self, config_path: P)`: Sets the FreeRTOS config path
- `generate_types(&self)`: Generates only type mappings
- `generate_all(&self)`: Generates types and configuration constants

## License

This project is licensed under the GNU Lesser General Public License v2.1 or later - see the [LICENSE](LICENSE) file for details.

## Author

Antonio Salsi - [passy.linux@zresa.it](mailto:passy.linux@zresa.it)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Links

- [Repository](https://github.com/HiHappyGarden/osal-rs)
- [Documentation](https://docs.rs/osal-rs-build)
- [Crates.io](https://crates.io/crates/osal-rs-build)
