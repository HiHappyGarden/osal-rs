# 🔄 Automatic FreeRTOS Constants Synchronization

## ✨ Features

**Constants are now synchronized AUTOMATICALLY!**

When you modify `include/FreeRTOSConfig.h`, the Rust values are automatically updated during compilation.

## 🎯 How It Works

### Before (Manual) ❌
```
1. You modify include/FreeRTOSConfig.h
2. You must remember to also modify src/freertos/constants.rs
3. If you forget, values become misaligned
4. Hard-to-find bugs
```

### Now (Automatic) ✅
```
1. You modify include/FreeRTOSConfig.h
2. You compile: cargo build
3. ✓ Rust values are automatically synchronized!
```

## 📝 Practical Example

### 1. Modify FreeRTOSConfig.h

```c
// include/FreeRTOSConfig.h
#define configTICK_RATE_HZ  ( ( TickType_t ) 2000 )  // ← Changed from 1000 to 2000
```

### 2. Recompile

```bash
cargo build --features freertos
```

Output:
```
warning: ✓ FreeRTOS constants auto-synchronized from FreeRTOSConfig.h
warning:   configTICK_RATE_HZ = 2000  ← Automatically updated!
```

### 3. Rust code automatically uses the new value

```rust
use osal_rs::constants::CONFIG_TICK_RATE_HZ;

fn main() {
    println!("Tick rate: {} Hz", CONFIG_TICK_RATE_HZ);
    // Output: Tick rate: 2000 Hz  ← Automatically updated!
}
```

## 🔍 Synchronization Testing

You can verify that values are synchronized:

```bash
# Compile
cargo build --features freertos

# Look at warnings to see extracted values
# warning:   configTICK_RATE_HZ = <value>
# warning:   configMAX_PRIORITIES = <value>

# Run tests
cargo test --lib --features freertos constants
```

## 📊 Automatically Synchronized Values

| C Constant | Rust Constant | Synchronization |
|------------|---------------|-----------------|
| `configTICK_RATE_HZ` | `CONFIG_TICK_RATE_HZ` | ✅ Automatic |
| `configMAX_PRIORITIES` | `CONFIG_MAX_PRIORITIES` | ✅ Automatic |
| `configMINIMAL_STACK_SIZE` | `CONFIG_MINIMAL_STACK_SIZE` | ✅ Automatic |
| `configTOTAL_HEAP_SIZE` | `CONFIG_TOTAL_HEAP_SIZE` | ✅ Automatic |
| `configMAX_TASK_NAME_LEN` | `CONFIG_MAX_TASK_NAME_LEN` | ✅ Automatic |
| `configCPU_CLOCK_HZ` | `CONFIG_CPU_CLOCK_HZ` | ✅ Automatic |

## 🛠️ How It Works Internally

1. **build.rs** reads `include/FreeRTOSConfig.h` during compilation
2. Extracts `#define` values using regex
3. Automatically generates `freertos_constants_generated.rs` in `OUT_DIR`
4. `src/freertos/constants.rs` includes the generated file with `include!(...)`
5. Cargo automatically rebuilds if `FreeRTOSConfig.h` changes

```
FreeRTOSConfig.h  →  build.rs  →  constants_generated.rs  →  constants.rs
     (C)              (parsing)        (generated)            (include!)
```

## ⚙️ Involved Files

| File | Role |
|------|------|
| `include/FreeRTOSConfig.h` | Source of constants (C) |
| `build.rs` | Script that extracts and generates |
| `$OUT_DIR/freertos_constants_generated.rs` | Automatically generated file |
| `src/freertos/constants.rs` | Includes the generated file |

## 🎓 Complete Example

### FreeRTOSConfig.h
```c
#define configTICK_RATE_HZ  ( ( TickType_t ) 2000 )
#define configMAX_PRIORITIES  ( 16 )
```

### After `cargo build --features freertos`

```rust
use osal_rs::constants::*;

fn main() {
    println!("TICK_RATE_HZ: {}", CONFIG_TICK_RATE_HZ);  // 2000
    println!("MAX_PRIORITIES: {}", CONFIG_MAX_PRIORITIES);  // 16
    
    // Helper functions use the updated values
    let ticks = ms_to_ticks(1000);
    println!("1000ms = {} ticks", ticks);  // 2000 (with 2000 Hz)
}
```

## ✅ Advantages

1. **Zero Maintenance**: You don't need to remember to synchronize manually
2. **Always Aligned**: C and Rust values are always synchronized
3. **Type-Safe**: Values are converted to correct Rust types
4. **Automatic Rebuild**: Cargo recompiles if FreeRTOSConfig.h changes
5. **Visibility**: Warnings show extracted values during compilation

## ⚠️ Notes

- Synchronization happens **during compilation** (`cargo build`)
- Values are extracted from `#define` in FreeRTOSConfig.h
- If you modify FreeRTOSConfig.h, simply run `cargo build` to update
- The generated file is in `target/debug/build/osal-rs-*/out/freertos_constants_generated.rs`

## 🧪 Verification Tests

```bash
# 1. Modify FreeRTOSConfig.h (e.g.: change configTICK_RATE_HZ)
# 2. Recompile
cargo build --features freertos

# 3. Check warnings to confirm
#    warning:   configTICK_RATE_HZ = <new value>

# 4. Run tests (some might fail if values change)
cargo test --lib --features freertos constants
```

## 🎉 Conclusion

**You no longer need to worry about manual synchronization!**

Modify `FreeRTOSConfig.h` and compile. Rust values will be automatically updated! ✨

## 📚 References

- **build.rs**: Build script that extracts values
- **src/freertos/constants.rs**: Includes the generated file
- **QUICK_START_CONFIG.md**: Quick guide to using constants
- **FREERTOS_CONSTANTS.md**: Complete documentation

