# OSAL-RS-Serde

An extensible serialization/deserialization framework for Rust, inspired by Serde but optimized for embedded systems and no-std environments.

## Features

- ✅ **No-std compatible**: Works perfectly in bare-metal environments
- ✅ **Memory-efficient**: Optimized for resource-constrained systems
- ✅ **Extensible**: Easy to create custom serializers for any format
- ✅ **Derive Macro**: Support for `#[derive(Serialize, Deserialize)]`
- ✅ **Type-safe**: Leverages Rust's type system
- ✅ **Reusable**: Can be used in any project, not just with osal-rs

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
osal-rs-serde = { version = "0.3", features = ["derive"] }
```

Available features:
- `alloc`: Enables dynamic allocation support (included in `default`)
- `std`: Enables standard library support
- `derive`: Enables `#[derive(Serialize, Deserialize)]` macros
## Project Structure

The `osal-rs-serde` crate includes:
- **Core library**: Traits and implementations for serialization/deserialization
- **Derive macros** (optional): Procedural macros for automatic derivation (in `derive/`)

Everything is contained in a single package for ease of use.
## Usage

### With Derive Macros (Recommended)

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

```rust
use osal_rs::os::{Queue, QueueFn};
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct Message {
    id: u32,
    value: i16,
}

fn main() {
    let queue = Queue::new(10, 32).unwrap();
    
    // Send message
    let msg = Message { id: 42, value: 100 };
    let mut buffer = [0u8; 32];
    let len = to_bytes(&msg, &mut buffer).unwrap();
    queue.post(&buffer[..len], 100).unwrap();
    
    // Receive message
    let mut recv_buffer = [0u8; 32];
    queue.fetch(&mut recv_buffer, 100).unwrap();
    let received: Message = from_bytes(&recv_buffer).unwrap();
}
```

## Supported Types

The framework automatically supports:

- Primitive types: `bool`, `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`
- Floating point: `f32`, `f64`
- Arrays: `[T; N]`
- Tuples: `(T1, T2)`, `(T1, T2, T3)`
- Option: `Option<T>`
- Any custom type that implements `Serialize`/`Deserialize`

## Custom Serializers

You can create custom serializers to support different formats:

```rust
use osal_rs_serde::{Serializer, Error};

struct JsonSerializer {
    // ... fields to handle JSON
}

impl Serializer for JsonSerializer {
    type Error = Error;
    
    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        // JSON implementation
        // ...
        Ok(())
    }
    
    // Implement other methods...
}
```

## Examples

See the `examples/` folder for complete examples:

```bash
# Basic example
cargo run --example basic

# With derive macros
cargo run --example with_derive --features derive

# Integration with OSAL-RS
cargo run --example with_queue
```

## Comparison with Serde

| Feature | osal-rs-serde | serde |
|---------|---------------|-------|
| No-std | ✅ | ✅ |
| Derive macro | ✅ | ✅ |
| Binary size | Small | Medium/Large |
| Supported formats | Customizable | Many built-in |
| Target | Embedded/RTOS | General purpose |

## License

GPL-3.0 - See [LICENSE](LICENSE) for details.

## Author

Antonio Salsi <passy.linux@zresa.it>
