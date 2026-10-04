# OSAL-RS-Serde Derive Macros

Procedural macros for automatic implementation of serialization traits in `osal-rs-serde`.

[![Crates.io](https://img.shields.io/crates/v/osal-rs-serde-derive.svg)](https://crates.io/crates/osal-rs-serde-derive)
[![Documentation](https://docs.rs/osal-rs-serde-derive/badge.svg)](https://docs.rs/osal-rs-serde-derive)
[![License: LGPL-2.1](https://img.shields.io/badge/License-LGPL%202.1-blue.svg)](LICENSE)

## Overview

This crate provides `#[derive(Serialize, Deserialize)]` macros that automatically implement the `Serialize` and `Deserialize` traits for your custom types. These macros are the recommended way to use `osal-rs-serde`.

## Usage

Add the `derive` feature to enable these macros:

```toml
[dependencies]
osal-rs-serde = { version = "1.2", features = ["derive"] }
```

Then use the macros on your structs:

```rust
use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct SensorData {
    temperature: i16,
    humidity: u8,
    pressure: u32,
}

let data = SensorData { temperature: 25, humidity: 60, pressure: 1013 };

let mut buffer = [0u8; 16];
let len = to_bytes(&data, &mut buffer).unwrap();

let decoded: SensorData = from_bytes(&buffer[..len]).unwrap();
assert_eq!(decoded, data);
```

## Supported Struct Types

### Named Fields (Most Common)

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}
```

### Tuple Structs

Fields are named `"0"`, `"1"`, ... when passed to the serializer.

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Color(u8, u8, u8);
```

### Unit Structs

Unit structs serialize to zero bytes.

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Marker;
```

## Supported Field Types

The derive macros work with any type that implements `Serialize`/`Deserialize`:

### Primitives

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AllPrimitives {
    a: bool,
    b: u8,
    c: i16,
    d: u32,
    e: i64,
    f: f32,
    g: f64,
}
```

### Arrays

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WithArrays {
    samples: [u16; 8],
    matrix: [[f32; 3]; 3],
}
```

### Tuples

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WithTuples {
    coordinate: (i32, i32),
    rgb: (u8, u8, u8),
}
```

### Option Types

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WithOptionals {
    required_id: u32,
    optional_name: Option<u8>,
    optional_value: Option<i16>,
}
```

### Nested Structs

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Inner {
    value: i32,
}

#[derive(Serialize, Deserialize)]
struct Outer {
    id: u32,
    inner: Inner,
}
```

### Collections (always available)

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WithCollections {
    items: Vec<u32>,
    name: String,
}
```

## Complex Examples

### Embedded System Telemetry

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SensorReading {
    sensor_id: u8,
    value: i16,
    timestamp: u64,
}

#[derive(Serialize, Deserialize)]
struct TelemetryPacket {
    device_id: u32,
    readings: [SensorReading; 4],
    battery_level: u8,
    status_flags: u16,
}
```

### Robot Control

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MotorState {
    motor_id: u8,
    speed: i16,
    current: u16,
    enabled: bool,
}

#[derive(Serialize, Deserialize)]
struct RobotCommand {
    timestamp: u64,
    motors: [MotorState; 4],
    emergency_stop: bool,
}
```

### Configuration with Optional Fields

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct DeviceConfig {
    device_id: u32,
    baudrate: u32,
    timeout_ms: Option<u16>,
    retry_count: Option<u8>,
    enabled: bool,
}
```

## Serialization Order

Fields are serialized in the order they are declared in the struct, little-endian:

```rust
use osal_rs_serde::{Serialize, to_bytes};

#[derive(Serialize)]
struct Example {
    first: u8,    // Byte 0
    second: u16,  // Bytes 1-2
    third: u32,   // Bytes 3-6
}

let mut buffer = [0u8; 8];
let len = to_bytes(&Example { first: 0x01, second: 0x0302, third: 0x07060504 }, &mut buffer).unwrap();
assert_eq!(&buffer[..len], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
```

This produces the binary layout: `[first, second_lo, second_hi, third_0, third_1, third_2, third_3]`

## Limitations

### Enums

Currently, enums are not supported by the derive macro:

```rust,compile_fail
// ❌ NOT SUPPORTED YET
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
enum Status {
    Active,
    Inactive,
}
```

For enums, implement the traits manually or use an integer representation:

```rust
// ✅ WORKAROUND
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Status {
    code: u8,  // 0 = Inactive, 1 = Active
}
```

### Unions

Unions are not supported:

```rust,compile_fail
// ❌ NOT SUPPORTED
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
union Data {
    integer: i32,
    float: f32,
}
```

### Generic Types

Generic types are not yet supported in the current version:

```rust,compile_fail
// ❌ NOT SUPPORTED YET
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Container<T> {
    value: T,
}
```

## Generated Code

For this struct:

```rust
use osal_rs_serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}
```

the macros generate the equivalent of:

```rust
use osal_rs_serde::{Serialize, Deserialize, Serializer, Deserializer};

struct Point {
    x: i32,
    y: i32,
}

impl Serialize for Point {
    fn serialize<S: Serializer>(&self, name: &str, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_struct_start(name, 2)?;
        serializer.serialize_field("x", &self.x)?;
        serializer.serialize_field("y", &self.y)?;
        serializer.serialize_struct_end()?;
        Ok(())
    }
}

impl Deserialize for Point {
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> Result<Self, D::Error> {
        deserializer.deserialize_struct_start(name)?;
        let result = Self {
            x: deserializer.deserialize_field::<i32>("x")?,
            y: deserializer.deserialize_field::<i32>("y")?,
        };
        deserializer.deserialize_struct_end()?;
        Ok(result)
    }
}
```

Tuple structs follow the same pattern, with fields named `"0"`, `"1"`, ...; unit structs generate empty implementations.

`name` is whatever the caller passes: `""` for a top-level call (e.g. via `to_bytes`/`from_bytes`), or the field name when the struct is nested inside another via `serialize_field`/`deserialize_field`. `ByteSerializer`/`ByteDeserializer` ignore it entirely since the binary format has no field names; a JSON-style serializer would use it as the key.

## Debugging

To see the generated code, use `cargo expand`:

```bash
cargo install cargo-expand
cargo expand --example your_example
```

## Error Messages

Common errors and solutions:

### "the trait bound `CustomType: Serialize` is not satisfied"

Every field type must implement `Serialize`/`Deserialize`:

```rust,compile_fail
use osal_rs_serde::{Serialize, Deserialize};

struct CustomType(u8); // ❌ missing #[derive(Serialize, Deserialize)]

#[derive(Serialize, Deserialize)]
struct MyType {
    value: CustomType,
}
```

**Solution**: derive (or manually implement) the traits on the field type too.

### "Serialize derive macro does not support enums yet" / "... does not support unions"

The derive was applied to an enum or a union. See [Limitations](#limitations): only structs (named, tuple or unit) are supported.

### "missing generics for struct" / "implicit elided lifetime not allowed here"

The derive was applied to a struct with generic or lifetime parameters, which is not supported yet. See [Generic Types](#generic-types).

## Performance

The derive macros generate efficient code with:
- No runtime overhead compared to manual implementation
- Inlining opportunities for the compiler
- Zero-cost abstractions
- No allocations, except where `Vec`/`String` fields themselves allocate

## Best Practices

1. **Always use derive when possible** - It's less error-prone than manual implementation
2. **Keep structs simple** - Avoid deeply nested structures when possible
3. **Treat field order as part of the format** - Reordering fields changes the binary layout
4. **Document binary format** - Add comments about the serialized format
5. **Use Option for optional data** - Don't use magic values

## Examples

See the parent crate's `examples/` directory for complete working examples:
- `with_derive.rs` - Basic usage
- `arrays_tuples.rs` - Arrays and tuples
- `nested_structs.rs` - Nested structures
- `optional_fields.rs` - Optional fields
- `robot_control.rs` - Complex embedded example

## License

LGPL-2.1-or-later - See [LICENSE](../LICENSE) for details.

## Author

Antonio Salsi - [passy.linux@zresa.it](mailto:passy.linux@zresa.it)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Links

- [Repository](https://github.com/HiHappyGarden/osal-rs)
- [Documentation](https://docs.rs/osal-rs-serde-derive)
- [Crates.io](https://crates.io/crates/osal-rs-serde-derive)
