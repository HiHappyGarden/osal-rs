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

//! Tests for `osal_rs::utils` (`Bytes<SIZE>` and the hex helper functions).
//! These are backend-independent (no FreeRTOS/POSIX calls), so the same
//! content is duplicated verbatim in
//! `osal-rs-tests/src/freertos/utils_tests.rs`. Gated on `posix` purely to
//! match this crate's convention of running its `tests/` suite against the
//! POSIX backend.

#![cfg(feature = "posix")]

use std::ffi::CString;

use osal_rs::utils::{
    bytes_to_hex, bytes_to_hex_into_slice, hex_to_bytes, hex_to_bytes_into_slice,
    register_bit_size, Bytes, CpuRegisterSize, Result,
};
use osal_rs::{log_debug, log_info};

const TAG: &str = "UtilsTests";

#[test]
fn test_bytes_construction() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_construction");

    let empty = Bytes::<16>::new();
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert_eq!(empty.size(), 16);
    assert_eq!(empty.capacity(), 16);

    let from_str = Bytes::<16>::from_str("Hello");
    assert_eq!(from_str.as_str(), "Hello");
    assert_eq!(from_str.len(), 5);

    let from_bytes = Bytes::<8>::from_bytes(b"ABCDE");
    assert_eq!(from_bytes.as_str(), "ABCDE");

    let c_string = CString::new("CText").unwrap();
    let from_char_ptr = Bytes::<16>::from_char_ptr(c_string.as_ptr());
    assert_eq!(from_char_ptr.as_str(), "CText");

    let from_cstr = Bytes::<16>::from_cstr(c_string.as_ptr());
    assert_eq!(from_cstr.as_str(), "CText");

    let raw = *b"UCHAR!!!";
    let from_uchar_ptr = Bytes::<8>::from_uchar_ptr(raw.as_ptr());
    assert_eq!(from_uchar_ptr.as_raw_bytes(), &raw[..]);

    let from_sync_str = Bytes::<16>::from_as_sync_str(&"Synced");
    assert_eq!(from_sync_str.as_str(), "Synced");

    log_debug!(TAG, "register_bit_size: {:?}", register_bit_size());
    assert!(matches!(register_bit_size(), CpuRegisterSize::Bit32 | CpuRegisterSize::Bit64));

    log_info!(TAG, "test_bytes_construction PASSED");
    Ok(())
}

#[test]
fn test_bytes_str_conversion() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_str_conversion");

    let mut bytes = Bytes::<16>::from_str("Hello World");
    assert_eq!(bytes.as_str(), "Hello World");
    assert_eq!(bytes.len(), 11);
    assert!(!bytes.is_empty());
    assert_eq!(bytes.as_raw_bytes(), b"Hello World");
    assert!(bytes.is_string());
    assert_eq!(bytes.to_bytes().len(), 16);

    // fill_str copies the *whole* zero-padded buffer, so only the leading
    // (real content) slice of `dest` is meaningful to check.
    let mut dest = String::from("................");
    bytes.fill_str(dest.as_mut_str())?;
    log_debug!(TAG, "fill_str result prefix: {}", &dest[..11]);
    assert_eq!(&dest[..11], "Hello World");

    let c_str = bytes.as_cstr();
    assert_eq!(c_str.to_bytes(), b"Hello World");

    let mut mutable = Bytes::<16>::from_str("Mutable");
    let c_str_mut = mutable.as_cstr_mut();
    assert_eq!(c_str_mut.to_bytes(), b"Mutable");

    log_info!(TAG, "test_bytes_str_conversion PASSED");
    Ok(())
}

#[test]
fn test_bytes_append_prepend() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_append_prepend");

    let mut bytes = Bytes::<16>::from_str("Hello");
    bytes.append_str(" World");
    assert_eq!(bytes.as_str(), "Hello World");

    let mut bytes2 = Bytes::<16>::from_str("Data: ");
    bytes2.append_bytes(&[0x41, 0x42, 0x43]);
    assert_eq!(bytes2.as_str(), "Data: ABC");

    let mut bytes3 = Bytes::<16>::from_str("Hello");
    let other = Bytes::<8>::from_str(" World");
    bytes3.append(&other);
    assert_eq!(bytes3.as_str(), "Hello World");

    let mut bytes4 = Bytes::<16>::from_str("Hello");
    let suffix = Bytes::<8>::from_str(" World");
    bytes4.append_as_sync_str(&suffix);
    assert_eq!(bytes4.as_str(), "Hello World");

    let mut prepend1 = Bytes::<16>::from_str("World");
    prepend1.prepend_str("Hello ");
    assert_eq!(prepend1.as_str(), "Hello World");

    let mut prepend2 = Bytes::<16>::from_str("World");
    prepend2.prepend_bytes(b"Hello ");
    assert_eq!(prepend2.as_str(), "Hello World");

    let mut prepend3 = Bytes::<16>::from_str("World");
    let prefix = Bytes::<8>::from_str("Hello ");
    prepend3.prepend(&prefix);
    assert_eq!(prepend3.as_str(), "Hello World");

    let mut prepend4 = Bytes::<16>::from_str("World");
    let prefix2 = Bytes::<8>::from_str("Hello ");
    prepend4.prepend_as_sync_str(&prefix2);
    assert_eq!(prepend4.as_str(), "Hello World");

    log_info!(TAG, "test_bytes_append_prepend PASSED");
    Ok(())
}

#[test]
fn test_bytes_mutation() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_mutation");

    let mut bytes = Bytes::<16>::from_str("Test");
    assert!(!bytes.is_empty());
    bytes.clear();
    assert!(bytes.is_empty());
    assert_eq!(bytes.len(), 0);

    let mut hw = Bytes::<16>::from_str("Hello");
    assert_eq!(hw.pop(), Some(b'o'));
    assert_eq!(hw.as_str(), "Hell");

    hw.push(b'!')?;
    assert_eq!(hw.as_str(), "Hell!");

    assert_eq!(hw.pop_char(), Some('!'));
    assert_eq!(hw.as_str(), "Hell");

    hw.push_char('o')?;
    assert_eq!(hw.as_str(), "Hello");

    let mut replaced = Bytes::<16>::from_str("Hello World");
    replaced.replace(b"World", b"Rust!")?;
    assert_eq!(replaced.as_str(), "Hello Rust!");

    let mut too_small = Bytes::<8>::from_str("Hello");
    assert!(too_small.replace(b"Hello", b"Hello World").is_err());

    let mut formatted = Bytes::<32>::new();
    formatted.format(format_args!("Hello {}", 42));
    assert_eq!(formatted.as_str(), "Hello 42");

    log_info!(TAG, "test_bytes_mutation PASSED");
    Ok(())
}

#[test]
fn test_hex_helpers() -> Result<()> {
    log_info!(TAG, "Starting test_hex_helpers");

    let data = [0x01u8, 0x23, 0xAB, 0xFF];
    let hex = bytes_to_hex(&data);
    assert_eq!(hex, "0123abff");

    let mut buffer = [0u8; 8];
    let written = bytes_to_hex_into_slice(&data, &mut buffer);
    assert_eq!(written, 8);
    assert_eq!(&buffer, b"0123abff");

    let decoded = hex_to_bytes("0123abff")?;
    assert_eq!(decoded.as_slice(), &data);
    assert!(hex_to_bytes("ABC").is_err());

    let mut out = [0u8; 4];
    let n = hex_to_bytes_into_slice("0123abff", &mut out)?;
    assert_eq!(n, 4);
    assert_eq!(out, data);

    let mut too_small = [0u8; 2];
    assert!(hex_to_bytes_into_slice("0123abff", &mut too_small).is_err());

    log_info!(TAG, "test_hex_helpers PASSED");
    Ok(())
}
