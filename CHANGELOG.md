# OSAL-RS Version History

## Version 0.2.0 - Advanced FreeRTOS Configuration

### New Features
- ✅ **Customizable FreeRTOS configuration**: Support for `FREERTOS_PORT` and `FREERTOS_HEAP` parameters
- ✅ **Environment variables**: Configuration via environment variables in build
- ✅ **Parameter validation**: Automatic validation of parameter validity
- ✅ **Port auto-detection**: Automatic detection of FreeRTOS port based on system
- ✅ **Extended documentation**: Complete configuration guide in `FREERTOS_CONFIG.md`
- ✅ **Example script**: `build_examples.sh` to demonstrate different configurations

### Technical Details
- **FREERTOS_PORT**: Configurable via environment variable (default: auto-detection)
- **FREERTOS_HEAP**: Configurable from 1 to 5 (default: 4)
- **Backward compatibility**: Maintains compatibility with existing builds
- **Informative warnings**: Shows configuration used during build

### Supported Configurations
- **FreeRTOS Ports**: ThirdParty/GCC/Posix, GCC/ARM_CM4F, GCC/ARM_CM3, GCC/ARM_CM0, etc.
- **Heap implementations**: heap_1.c, heap_2.c, heap_3.c, heap_4.c, heap_5.c
- **Auto-detection**: Linux → Posix, ARM → CM4F, Others → Posix

## Version 0.1.0 - Initial Release

### Features
- ✅ FreeRTOS Kernel v11.2.0 integration
- ✅ CMake-based build system
- ✅ Automatic download from GitHub
- ✅ GCC/Posix port support (Linux simulator)
- ✅ Basic Rust API structure (task, queue, semaphore modules)
- ✅ Build script integration via build.rs
- ✅ Example application

### FreeRTOS Details
- **Version**: V11.2.0
- **Repository**: https://github.com/FreeRTOS/FreeRTOS-Kernel
- **Git Tag**: V11.2.0
- **Release Date**: March 04, 2025
- **Download Method**: CMake FetchContent from GitHub

### Build System
- Cargo + CMake integration
- Automatic dependency management
- Static library linking (libfreertos.a)
- Cross-platform support (Linux primary)

### Project Structure
- Rust crate with no_std support
- FFI bindings for FreeRTOS
- Safe Rust wrappers (planned)
- Example programs

### Known Limitations
- Currently supports Posix port only
- FFI bindings are placeholder (to be completed)
- No thread-safe wrappers yet
- Documentation in progress

### Next Steps (v0.2.0)
- [ ] Complete FFI bindings with bindgen
- [ ] Implement safe Rust wrappers for tasks
- [ ] Add queue and semaphore implementations
- [ ] Support additional ports (ARM Cortex-M)
- [ ] Add comprehensive tests
- [ ] Performance benchmarks
