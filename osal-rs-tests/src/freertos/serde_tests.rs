/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! On-target checks for `osal-rs-serde` derive output.
//!
//! The host suite in `osal-rs-serde/tests` covers the same cases; running them
//! here catches target-specific differences (no_std + alloc, `MaybeUninit`
//! array decoding, wide integers) that the host build cannot see.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use osal_rs::utils::Result;
use osal_rs::{log_debug, log_info};
use osal_rs_serde::{from_bytes, to_bytes, ByteDeserializer, ByteSerializer, Deserialize, Serialize};

const TAG: &str = "SerdeTests";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Coordinates(i16, u8, u32);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Celsius(i16);

pub fn test_derive_tuple_struct() -> Result<()> {
    log_info!(TAG, "Starting test_derive_tuple_struct");
    let coords = Coordinates(-100, 42, 0xDEADBEEF);

    let mut buffer = [0u8; 32];
    let len = to_bytes(&coords, &mut buffer).expect("serialize Coordinates");

    // i16 (2) + u8 (1) + u32 (4), little-endian
    assert_eq!(len, 7);
    assert_eq!(&buffer[..2], &(-100i16).to_le_bytes());
    assert_eq!(buffer[2], 42);
    assert_eq!(&buffer[3..7], &0xDEADBEEFu32.to_le_bytes());

    let decoded: Coordinates = from_bytes(&buffer[..len]).expect("deserialize Coordinates");
    log_debug!(TAG, "Decoded {:?}", decoded);
    assert_eq!(decoded, coords);
    log_info!(TAG, "test_derive_tuple_struct PASSED");
    Ok(())
}

pub fn test_derive_newtype_struct() -> Result<()> {
    log_info!(TAG, "Starting test_derive_newtype_struct");
    let temp = Celsius(-273);

    let mut buffer = [0u8; 8];
    let len = to_bytes(&temp, &mut buffer).expect("serialize Celsius");
    assert_eq!(len, 2);

    let decoded: Celsius = from_bytes(&buffer[..len]).expect("deserialize Celsius");
    assert_eq!(decoded, temp);
    log_info!(TAG, "test_derive_newtype_struct PASSED");
    Ok(())
}

pub fn test_derive_tuple_struct_nested() -> Result<()> {
    log_info!(TAG, "Starting test_derive_tuple_struct_nested");

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Reading {
        id: u8,
        temp: Celsius,
        position: Coordinates,
        history: [Celsius; 2],
    }

    let reading = Reading {
        id: 7,
        temp: Celsius(215),
        position: Coordinates(1, 2, 3),
        history: [Celsius(-5), Celsius(30)],
    };

    let mut buffer = [0u8; 32];
    let len = to_bytes(&reading, &mut buffer).expect("serialize Reading");
    // id (1) + temp (2) + position (7) + history (2 * 2)
    assert_eq!(len, 14);

    let decoded: Reading = from_bytes(&buffer[..len]).expect("deserialize Reading");
    assert_eq!(decoded, reading);
    log_info!(TAG, "test_derive_tuple_struct_nested PASSED");
    Ok(())
}

pub fn test_derive_collections() -> Result<()> {
    log_info!(TAG, "Starting test_derive_collections");

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct WithCollections {
        items: Vec<u32>,
        name: String,
    }

    let value = WithCollections { items: vec![1, 2, 3], name: "osal".to_string() };

    let mut buffer = [0u8; 64];
    let len = to_bytes(&value, &mut buffer).expect("serialize WithCollections");
    // items: u32 length (4) + 3 * u32 (12); name: u32 length (4) + 4 UTF-8 bytes
    assert_eq!(len, 24);
    assert_eq!(&buffer[..4], &3u32.to_le_bytes());
    assert_eq!(&buffer[16..20], &4u32.to_le_bytes());
    assert_eq!(&buffer[20..24], b"osal");

    let decoded: WithCollections = from_bytes(&buffer[..len]).expect("deserialize WithCollections");
    assert_eq!(decoded, value);
    log_info!(TAG, "test_derive_collections PASSED");
    Ok(())
}

/// Wraps `ByteSerializer`/`ByteDeserializer` and records the struct protocol calls,
/// so that the event sequences produced by `Serialize` and `Deserialize` can be compared.
mod tracing {
    use alloc::format;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    use osal_rs_serde::{ByteDeserializer, ByteSerializer, Deserialize, Deserializer, Error, Serialize, Serializer};

    macro_rules! delegate_ser {
        ($($method:ident: $ty:ty),*) => {
            $(fn $method(&mut self, name: &str, v: $ty) -> Result<(), Error> { self.inner.$method(name, v) })*
        };
    }

    macro_rules! delegate_de {
        ($($method:ident: $ty:ty),*) => {
            $(fn $method(&mut self, name: &str) -> Result<$ty, Error> { self.inner.$method(name) })*
        };
    }

    pub struct TracingSerializer<'a> {
        pub inner: ByteSerializer<'a>,
        pub events: Vec<String>,
    }

    impl Serializer for TracingSerializer<'_> {
        type Error = Error;

        delegate_ser!(
            serialize_bool: bool, serialize_u8: u8, serialize_i8: i8,
            serialize_u16: u16, serialize_i16: i16, serialize_u32: u32, serialize_i32: i32,
            serialize_u64: u64, serialize_i64: i64, serialize_u128: u128, serialize_i128: i128,
            serialize_f32: f32, serialize_f64: f64, serialize_bytes: &[u8],
            serialize_string: &String, serialize_str: &str
        );

        fn serialize_vec<T: Serialize>(&mut self, name: &str, v: &Vec<T>) -> Result<(), Error> {
            self.serialize_u32(name, v.len() as u32)?;
            v.iter().try_for_each(|item| item.serialize(name, self))
        }

        fn serialize_array<T: Serialize>(&mut self, name: &str, v: &[T]) -> Result<(), Error> {
            v.iter().try_for_each(|item| item.serialize(name, self))
        }

        fn serialize_struct_start(&mut self, name: &str, _len: usize) -> Result<(), Error> {
            self.events.push(format!("start:{name}"));
            Ok(())
        }

        fn serialize_field<T: Serialize>(&mut self, name: &str, value: &T) -> Result<(), Error> {
            self.events.push(format!("field:{name}"));
            value.serialize(name, self)
        }

        fn serialize_struct_end(&mut self) -> Result<(), Error> {
            self.events.push("end".to_string());
            Ok(())
        }
    }

    pub struct TracingDeserializer<'a> {
        pub inner: ByteDeserializer<'a>,
        pub events: Vec<String>,
    }

    impl Deserializer for TracingDeserializer<'_> {
        type Error = Error;

        delegate_de!(
            deserialize_bool: bool, deserialize_u8: u8, deserialize_i8: i8,
            deserialize_u16: u16, deserialize_i16: i16, deserialize_u32: u32, deserialize_i32: i32,
            deserialize_u64: u64, deserialize_i64: i64, deserialize_u128: u128, deserialize_i128: i128,
            deserialize_f32: f32, deserialize_f64: f64, deserialize_string: String
        );

        fn deserialize_bytes(&mut self, name: &str, buffer: &mut [u8]) -> Result<usize, Error> {
            self.inner.deserialize_bytes(name, buffer)
        }

        fn deserialize_vec<T: Deserialize>(&mut self, name: &str) -> Result<Vec<T>, Error> {
            let len = self.deserialize_u32(name)? as usize;
            (0..len).map(|_| T::deserialize(self, name)).collect()
        }

        fn deserialize_array<T: Deserialize, const N: usize>(&mut self, name: &str) -> Result<[T; N], Error> {
            let items = (0..N).map(|_| T::deserialize(self, name)).collect::<Result<Vec<T>, Error>>()?;
            items.try_into().map_err(|_| Error::InvalidData)
        }

        fn deserialize_struct_start(&mut self, name: &str) -> Result<(), Error> {
            self.events.push(format!("start:{name}"));
            Ok(())
        }

        fn deserialize_field<T: Deserialize>(&mut self, name: &str) -> Result<T, Error> {
            self.events.push(format!("field:{name}"));
            T::deserialize(self, name)
        }

        fn deserialize_struct_end(&mut self) -> Result<(), Error> {
            self.events.push("end".to_string());
            Ok(())
        }
    }
}

pub fn test_derive_tuple_struct_protocol_symmetry() -> Result<()> {
    use tracing::{TracingDeserializer, TracingSerializer};

    log_info!(TAG, "Starting test_derive_tuple_struct_protocol_symmetry");
    let coords = Coordinates(-100, 42, 0xDEADBEEF);

    let mut buffer = [0u8; 32];
    let mut ser = TracingSerializer { inner: ByteSerializer::new(&mut buffer), events: Vec::new() };
    coords.serialize("coords", &mut ser).expect("serialize Coordinates");
    let len = ser.inner.position();
    let ser_events = ser.events;

    assert_eq!(ser_events, ["start:coords", "field:0", "field:1", "field:2", "end"]);

    let mut de = TracingDeserializer { inner: ByteDeserializer::new(&buffer[..len]), events: Vec::new() };
    let decoded = Coordinates::deserialize(&mut de, "coords").expect("deserialize Coordinates");
    log_debug!(TAG, "Deserializer events: {:?}", de.events);

    assert_eq!(decoded, coords);
    assert_eq!(de.events, ser_events);
    log_info!(TAG, "test_derive_tuple_struct_protocol_symmetry PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Serde Tests ==========");
    test_derive_tuple_struct()?;
    test_derive_newtype_struct()?;
    test_derive_tuple_struct_nested()?;
    test_derive_collections()?;
    test_derive_tuple_struct_protocol_symmetry()?;
    log_info!(TAG, "========== All Serde Tests PASSED ==========");
    Ok(())
}
