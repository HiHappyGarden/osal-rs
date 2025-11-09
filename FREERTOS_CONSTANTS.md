# Reading FreeRTOS Configuration Constants from Rust

## Overview

OSAL-RS allows reading configuration constants defined in `FreeRTOSConfig.h` directly from Rust code through the `constants` module.

## Available Constants

### CONFIG_TICK_RATE_HZ
System tick frequency in Hz (ticks per second).

```rust
use osal_rs::constants::CONFIG_TICK_RATE_HZ;

fn main() {
    println!("Tick rate: {} Hz", CONFIG_TICK_RATE_HZ);
    // Output: Tick rate: 1000 Hz
}
```

### Other Constants

- **CONFIG_MAX_PRIORITIES**: Maximum number of priorities (8)
- **CONFIG_MINIMAL_STACK_SIZE**: Minimum stack size in words (128)
- **CONFIG_TOTAL_HEAP_SIZE**: Total heap size in bytes (15 KB)
- **CONFIG_MAX_TASK_NAME_LEN**: Maximum length of task name (16)
- **CONFIG_CPU_CLOCK_HZ**: CPU clock frequency in Hz (1 GHz)

## Helper Functions

### get_tick_period_ms()
Calculates tick period in milliseconds:

```rust
use osal_rs::constants::get_tick_period_ms;

let period = get_tick_period_ms();
println!("Tick period: {} ms", period);
// Output: Tick period: 1 ms
```

### ms_to_ticks(milliseconds)
Converts milliseconds to ticks:

```rust
use osal_rs::constants::ms_to_ticks;

let delay_ticks = ms_to_ticks(500); // 500ms
println!("500ms = {} ticks", delay_ticks);
// Output: 500ms = 500 ticks
```

### ticks_to_ms(ticks)
Converts ticks to milliseconds:

```rust
use osal_rs::constants::ticks_to_ms;

let delay_ms = ticks_to_ms(1000); // 1000 ticks
println!("1000 ticks = {} ms", delay_ms);
// Output: 1000 ticks = 1000 ms
```

## Complete Example

```rust
use osal_rs::constants::*;

fn main() {
    // Read CONFIG_TICK_RATE_HZ
    println!("Tick Rate: {} Hz", CONFIG_TICK_RATE_HZ);
    
    // Calculate period
    let period = get_tick_period_ms();
    println!("Tick period: {} ms", period);
    
    // Convert time for delay
    let delay_100ms = ms_to_ticks(100);
    println!("To wait 100ms requires {} ticks", delay_100ms);
    
    // Check other constants
    println!("Maximum priorities: {}", CONFIG_MAX_PRIORITIES);
    println!("Minimum stack: {} words", CONFIG_MINIMAL_STACK_SIZE);
}
```

## Usage in Threads

```rust
use osal_rs::constants::ms_to_ticks;
use osal_rs::{Thread, ThreadDefaultPriority};
use alloc::sync::Arc;

fn main() {
    let thread = Thread::new(
        |_| {
            // Wait 100ms using conversion
            let delay = ms_to_ticks(100);
            // Use delay with vTaskDelay...
            Arc::new(())
        },
        "my_thread",
        1024,
        None,
        ThreadDefaultPriority::Normal,
    ).unwrap();
}
```

## Executable Example

See `examples/freertos_config.rs` for a complete example:

```bash
cargo build --example freertos_config --features freertos
```

## Important Notes

⚠️ **Manual Synchronization**: Constants in the `constants.rs` module must be updated manually if you modify `FreeRTOSConfig.h`.

### Constants Verification

Run tests to verify that constants are synchronized:

```bash
cargo test --lib --features freertos constants
```

### Updating Constants

If you modify `FreeRTOSConfig.h`, remember to also update `src/freertos/constants.rs`:

1. Modify `include/FreeRTOSConfig.h`:
   ```c
   #define configTICK_RATE_HZ  ( ( TickType_t ) 2000 )  // Changed to 2000
   ```

2. Update `src/freertos/constants.rs`:
   ```rust
   pub const CONFIG_TICK_RATE_HZ: TickType_t = 2000;  // Updated
   ```

## Alternative Approach: Automatic Bindgen

For automatic synchronization, you can use the `bindgen` feature which automatically generates bindings from C constants:

```bash
cargo build --features "freertos,bindgen"
```

This requires `libclang` to be installed on the system.

## References

- File: `src/freertos/constants.rs`
- Configuration: `include/FreeRTOSConfig.h`
- Example: `examples/freertos_config.rs`
- FreeRTOS Documentation: https://www.freertos.org/a00110.html

