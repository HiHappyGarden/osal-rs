# osal-rs-build

Build-time utilities for [osal-rs](https://github.com/HiHappyGarden/osal-rs)'s `build.rs` - platform type generation and, for the POSIX backend, C porting layer compilation.

[![Crates.io](https://img.shields.io/crates/v/osal-rs-build.svg)](https://crates.io/crates/osal-rs-build)
[![Documentation](https://docs.rs/osal-rs-build/badge.svg)](https://docs.rs/osal-rs-build)
[![License: LGPL-2.1](https://img.shields.io/badge/License-LGPL%202.1-blue.svg)](LICENSE)

## Overview

`osal-rs-build` is the build-time helper crate behind `osal-rs`'s two backends. Its [`TypeGenerator`] type:

- Detects the active backend's `TickType`/`UBaseType`/`BaseType`/`StackType` sizes and writes them as Rust type aliases into `OUT_DIR/types_generated.rs` (included by `osal-rs` via `include!`)
- Wires up `cargo:rerun-if-changed` for the relevant C porting sources
- For the `posix` backend, also compiles and statically links the C porting layer (`osal-rs-porting/posix/`)
- Detects whether the host supports real-time (`SCHED_FIFO`) scheduling and turns on the `real_time` cfg for `osal-rs` accordingly - this is automatic, not something the consuming crate has to request

Exactly one of the `freertos` / `posix` features is expected to be enabled, matching whichever backend `osal-rs` itself is being built with.

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `freertos` | ❌ | Generate types for the FreeRTOS backend. |
| `posix` | ❌ | Generate types for the POSIX backend and compile/link its C porting shim. |

There is no default: pick exactly one, matching the feature enabled on `osal-rs` itself.

## Installation

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
osal-rs-build = { version = "1.0", default-features = false, features = ["posix"] }
# or ["freertos"]
```

## Usage

`osal-rs`'s own `build.rs` is the reference consumer. In your `build.rs`:

```rust
use std::env;
use std::path::PathBuf;
use osal_rs_build::TypeGenerator;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut generator = TypeGenerator::new(&PathBuf::from(manifest_dir));

    TypeGenerator::add_rerun_if_changed();
    generator.generate_all();
}
```

`TypeGenerator::new` takes the crate's manifest directory (`CARGO_MANIFEST_DIR`). For the `freertos` feature it walks up two parent directories to find the workspace root, used to locate `FreeRTOSConfig.h`; for `posix` the argument is accepted but unused.

`generate_all()`:
- **`posix`**: probes the host architecture and `SCHED_FIFO` support by compiling and running a small C program via `gcc`, writes `types_generated.rs`, enables the `real_time` cfg if the probe found `SCHED_FIFO` support
- **`freertos`**: writes `types_generated.rs` (see note on type detection below) and enables the `real_time` cfg if supported

### With a Custom FreeRTOSConfig.h Location

By default the generator looks for `FreeRTOSConfig.h` at `<workspace_root>/inc/FreeRTOSConfig.h`. Override it with the `FREERTOS_CONFIG_PATH` environment variable:

```bash
export FREERTOS_CONFIG_PATH=/path/to/FreeRTOSConfig.h
cargo build --features freertos
```

### In Your Rust Code

The generated types are consumed via `include!`, not as a normal module:

```rust
// In osal-rs's lib.rs (or your own crate built the same way)
include!(concat!(env!("OUT_DIR"), "/types_generated.rs"));

// Now the generated aliases are in scope
fn example_task() {
    let tick: TickType = 1000;
    let priority: UBaseType = 5;
}
```

## Generated Types

Both backends produce the same four Rust type aliases, sized for the architecture:

| Type | Description |
|------|-------------|
| `TickType` | Timer tick counter |
| `UBaseType` | Unsigned base type |
| `BaseType` | Signed base type |
| `StackType` | Stack element type |

- **`posix`**: sizes are derived from the detected host architecture - 8 bytes (`u64`/`i64`) on `x86_64`/`aarch64`/`riscv64`, 4 bytes (`u32`/`i32`) on `x86`/`arm`/`riscv32`. Other architectures fail the build with a clear panic rather than silently guessing.
- **`freertos`**: type detection from an actual `FreeRTOSConfig.h`/toolchain is not implemented yet - the generator currently always emits the common 32-bit mapping (4-byte types), which matches typical Cortex-M configurations (e.g. Raspberry Pi Pico). If your target uses different FreeRTOS type sizes, this crate does not yet detect that.

## Real-Time Scheduling Detection (`SCHED_FIFO`)

For the `posix` backend, the same architecture probe also checks whether the host supports `SCHED_FIFO` real-time scheduling and, if so, emits `cargo:rustc-cfg=feature="real_time"` for the crate being built. This is fully automatic - `osal-rs` consumers don't request or configure it, it just reflects what the host supports.

## Requirements

- Rust 1.85.0 or later
- `gcc` and `ar` available on `PATH` (used to compile/run the architecture probe, and - for `posix` - to build and archive the C porting shim)

## Used By

- [osal-rs](https://github.com/HiHappyGarden/osal-rs) - Operating System Abstraction Layer for Rust
- [hi-happy-garden-rs](https://github.com/HiHappyGarden/hi-happy-garden-rs) - Embedded Rust project for Raspberry Pi Pico

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
