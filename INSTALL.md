# OSAL-RS Installation Guide

## Prerequisites

### Required Tools

1. **Rust** (edition 2021 or later)
   ```bash
   # Install Rust via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **CMake** (version 3.15 or later)
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install cmake build-essential git
   
   # Fedora/RHEL
   sudo dnf install cmake gcc git
   
   # Arch Linux
   sudo pacman -S cmake base-devel git
   
   # macOS
   brew install cmake git
   ```

3. **C Compiler** (gcc or clang)
   Usually installed with build-essential on Linux

4. **Git**
   Required for CMake to download FreeRTOS from GitHub

## Installation

### Option 1: Clone from Git (if published)

```bash
git clone https://github.com/your-username/osal-rs.git
cd osal-rs
cargo build --release
```

### Option 2: Local Development

```bash
# Navigate to the osal-rs directory
cd ~/osal-rs

# Build the project
cargo build --release

# Run tests
cargo test

# Run example
cargo run --example basic
```

## Using OSAL-RS in Your Project

### Add as Local Dependency

In your project's `Cargo.toml`:

```toml
[dependencies]
osal-rs = { path = "../osal-rs" }
```

### Add as Git Dependency (if published)

```toml
[dependencies]
osal-rs = { git = "https://github.com/your-username/osal-rs.git", tag = "v0.1.0" }
```

## Verification

### 1. Check Build

```bash
cd ~/osal-rs
cargo build
```

Expected output:
```
Compiling osal-rs v0.1.0
warning: osal-rs@0.1.0: FreeRTOS kernel built at: ...
Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

### 2. Verify FreeRTOS Download

```bash
find target/debug/build -name "freertos_kernel-src" -type d
```

Should show the downloaded FreeRTOS kernel directory.

### 3. Verify Library

```bash
find target -name "libfreertos.a" -exec ls -lh {} \;
```

Should show:
- Debug: ~315KB
- Release: ~136KB

### 4. Run Example

```bash
cargo run --example basic
```

Expected output:
```
===========================================
  OSAL-RS - Operating System Abstraction Layer
===========================================

FreeRTOS Kernel Version: V11.2.0

Inizializzazione OSAL...
✓ OSAL inizializzato con successo!
```

## First Build

The first build will:
1. Download dependencies from crates.io (~4 packages)
2. Download FreeRTOS Kernel v11.2.0 from GitHub (~2MB)
3. Compile FreeRTOS kernel (~20-30 seconds)
4. Compile Rust code

**Estimated time:** 20-40 seconds depending on your system

Subsequent builds are much faster (<1 second for incremental builds).

## Troubleshooting

### Error: CMake not found

```bash
# Verify CMake installation
cmake --version

# If not installed, install it
sudo apt-get install cmake  # Ubuntu/Debian
```

### Error: Could not find git

```bash
# Install git
sudo apt-get install git  # Ubuntu/Debian
```

### Error: C compiler cannot create executables

```bash
# Install build tools
sudo apt-get install build-essential  # Ubuntu/Debian
```

### Error: Failed to download FreeRTOS

Check your internet connection. CMake needs to access:
```
https://github.com/FreeRTOS/FreeRTOS-Kernel.git
```

If behind a proxy, configure git:
```bash
git config --global http.proxy http://proxy:port
```

### Warning: unused variable 'out_dir'

This is expected and can be ignored. It's a harmless warning in build.rs.

## Development Setup

For contributing to OSAL-RS:

```bash
# Clone the repository
cd ~/osal-rs

# Install development dependencies
cargo install cargo-watch cargo-expand

# Run in watch mode
cargo watch -x build

# Format code
cargo fmt

# Run clippy
cargo clippy
```

## Platform-Specific Notes

### Linux (Primary Platform)
- Uses FreeRTOS Posix port for simulation
- Fully supported and tested

### macOS
- Should work with Posix port
- Requires Xcode Command Line Tools
- Not extensively tested

### Windows
- May require additional setup
- MinGW or MSVC toolchain
- Windows port configuration needed
- Experimental support

## Building for Embedded Targets

To use OSAL-RS on embedded systems:

1. Modify `CMakeLists.txt` to use appropriate FreeRTOS port
2. Configure `FreeRTOSConfig.h` for your MCU
3. Set up cross-compilation toolchain
4. Update `.cargo/config.toml` with target specification

Example for ARM Cortex-M4:
```toml
[target.thumbv7em-none-eabihf]
rustflags = ["-C", "link-arg=-nostartfiles"]
```

See `DEVELOPMENT.md` for detailed embedded setup instructions.

## Next Steps

After installation:

1. Read `README.md` for project overview
2. Check `DEVELOPMENT.md` for architecture details
3. Explore `examples/` directory
4. Review `CHANGELOG.md` for version history

## Support

For issues or questions:
- Check GitHub Issues (if published)
- Review troubleshooting section above
- Read FreeRTOS documentation: https://www.freertos.org/

## License

OSAL-RS is licensed under GPL-3 License.
FreeRTOS Kernel is licensed under MIT License by Amazon.
