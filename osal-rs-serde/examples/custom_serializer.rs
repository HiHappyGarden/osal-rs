/***************************************************************************
 *
 * osal-rs-serde - Custom Serializer Example
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

//! Example showing how to create a custom serializer.
//! This example implements a simple text-based serializer.

use osal_rs_serde::{Serialize, Deserialize, Serializer, Error};

/// A simple text-based serializer that writes values as comma-separated strings
struct TextSerializer<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> TextSerializer<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        let bytes = s.as_bytes();
        if self.position + bytes.len() > self.buffer.len() {
            return Err(Error::BufferTooSmall);
        }
        self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
        Ok(())
    }

    fn write_separator(&mut self) -> Result<(), Error> {
        self.write_str(",")
    }

    fn position(&self) -> usize {
        self.position
    }
}

impl<'a> Serializer for TextSerializer<'a> {
    type Error = Error;

    fn serialize_bool(&mut self, _name: &str, v: bool) -> Result<(), Error> {
        self.write_str(if v { "true" } else { "false" })?;
        self.write_separator()
    }

    fn serialize_u8(&mut self, _name: &str, v: u8) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v as i64, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_i8(&mut self, _name: &str, v: i8) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v as i64, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_u16(&mut self, _name: &str, v: u16) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v as i64, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_i16(&mut self, _name: &str, v: i16) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v as i64, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_u32(&mut self, _name: &str, v: u32) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v as i64, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_i32(&mut self, _name: &str, v: i32) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v as i64, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_u64(&mut self, _name: &str, v: u64) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v as i64, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_i64(&mut self, _name: &str, v: i64) -> Result<(), Error> {
        let mut buf = [0u8; 20];
        let s = format_number(v, &mut buf);
        self.write_str(s)?;
        self.write_separator()
    }

    fn serialize_u128(&mut self, _name: &str, _v: u128) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_i128(&mut self, _name: &str, _v: i128) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_f32(&mut self, _name: &str, _v: f32) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_f64(&mut self, _name: &str, _v: f64) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_bytes(&mut self, _name: &str, _v: &[u8]) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_string(&mut self, _name: &str, _v: &String) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_str(&mut self, _name: &str, _v: &str) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_vec<T: osal_rs_serde::Serialize>(&mut self, _name: &str, _v: &Vec<T>) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn serialize_array<T: osal_rs_serde::Serialize>(&mut self, _name: &str, _v: &[T]) -> Result<(), Error> {
        Err(Error::Unsupported)
    }
}

// Simple number formatter for no-std environment
fn format_number(mut n: i64, buf: &mut [u8]) -> &str {
    if n == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap();
    }

    let negative = n < 0;
    if negative {
        n = -n;
    }

    let mut pos = buf.len();
    while n > 0 {
        pos -= 1;
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
    }

    if negative {
        pos -= 1;
        buf[pos] = b'-';
    }

    core::str::from_utf8(&buf[pos..]).unwrap()
}

/// Example data structure
#[derive(Serialize, Deserialize, Debug)]
struct SensorReading {
    id: u16,
    value: i32,
    valid: bool,
}

fn main() {
    println!("=== OSAL-RS-Serde Custom Serializer Example ===\n");

    let reading = SensorReading {
        id: 42,
        value: -273,
        valid: true,
    };

    println!("Original data: {:?}", reading);

    // Use custom text serializer
    let mut buffer = [0u8; 128];
    let mut serializer = TextSerializer::new(&mut buffer);
    reading.serialize("", &mut serializer).unwrap();
    
    let len = serializer.position();
    let text = core::str::from_utf8(&buffer[..len]).unwrap();
    println!("Text serialized ({} bytes): {}", len, text);

    // Compare with binary serializer
    let mut bin_buffer = [0u8; 128];
    let bin_len = osal_rs_serde::to_bytes(&reading, &mut bin_buffer).unwrap();
    println!("Binary serialized ({} bytes): {:?}", bin_len, &bin_buffer[..bin_len]);

    println!("\nText format is more readable but uses more space!");
    println!("Text: {} bytes, Binary: {} bytes", len, bin_len);

    println!("\n=== Example completed successfully! ===");
}
