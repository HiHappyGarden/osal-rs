# osal-rs-serde - Usage Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
osal-rs-serde = { version = "0.3", features = ["derive"] }
```

### Features

- `alloc`: Enables dynamic allocation support (default)
- `std`: Enables standard library support
- `derive`: Enables `#[derive(Serialize, Deserialize)]` macros (recommended)

## Basic Usage

### With Derive Macros

The simplest way is to use derive macros:

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 10, y: 20 };

// Serialize
let mut buffer = [0u8; 8];
let len = to_bytes(&point, &mut buffer).unwrap();

// Deserialize
let restored: Point = from_bytes(&buffer[..len]).unwrap();
```

### Manual Implementation

For full control, you can implement traits manually:

```rust
use osal_rs_serde::{Serialize, Deserialize, Serializer, Deserializer};

struct Point {
    x: i32,
    y: i32,
}

impl Serialize for Point {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> core::result::Result<(), S::Error> {
        serializer.serialize_i32(self.x)?;
        serializer.serialize_i32(self.y)?;
        Ok(())
    }
}

impl Deserialize for Point {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
        Ok(Point {
            x: deserializer.deserialize_i32()?,
            y: deserializer.deserialize_i32()?,
        })
    }
}
```

## Supported Types

### Primitive Types

All primitive types are automatically supported:

```rust
// Integers
let v1: u8 = 42;
let v2: i32 = -100;
let v3: u64 = 1234567890;

// Floats
let f1: f32 = 3.14;
let f2: f64 = 2.718281828;

// Boolean
let b: bool = true;
```

### Arrays

Fixed-size arrays are supported:

```rust
let array: [u8; 5] = [1, 2, 3, 4, 5];
let mut buffer = [0u8; 5];
to_bytes(&array, &mut buffer).unwrap();
```

### Tuples

Tuples up to 3 elements:

```rust
let tuple = (100u16, 200u16, 300u16);
let mut buffer = [0u8; 6];
to_bytes(&tuple, &mut buffer).unwrap();
```

### Option

The `Option<T>` type is supported:

```rust
let some: Option<u32> = Some(42);
let none: Option<u32> = None;

// Some serializes as: 1 byte (tag) + value
// None serializes as: 1 byte (tag)
```

### Custom Structures

With derive macros, any structure is serializable:

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    id: u32,
    enabled: bool,
    rate: u16,
}
```

### Nested Structures

Structures can contain other structures:

```rust
#[derive(Serialize, Deserialize)]
struct Inner {
    value: i32,
}

#[derive(Serialize, Deserialize)]
struct Outer {
    inner: Inner,
    flag: bool,
}
```

## Advanced Usage

### Custom Serializers

You can create custom serializers for specific formats:

```rust
use osal_rs_serde::{Serializer, Error};

struct MySerializer {
    // Your state
}

impl Serializer for MySerializer {
    type Error = Error;
    
    fn serialize_u32(&mut self, v: u32) -> core::result::Result<(), Error> {
        // Your implementation
        Ok(())
    }
    
    // Implement other methods...
}
```

### Error Handling

The framework uses an enumerated Error type:

```rust
use osal_rs_serde::{Error, to_bytes};

let value = 42u32;
let mut buffer = [0u8; 2]; // Too small!

match to_bytes(&value, &mut buffer) {
    Ok(len) => println!("Serialized {} bytes", len),
    Err(Error::BufferTooSmall) => println!("Buffer too small"),
    Err(e) => println!("Other error: {:?}", e),
}
```

## Integration with osal-rs

### Usage with Queue

```rust
use osal_rs::os::{Queue, QueueFn};
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize)]
struct Message {
    id: u32,
    data: [u8; 16],
}

// Create queue
let queue = Queue::new(10, 32).unwrap();

// Serialize and send
let msg = Message { id: 1, data: [0; 16] };
let mut buffer = [0u8; 32];
let len = to_bytes(&msg, &mut buffer).unwrap();
queue.post(&buffer[..len], 100).unwrap();

// Receive and deserialize
let mut recv_buffer = [0u8; 32];
queue.fetch(&mut recv_buffer, 100).unwrap();
let received: Message = from_bytes(&recv_buffer).unwrap();
```

## Best Practices

1. **Use derive macros when possible** - It's simpler and less error-prone
2. **Allocate sufficient buffers** - Calculate required size in advance
3. **Handle errors** - Don't use `.unwrap()` in production
4. **Serialize in little-endian** - This is the default, consider portability
5. **Test with real data** - Verify sizes and limits

## Performance

### Type Sizes

```
bool:       1 byte
u8/i8:      1 byte
u16/i16:    2 bytes
u32/i32:    4 bytes
u64/i64:    8 bytes
f32:        4 bytes
f64:        8 bytes
Option<T>:  1 byte (tag) + sizeof(T) if Some
Array[T;N]: sizeof(T) * N
```

### Zero-Copy Deserialization

For optimal performance in embedded, use stack-allocated buffers:

```rust
let mut buffer: [u8; 64] = [0; 64];  // Stack allocation
```

## Current Limitations

- **Enums**: Not yet supported (in development)
- **Vec/String**: Require `alloc` feature
- **Tuples**: Only supported up to 3 elements
- **Union**: Not supported

## Examples

See the `examples/` folder for complete examples:

```bash
# Basic example
cargo run --package osal-rs-serde --example basic

# With derive
cargo run --package osal-rs-serde --example with_derive --features derive

# Custom serializer
cargo run --package osal-rs-serde --example custom_serializer --features derive

# Advanced integration
cargo run --package osal-rs-serde --example integration --features derive
```

## Contributing

The project is open source. Contributions are welcome!

## License

GPL-3.0
