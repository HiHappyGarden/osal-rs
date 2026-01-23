# OSAL-RS-Serde

An extensible, lightweight serialization/deserialization framework for Rust, inspired by Serde but optimized for embedded systems and no-std environments.

## Features

- ✅ **No-std compatible**: Works perfectly in bare-metal embedded environments
- ✅ **Memory-efficient**: Optimized for resource-constrained systems with predictable memory usage
- ✅ **Extensible**: Easy to create custom serializers for any format (binary, JSON, MessagePack, etc.)
- ✅ **Derive Macro**: Full support for `#[derive(Serialize, Deserialize)]`
- ✅ **Type-safe**: Leverages Rust's type system for compile-time guarantees
- ✅ **Zero-copy**: Direct buffer reads/writes without intermediate allocations
- ✅ **Reusable**: Can be used in any project, not just with osal-rs


### Supported Types

#### Primitives
- **Integers**: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`
- **Floats**: `f32`, `f64`
- **Boolean**: `bool`

#### Compound Types
- **Arrays**: `[T; N]` for any serializable type T
- **Tuples**: `(T1, T2)`, `(T1, T2, T3)` up to 3 elements
- **Option**: `Option<T>` for optional fields

#### Collections (with `alloc` feature)
- **Vec**: `Vec<T>` for dynamic arrays
- **String**: `String` and `&str`

#### Custom Types
- Any struct with `#[derive(Serialize, Deserialize)]`
- Nested struct composition fully supported

### Binary Format & Memory Sizes

The default `ByteSerializer` uses little-endian binary format with no padding:

```
bool:       1 byte (0 or 1)
u8/i8:      1 byte
u16/i16:    2 bytes
u32/i32:    4 bytes
u64/i64:    8 bytes
u128/i128:  16 bytes
f32:        4 bytes (IEEE 754)
f64:        8 bytes (IEEE 754)
Option<T>:  1 byte tag + sizeof(T) if Some, 1 byte if None
Array[T;N]: sizeof(T) * N (no length prefix)
Tuple:      sum(sizeof each field)
Vec<T>:     4 bytes (u32 length) + sizeof(T) * length
String:     4 bytes (u32 length) + UTF-8 bytes
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
osal-rs-serde = { version = "0.3", features = ["derive"] }
```

Available features:
- `default`: Includes `alloc` feature for Vec and String support
- `alloc`: Enables dynamic allocation support (Vec, String)
- `std`: Enables standard library support (error traits, etc.)
- `derive`: Enables `#[derive(Serialize, Deserialize)]` macros (**recommended**)

For no-std environments without allocation:
```toml
[dependencies]
osal-rs-serde = { version = "0.3", default-features = false, features = ["derive"] }
```

## Project Structure

The `osal-rs-serde` workspace includes:
- **osal-rs-serde**: Core library with traits and implementations
- **osal-rs-serde/derive**: Procedural macros for automatic derivation (optional, enabled via `derive` feature)

Everything is contained in a single package for ease of use.
## Usage

### With Derive Macros (Recommended)

#### Basic Struct Example

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct SensorData {
    temperature: i16,
    humidity: u8,
    pressure: u32,
}

fn main() {
    // Create a structure
    let data = SensorData {
        temperature: 25,
        humidity: 60,
        pressure: 1013,
    };

    // Serialize
    let mut buffer = [0u8; 32];
    let len = to_bytes(&data, &mut buffer).unwrap();
    println!("Serialized {} bytes", len);

    // Deserialize
    let read_data: SensorData = from_bytes(&buffer[..len]).unwrap();
    println!("Temperature: {}", read_data.temperature);
}
```

#### Struct with Optional Fields

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct Config {
    device_id: u32,
    name: Option<u8>,  // Optional device name code
    enabled: bool,
    timeout: Option<u16>,  // Optional timeout in ms
}

fn main() {
    let config = Config {
        device_id: 100,
        name: Some(42),
        enabled: true,
        timeout: None,
    };

    let mut buffer = [0u8; 64];
    let len = to_bytes(&config, &mut buffer).unwrap();
    let decoded: Config = from_bytes(&buffer[..len]).unwrap();
}
```

#### Struct with Arrays and Tuples

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct TelemetryPacket {
    timestamp: u64,
    coordinates: (i32, i32, i32),  // x, y, z
    samples: [u16; 8],              // 8 sensor readings
    status: u8,
}

fn main() {
    let packet = TelemetryPacket {
        timestamp: 1642857600,
        coordinates: (100, 200, 50),
        samples: [10, 20, 30, 40, 50, 60, 70, 80],
        status: 0xFF,
    };

    let mut buffer = [0u8; 128];
    let len = to_bytes(&packet, &mut buffer).unwrap();
    println!("Telemetry packet: {} bytes", len);
}
```

#### Nested Structs

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct Location {
    latitude: i32,
    longitude: i32,
}

#[derive(Serialize, Deserialize)]
struct Device {
    id: u32,
    battery: u8,
    location: Location,
    active: bool,
}

fn main() {
    let device = Device {
        id: 42,
        battery: 85,
        location: Location {
            latitude: 45500000,
            longitude: 9200000,
        },
        active: true,
    };

    let mut buffer = [0u8; 64];
    let len = to_bytes(&device, &mut buffer).unwrap();
    let decoded: Device = from_bytes(&buffer[..len]).unwrap();
    println!("Device at {}, {}", 
             decoded.location.latitude, 
             decoded.location.longitude);
}
```

#### Complex Embedded System Example

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct MotorControl {
    motor_id: u8,
    speed: i16,        // -1000 to 1000
    direction: bool,   // true = forward, false = reverse
    current: u16,      // mA
}

#[derive(Serialize, Deserialize)]
struct RobotState {
    timestamp: u64,
    motors: [MotorControl; 4],  // 4 motors
    battery_voltage: u16,        // mV
    temperature: i8,             // °C
    error_flags: u32,
}

fn main() {
    let state = RobotState {
        timestamp: 1000000,
        motors: [
            MotorControl { motor_id: 0, speed: 500, direction: true, current: 1200 },
            MotorControl { motor_id: 1, speed: 500, direction: true, current: 1150 },
            MotorControl { motor_id: 2, speed: -300, direction: false, current: 800 },
            MotorControl { motor_id: 3, speed: -300, direction: false, current: 850 },
        ],
        battery_voltage: 12400,  // 12.4V
        temperature: 35,
        error_flags: 0,
    };

    let mut buffer = [0u8; 256];
    let len = to_bytes(&state, &mut buffer).unwrap();
    println!("Robot state serialized: {} bytes", len);
    
    // Deserialize and check
    let decoded: RobotState = from_bytes(&buffer[..len]).unwrap();
    println!("Battery: {}mV, Temp: {}°C", 
             decoded.battery_voltage, 
             decoded.temperature);
}
```

### Manual Implementation

```rust
use osal_rs_serde::{Serialize, Deserialize, Serializer, Deserializer};

struct Point {
    x: i32,
    y: i32,
}

impl Serialize for Point {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_i32(self.x)?;
        serializer.serialize_i32(self.y)?;
        Ok(())
    }
}

impl Deserialize for Point {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        Ok(Point {
            x: deserializer.deserialize_i32()?,
            y: deserializer.deserialize_i32()?,
        })
    }
}
```

### Usage with OSAL-RS Queue

Perfect for inter-task communication in RTOS environments:

```rust
use osal_rs::os::{Queue, QueueFn};
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct Command {
    id: u32,
    action: u8,
    params: [u16; 4],
}

fn sender_task(queue: &Queue) {
    let cmd = Command { 
        id: 42, 
        action: 0x10,
        params: [100, 200, 300, 400],
    };
    
    let mut buffer = [0u8; 32];
    let len = to_bytes(&cmd, &mut buffer).unwrap();
    queue.post(&buffer[..len], 100).unwrap();
}

fn receiver_task(queue: &Queue) {
    let mut buffer = [0u8; 32];
    queue.fetch(&mut buffer, 100).unwrap();
    let cmd: Command = from_bytes(&buffer).unwrap();
    println!("Received command: id={}, action=0x{:02X}", cmd.id, cmd.action);
}
```

## Supported Types

The framework automatically supports serialization/deserialization for:

- **Primitives**: `bool`, `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `f32`, `f64`
- **Compound**: Arrays `[T; N]`, tuples `(T1, T2)` and `(T1, T2, T3)`, `Option<T>`
- **Collections** (with `alloc`): `Vec<T>`, `String`, `&str`
- **Custom**: Any struct implementing `Serialize`/`Deserialize` (or using derive)
- **Nested**: Full support for nested struct composition

## Custom Serializers

You can create custom serializers to support different formats (JSON, MessagePack, CBOR, etc.):

```rust
use osal_rs_serde::{Serializer, Deserializer, Error};

struct JsonSerializer<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> Serializer for JsonSerializer<'a> {
    type Error = Error;
    
    fn serialize_u32(&mut self, name: &str, v: u32) -> Result<(), Self::Error> {
        // Write JSON format: "name": value
        // Your implementation here...
        Ok(())
    }
    
    fn serialize_bool(&mut self, name: &str, v: bool) -> Result<(), Self::Error> {
        // Write JSON format: "name": true/false
        // Your implementation here...
        Ok(())
    }
    
    // Implement other serialize_* methods...
}

// Similarly implement Deserializer trait for deserialization
```

See `examples/custom_serializer.rs` for a complete text-based serializer implementation.

## Performance Considerations

### Buffer Size Calculation

For fixed-size types, calculate buffer size at compile time:

```rust
#[derive(Serialize, Deserialize)]
struct Packet {
    id: u32,        // 4 bytes
    value: i16,     // 2 bytes
    flags: u8,      // 1 byte
}
// Total: 7 bytes

const BUFFER_SIZE: usize = 7;
let mut buffer = [0u8; BUFFER_SIZE];
```

### Zero-Copy Operation

The serializer writes directly to your buffer with no intermediate allocations:

```rust
// Stack-allocated buffer - no heap allocation
let mut buffer = [0u8; 64];
let len = to_bytes(&data, &mut buffer)?;

// Use only the filled portion
send_to_uart(&buffer[..len]);
```

### Compile-Time Guarantees

The type system ensures correctness:
- Cannot deserialize wrong type from buffer
- Compile-time checks for trait implementations
- No runtime type checks needed

## Examples

The `examples/` directory contains complete working examples demonstrating various features:

### Running Examples

```bash
# Basic struct serialization
cargo run --example basic --features derive

# Using derive macros (recommended approach)
cargo run --example with_derive --features derive

# Arrays and tuples
cargo run --example arrays_tuples --features derive

# Nested struct composition
cargo run --example nested_structs --features derive

# Optional fields with Option<T>
cargo run --example optional_fields --features derive

# Complex embedded system (robot control)
cargo run --example robot_control --features derive

# Custom serializer implementation
cargo run --example custom_serializer

# Integration with OSAL-RS
cargo run --example integration --features derive
```

### Example Descriptions

- **`basic.rs`**: Simple manual implementation without derive macros
- **`with_derive.rs`**: Same example using `#[derive]` macros
- **`arrays_tuples.rs`**: Working with arrays and tuples in structs
- **`nested_structs.rs`**: Nested struct composition patterns
- **`optional_fields.rs`**: Using `Option<T>` for optional data
- **`robot_control.rs`**: Complex real-world embedded system example with motor control
- **`custom_serializer.rs`**: Creating a custom text-based serializer
- **`integration.rs`**: Integration with OSAL-RS queues for inter-task communication

## Best Practices

### 1. Use Derive Macros

Always prefer derive macros for standard serialization:

```rust
#[derive(Serialize, Deserialize)]
struct MyStruct {
    // fields...
}
```

### 2. Calculate Buffer Sizes

Pre-calculate buffer sizes for better performance:

```rust
const fn calculate_size() -> usize {
    size_of::<u32>() + size_of::<i16>() + size_of::<bool>()
}

let mut buffer = [0u8; calculate_size()];
```

### 3. Error Handling

Always handle serialization errors appropriately:

```rust
match to_bytes(&data, &mut buffer) {
    Ok(len) => send_data(&buffer[..len]),
    Err(Error::BufferTooSmall) => {
        // Handle buffer overflow
    }
    Err(e) => {
        // Handle other errors
    }
}
```

### 4. Versioning

Consider adding version fields for forward compatibility:

```rust
#[derive(Serialize, Deserialize)]
struct Message {
    version: u8,
    // other fields...
}
```

## Comparison with Serde

| Feature | osal-rs-serde | serde |
|---------|---------------|-------|
| No-std support | ✅ Native | ✅ Via feature |
| Derive macros | ✅ Built-in | ✅ Separate crate |
| Binary size | **Very small** | Medium/Large |
| Supported formats | Custom (extendable) | Many built-in |
| Target use case | **Embedded/RTOS** | General purpose |
| Zero-copy | ✅ Always | Depends on format |
| Compile time | **Fast** | Slower |
| Learning curve | **Gentle** | Moderate |

**Choose osal-rs-serde when:**
- Working in embedded/no-std environments
- Need predictable memory usage
- Want minimal binary size
- Require simple, fast compilation
- Building RTOS applications

**Choose serde when:**
- Need many pre-built format implementations
- Working primarily with std
- Require advanced features (flatten, rename, etc.)
- Ecosystem integration is important

## License

GPL-3.0 - See [LICENSE](LICENSE) for details.

## Author

Antonio Salsi <passy.linux@zresa.it>
