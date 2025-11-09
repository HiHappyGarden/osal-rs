# FreeRTOS Configuration

This document explains how to configure FreeRTOS parameters for the osal-rs project.

## Configurable Parameters

### FREERTOS_PORT

Specifies the hardware/compiler port to use for FreeRTOS.

**Common values:**
- `ThirdParty/GCC/Posix` - For simulation on POSIX systems (Linux/macOS)
- `GCC/ARM_CM4F` - For ARM Cortex-M4F microcontrollers
- `GCC/ARM_CM3` - For ARM Cortex-M3 microcontrollers
- `GCC/ARM_CM0` - For ARM Cortex-M0 microcontrollers
- `GCC/ARM_CM7` - For ARM Cortex-M7 microcontrollers

**Default behavior:**
- On Linux: `ThirdParty/GCC/Posix`
- On ARM processors: `GCC/ARM_CM4F`
- Other systems: `ThirdParty/GCC/Posix`

### FREERTOS_HEAP

Specifies the heap memory implementation to use.

**Possible values:**
- `1` - heap_1.c - Simple allocation, no deallocation
- `2` - heap_2.c - Simple allocation/deallocation, no coalescence
- `3` - heap_3.c - Wrapper for standard malloc/free
- `4` - heap_4.c - Allocation/deallocation with coalescence (default)
- `5` - heap_5.c - Like heap_4 but with multiple memory regions

**Default behavior:** `4` (heap_4.c)

## Usage

### Via environment variables

```bash
# Set port for ARM Cortex-M4F
export FREERTOS_PORT="GCC/ARM_CM4F"

# Set heap implementation 3 (malloc/free wrapper)
export FREERTOS_HEAP="3"

# Build the project
cargo build --features freertos
```

### Via cargo with inline env vars

```bash
# Build with specific parameters
FREERTOS_PORT="GCC/ARM_CM3" FREERTOS_HEAP="1" cargo build --features freertos
```

### Via CMake directly

If using CMake directly (not through cargo):

```bash
mkdir build && cd build
cmake .. -DFREERTOS_PORT="GCC/ARM_CM4F" -DFREERTOS_HEAP="2"
make
```

## Configuration Examples

### Simulation on Linux
```bash
# Use default settings (auto-detected)
cargo build --features freertos

# Or explicitly
FREERTOS_PORT="ThirdParty/GCC/Posix" FREERTOS_HEAP="4" cargo build --features freertos
```

### ARM Cortex-M4F microcontroller
```bash
FREERTOS_PORT="GCC/ARM_CM4F" FREERTOS_HEAP="4" cargo build --features freertos --target thumbv7em-none-eabihf
```

### Microcontroller with limited memory
```bash
# Use heap_1 for systems with very limited memory
FREERTOS_PORT="GCC/ARM_CM0" FREERTOS_HEAP="1" cargo build --features freertos --target thumbv6m-none-eabi
```

### System with custom malloc/free
```bash
# Use heap_3 to integrate with existing allocator
FREERTOS_PORT="GCC/ARM_CM7" FREERTOS_HEAP="3" cargo build --features freertos --target thumbv7em-none-eabihf
```

## Notes

- If you don't specify `FREERTOS_PORT`, the system will try to automatically detect the appropriate port
- If you don't specify `FREERTOS_HEAP`, `heap_4.c` will be used as default
- Parameter validation occurs during CMake configuration
- Cargo warning messages will show the values used during compilation

## Troubleshooting

### Error "FREERTOS_HEAP must be 1, 2, 3, 4, or 5"
You specified an invalid value for FREERTOS_HEAP. Use only numbers from 1 to 5.

### Port compilation error
Verify that the specified port exists in the FreeRTOS repository and is compatible with your compilation target.

For example, using `FREERTOS_PORT="GCC/ARM_CM4F"` on an x86_64 Linux system will cause compilation errors because the ARM port is not compatible with the x86_64 architecture.

**Solutions:**
- For development/testing on Linux: use `ThirdParty/GCC/Posix` (default)
- For ARM targets: specify the appropriate Rust target along with the FreeRTOS port
- Verify compatibility between FreeRTOS port and Rust compilation target

### Link errors
Make sure the Rust compilation target is compatible with the selected FreeRTOS port.

### Missing configuration errors
If you see errors like `'configMAX_SYSCALL_INTERRUPT_PRIORITY' undeclared`, it means the selected port requires additional configurations in `FreeRTOSConfig.h` that might not be present in the current configuration file, which is optimized for the POSIX port.