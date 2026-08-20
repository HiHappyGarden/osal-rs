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
//! content is duplicated in `osal-rs/tests/std_utils_tests.rs` - keep the two
//! in sync. The `osal-rs-serde` round-trip at the end needs the
//! `serde` feature on both sides.

extern crate alloc;

use core::ptr::null;

use alloc::ffi::CString;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use osal_rs::utils::{
    bytes_to_hex, bytes_to_hex_into_slice, hex_to_bytes, hex_to_bytes_into_slice,
    register_bit_size, AsSyncStr, Bytes, CpuRegisterSize, Error, OsalRsBool, Result, MAX_DELAY,
};
use osal_rs::{log_debug, log_info};

const TAG: &str = "UtilsTests";

pub fn test_bytes_construction() -> Result<()> {
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

pub fn test_bytes_str_conversion() -> Result<()> {
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

pub fn test_bytes_append_prepend() -> Result<()> {
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

pub fn test_bytes_mutation() -> Result<()> {
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

pub fn test_hex_helpers() -> Result<()> {
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

pub fn test_error_display_all_variants() -> Result<()> {
    log_info!(TAG, "Starting test_error_display_all_variants");

    // Every variant must render a non-empty, distinct message: these strings
    // are what ends up in logs when an OSAL call fails.
    let cases: [(Error<'static>, &str); 20] = [
        (Error::OutOfMemory, "Out of memory"),
        (Error::QueueSendTimeout, "Queue send timeout"),
        (Error::QueueReceiveTimeout, "Queue receive timeout"),
        (Error::MutexTimeout, "Mutex timeout"),
        (Error::MutexLockFailed, "Mutex lock failed"),
        (Error::Timeout, "Operation timeout"),
        (Error::QueueFull, "Queue full"),
        (Error::StringConversionError, "String conversion error"),
        (Error::TaskNotFound, "Task not found"),
        (Error::InvalidQueueSize, "Invalid queue size"),
        (Error::NullPtr, "Null pointer encountered"),
        (Error::NotFound, "Item not found"),
        (Error::OutOfIndex, "Index out of bounds"),
        (Error::InvalidType, "Invalid type for operation"),
        (Error::Empty, "No data available"),
        (Error::WriteError("disk"), "Write error occurred: disk"),
        (Error::ReadError("eof"), "Read error occurred: eof"),
        (Error::ReturnWithCode(-7), "Return with code: -7"),
        (Error::Unhandled("boom"), "Unhandled error: boom"),
        (
            Error::UnhandledOwned(String::from("owned boom")),
            "Unhandled error owned: owned boom",
        ),
    ];

    for (error, expected) in &cases {
        let rendered = format!("{}", error);
        log_debug!(TAG, "{:?} -> {}", error, rendered);
        assert_eq!(rendered, *expected);
    }

    // `Debug` (derived) and equality are used by `assert!(matches!(..))` in
    // the rest of the suite, so exercise them here too.
    assert_eq!(Error::Timeout, Error::Timeout);
    assert_ne!(Error::Timeout, Error::QueueFull);
    assert!(!format!("{:?}", Error::NullPtr).is_empty());

    log_info!(TAG, "test_error_display_all_variants PASSED");
    Ok(())
}

pub fn test_osal_rs_bool_and_constants() -> Result<()> {
    log_info!(TAG, "Starting test_osal_rs_bool_and_constants");

    assert_ne!(OsalRsBool::True, OsalRsBool::False);
    assert_eq!(OsalRsBool::True, OsalRsBool::True);
    log_debug!(TAG, "OsalRsBool: {:?} / {:?}", OsalRsBool::True, OsalRsBool::False);

    // `MAX_DELAY` is the "wait forever" sentinel handed to blocking calls.
    assert!(MAX_DELAY.as_millis() > 0);

    // `register_bit_size` is a `const fn`, so it must also work in a const
    // context, not just at runtime.
    const SIZE: CpuRegisterSize = register_bit_size();
    assert_eq!(SIZE, register_bit_size());
    assert!(matches!(SIZE, CpuRegisterSize::Bit32 | CpuRegisterSize::Bit64));
    assert_eq!(
        SIZE,
        if size_of::<usize>() == 8 {
            CpuRegisterSize::Bit64
        } else {
            CpuRegisterSize::Bit32
        }
    );

    log_info!(TAG, "test_osal_rs_bool_and_constants PASSED");
    Ok(())
}

pub fn test_bytes_trait_conversions() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_trait_conversions");

    // `Default` == `new()`: an all-zero buffer.
    let default_bytes: Bytes<16> = Default::default();
    assert_eq!(default_bytes.len(), 0);
    assert_eq!(default_bytes.as_raw_bytes(), b"");
    assert_eq!(default_bytes.to_bytes(), Bytes::<16>::new().to_bytes());

    // `FromStr` (via `parse`) and `From<&str>` (via `into`) must agree with
    // the inherent `from_str`.
    let parsed: Bytes<16> = "Hello".parse().unwrap();
    let converted: Bytes<16> = "Hello".into();
    assert_eq!(parsed.as_str(), "Hello");
    assert_eq!(converted.as_str(), "Hello");
    assert_eq!(parsed.to_bytes(), converted.to_bytes());

    // `Deref`/`DerefMut` expose the raw `[u8; SIZE]`, including indexing.
    let mut indexed: Bytes<8> = "abc".into();
    assert_eq!(indexed[0], b'a');
    assert_eq!(indexed.iter().position(|&b| b == 0), Some(3));
    indexed[0] = b'A';
    indexed[3] = b'd';
    assert_eq!(indexed.as_str(), "Abcd");

    // `Display` renders the string content; `Debug` is derived, so it shows
    // the raw byte array instead.
    let shown: Bytes<16> = "shown".into();
    assert_eq!(format!("{}", shown), "shown");
    assert!(format!("{:?}", shown).starts_with("Bytes(["));

    // `Display` on invalid UTF-8 falls back to a diagnostic instead of
    // panicking.
    let invalid = Bytes::<2>::from_bytes(&[0xFF, 0xFE]);
    assert!(!format!("{}", invalid).is_empty());

    // `Clone`/`PartialEq` on the concrete type.
    let cloned = shown.clone();
    assert_eq!(cloned.as_str(), shown.as_str());

    log_info!(TAG, "test_bytes_trait_conversions PASSED");
    Ok(())
}

pub fn test_bytes_into_vec() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_into_vec");

    // The zero padding is stripped at the first null terminator.
    let padded: Bytes<16> = "Hello".into();
    let vec: Vec<u8> = padded.into();
    assert_eq!(vec.as_slice(), b"Hello");
    assert_eq!(vec.len(), 5);

    // `From` and the blanket `Into` must agree, and since `Bytes` is `Copy`
    // the source buffer is still usable afterwards.
    let from_vec = Vec::from(padded);
    let into_vec: Vec<u8> = padded.into();
    assert_eq!(from_vec, into_vec);
    assert_eq!(padded.as_str(), "Hello");

    // The inherent `into_vec` yields the same vector without any type
    // annotation, while the `to_vec()` reached through `Deref` on `[u8; SIZE]`
    // copies the whole padded array instead.
    assert_eq!(padded.into_vec(), from_vec);
    assert_eq!(padded.to_vec().len(), 16);
    assert_eq!(padded.to_vec()[5], 0);

    // The owning conversion sees exactly the borrowed `as_raw_bytes` view.
    assert_eq!(into_vec.as_slice(), padded.as_raw_bytes());
    assert_eq!(into_vec.len(), padded.len());

    // A completely filled buffer has no terminator: every byte is kept.
    let full = Bytes::<5>::from_str("Hello");
    let full_vec: Vec<u8> = full.into();
    assert_eq!(full_vec.len(), 5);
    assert_eq!(full_vec.as_slice(), b"Hello");

    // Truncating construction: only the first `SIZE` bytes survive, and none
    // of them is zero, so the whole buffer is converted.
    let truncated: Vec<u8> = Bytes::<3>::from_str("Hello").into();
    assert_eq!(truncated.as_slice(), b"Hel");

    // An empty buffer yields an empty vector.
    let empty: Vec<u8> = Bytes::<8>::new().into();
    assert!(empty.is_empty());

    // Binary payloads stop at the first embedded zero, so bytes after it are
    // not reachable through this conversion.
    let binary = Bytes::<8>::from_bytes(&[0xDE, 0xAD, 0x00, 0xBE, 0xEF]).into_vec();
    assert_eq!(binary.as_slice(), &[0xDEu8, 0xAD]);

    // Invalid UTF-8 is preserved byte for byte: no string round-trip happens.
    let invalid = Bytes::<2>::from_bytes(&[0xFF, 0xFE]).into_vec();
    assert_eq!(invalid.as_slice(), &[0xFFu8, 0xFE]);

    log_debug!(TAG, "converted {} bytes out of {}", into_vec.len(), padded.size());

    log_info!(TAG, "test_bytes_into_vec PASSED");
    Ok(())
}

pub fn test_as_sync_str_trait_object() -> Result<()> {
    log_info!(TAG, "Starting test_as_sync_str_trait_object");

    let hello: Bytes<16> = "hello".into();
    let hello_again: Bytes<32> = "hello".into();
    let world: Bytes<16> = "world".into();

    // Compared through the trait object, so different `SIZE`s with the same
    // text are equal.
    let a: &dyn AsSyncStr = &hello;
    let b: &dyn AsSyncStr = &hello_again;
    let c: &dyn AsSyncStr = &world;

    assert_eq!(a.as_str(), "hello");
    assert!(a == b);
    assert!(a != c);

    log_debug!(TAG, "dyn AsSyncStr Display: {} / Debug: {:?}", a, a);
    assert_eq!(format!("{}", a), "hello");
    assert_eq!(format!("{:?}", a), "hello");

    log_info!(TAG, "test_as_sync_str_trait_object PASSED");
    Ok(())
}

pub fn test_bytes_null_pointer_constructors() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_null_pointer_constructors");

    // Null in, empty buffer out - no deref of the null pointer.
    assert_eq!(Bytes::<16>::from_char_ptr(null()).len(), 0);
    assert_eq!(Bytes::<16>::from_cstr(null()).len(), 0);
    assert_eq!(Bytes::<16>::from_uchar_ptr(null()).len(), 0);

    log_info!(TAG, "test_bytes_null_pointer_constructors PASSED");
    Ok(())
}

pub fn test_bytes_truncating_constructors() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_truncating_constructors");

    // Source longer than SIZE: every constructor truncates rather than
    // overflowing the fixed array.
    let long = CString::new("This is a very long string").unwrap();

    let from_char_ptr = Bytes::<8>::from_char_ptr(long.as_ptr());
    assert_eq!(from_char_ptr.as_raw_bytes(), b"This is ");

    let from_cstr = Bytes::<8>::from_cstr(long.as_ptr());
    assert_eq!(from_cstr.as_raw_bytes(), b"This is ");

    let from_bytes = Bytes::<4>::from_bytes(b"abcdefgh");
    assert_eq!(from_bytes.as_raw_bytes(), b"abcd");

    let from_str = Bytes::<3>::from_str("Hello");
    assert_eq!(from_str.as_raw_bytes(), b"Hel");

    let from_sync = Bytes::<4>::from_as_sync_str(&"abcdefgh");
    assert_eq!(from_sync.as_raw_bytes(), b"abcd");

    // Exactly SIZE: fills the buffer with no room for a null terminator, so
    // `len()` falls back to SIZE.
    let exact = Bytes::<5>::from_str("Hello");
    assert_eq!(exact.len(), 5);
    assert!(!exact.is_empty());
    assert_eq!(exact.as_str(), "Hello");

    log_info!(TAG, "test_bytes_truncating_constructors PASSED");
    Ok(())
}

pub fn test_bytes_append_prepend_truncation() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_append_prepend_truncation");

    // Appending past capacity stops at SIZE instead of panicking.
    let mut append_str = Bytes::<8>::from_str("Hello");
    append_str.append_str(" World");
    assert_eq!(append_str.as_raw_bytes(), b"Hello Wo");

    let mut append_bytes = Bytes::<8>::from_str("Hello");
    append_bytes.append_bytes(b" World");
    assert_eq!(append_bytes.as_raw_bytes(), b"Hello Wo");

    let mut append_other = Bytes::<8>::from_str("Hello");
    append_other.append(&Bytes::<16>::from_str(" World"));
    assert_eq!(append_other.as_raw_bytes(), b"Hello Wo");

    let mut append_sync = Bytes::<8>::from_str("Hello");
    append_sync.append_as_sync_str(&Bytes::<16>::from_str(" World"));
    assert_eq!(append_sync.as_raw_bytes(), b"Hello Wo");

    // Appending to a completely full buffer is a no-op.
    let mut full = Bytes::<5>::from_str("Hello");
    full.append_str("!!!");
    assert_eq!(full.as_raw_bytes(), b"Hello");

    // Prepending past capacity keeps the prefix and drops the tail.
    let mut prepend_str = Bytes::<8>::from_str("World");
    prepend_str.prepend_str("Hello ");
    assert_eq!(prepend_str.as_raw_bytes(), b"Hello Wo");

    let mut prepend_bytes = Bytes::<8>::from_str("World");
    prepend_bytes.prepend_bytes(b"Hello ");
    assert_eq!(prepend_bytes.as_raw_bytes(), b"Hello Wo");

    let mut prepend_other = Bytes::<8>::from_str("end");
    prepend_other.prepend(&Bytes::<32>::from_str("begin_"));
    assert_eq!(prepend_other.as_raw_bytes(), b"begin_en");

    let mut prepend_sync = Bytes::<8>::from_str("World");
    prepend_sync.prepend_as_sync_str(&Bytes::<16>::from_str("Hello "));
    assert_eq!(prepend_sync.as_raw_bytes(), b"Hello Wo");

    // A prefix longer than the whole buffer replaces the content outright.
    let mut swamped = Bytes::<4>::from_str("xy");
    swamped.prepend_str("abcdef");
    assert_eq!(swamped.as_raw_bytes(), b"abcd");

    // Prepending onto an empty buffer.
    let mut empty = Bytes::<8>::new();
    empty.prepend_str("hi");
    assert_eq!(empty.as_str(), "hi");

    log_info!(TAG, "test_bytes_append_prepend_truncation PASSED");
    Ok(())
}

pub fn test_bytes_replace_variants() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_replace_variants");

    // Same-length replacement: no shifting.
    let mut same = Bytes::<16>::from_str("Hello World");
    same.replace(b"World", b"Rust!")?;
    assert_eq!(same.as_str(), "Hello Rust!");

    // Shorter replacement: the tail shifts left and the slack is zeroed.
    let mut shorter = Bytes::<16>::from_str("Hello World");
    shorter.replace(b"World", b"Yo")?;
    assert_eq!(shorter.as_str(), "Hello Yo");
    assert_eq!(shorter.len(), 8);

    // Longer replacement that still fits: the tail shifts right.
    let mut longer = Bytes::<24>::from_str("a-b");
    longer.replace(b"-", b"+++")?;
    assert_eq!(longer.as_str(), "a+++b");

    // Multiple occurrences are all replaced.
    let mut repeated = Bytes::<16>::from_str("aXbXc");
    repeated.replace(b"X", b"-")?;
    assert_eq!(repeated.as_str(), "a-b-c");

    // Pattern absent: content untouched.
    let mut absent = Bytes::<16>::from_str("Hello");
    absent.replace(b"zzz", b"!")?;
    assert_eq!(absent.as_str(), "Hello");

    // Empty pattern: early return, nothing changes.
    let mut empty_pattern = Bytes::<16>::from_str("Hello");
    empty_pattern.replace(b"", b"!")?;
    assert_eq!(empty_pattern.as_str(), "Hello");

    // Deletion via an empty replacement.
    let mut deleted = Bytes::<16>::from_str("a-b-c");
    deleted.replace(b"-", b"")?;
    assert_eq!(deleted.as_str(), "abc");

    // Does not fit: reported instead of overflowing.
    let mut too_small = Bytes::<8>::from_str("Hello");
    assert!(matches!(
        too_small.replace(b"Hello", b"Hello World"),
        Err(Error::StringConversionError)
    ));
    // The failed replace left the original content in place.
    assert_eq!(too_small.as_str(), "Hello");

    log_info!(TAG, "test_bytes_replace_variants PASSED");
    Ok(())
}

pub fn test_bytes_push_pop_edges() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_push_pop_edges");

    // Pop on empty.
    let mut empty = Bytes::<8>::new();
    assert_eq!(empty.pop(), None);
    assert_eq!(empty.pop_char(), None);

    // Push until full, then fail.
    let mut small = Bytes::<2>::new();
    small.push(b'a')?;
    small.push(b'b')?;
    assert!(matches!(small.push(b'c'), Err(Error::StringConversionError)));
    assert!(matches!(
        small.push_char('c'),
        Err(Error::StringConversionError)
    ));
    assert_eq!(small.as_raw_bytes(), b"ab");

    // Non-ASCII is rejected before any capacity check.
    let mut roomy = Bytes::<16>::new();
    assert!(matches!(
        roomy.push_char('é'),
        Err(Error::StringConversionError)
    ));
    roomy.push_char('o')?;
    assert_eq!(roomy.as_str(), "o");

    // Pop down to empty and back.
    let mut word = Bytes::<8>::from_str("hi");
    assert_eq!(word.pop(), Some(b'i'));
    assert_eq!(word.pop_char(), Some('h'));
    assert_eq!(word.pop(), None);
    assert!(word.is_empty());

    log_info!(TAG, "test_bytes_push_pop_edges PASSED");
    Ok(())
}

pub fn test_bytes_invalid_utf8() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_invalid_utf8");

    // 0xFF is never valid UTF-8.
    let invalid = Bytes::<4>::from_bytes(&[0xFF, 0xFE, 0xFD, 0xFC]);
    assert!(!invalid.is_string());
    assert_eq!(invalid.as_str(), "Bytes::as_str() Conversion error - invalid UTF-8");

    // A buffer whose *content* is valid but whose zero padding is not part of
    // the string still reports as a string.
    let valid = Bytes::<8>::from_str("ok");
    assert!(valid.is_string());

    // `fill_str` fails on invalid UTF-8 rather than producing a bad `&str`.
    let mut invalid_fill = Bytes::<4>::from_bytes(&[0xFF, 0xFE, 0xFD, 0xFC]);
    let mut dest = String::from("....");
    assert!(matches!(
        invalid_fill.fill_str(dest.as_mut_str()),
        Err(Error::StringConversionError)
    ));

    // `fill_str` into a destination shorter than the buffer copies only what
    // fits.
    let mut source = Bytes::<8>::from_str("abcdefgh");
    let mut short_dest = String::from("...");
    source.fill_str(short_dest.as_mut_str())?;
    assert_eq!(short_dest, "abc");

    log_info!(TAG, "test_bytes_invalid_utf8 PASSED");
    Ok(())
}

pub fn test_bytes_clear_and_capacity() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_clear_and_capacity");

    let mut bytes = Bytes::<16>::from_str("something");
    assert_eq!(bytes.capacity(), 16);
    assert_eq!(bytes.size(), 16);
    assert_eq!(bytes.to_bytes().len(), 16);

    bytes.clear();
    assert!(bytes.is_empty());
    assert_eq!(bytes.len(), 0);
    assert_eq!(bytes.as_str(), "");
    assert_eq!(bytes.as_cstr().to_bytes(), b"");

    // Clearing an already-empty buffer is a no-op.
    bytes.clear();
    assert!(bytes.is_empty());

    log_info!(TAG, "test_bytes_clear_and_capacity PASSED");
    Ok(())
}

pub fn test_bytes_format_and_write() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_format_and_write");

    use core::fmt::Write;

    // `format` replaces the whole content.
    let mut buffer = Bytes::<32>::from_str("stale content");
    buffer.format(format_args!("Hello {}", 42));
    assert_eq!(buffer.as_str(), "Hello 42");

    // Truncation on overflow rather than panic.
    let mut tiny = Bytes::<8>::new();
    tiny.format(format_args!("{:.2}", 3.14159));
    assert_eq!(tiny.as_str(), "3.14");

    let mut overflowing = Bytes::<4>::new();
    overflowing.format(format_args!("{}", "much too long"));
    assert_eq!(overflowing.as_raw_bytes(), b"much");

    // `core::fmt::Write` appends instead of replacing.
    let mut written = Bytes::<16>::from_str("a=");
    write!(written, "{}", 7).unwrap();
    write!(written, ",b={}", 8).unwrap();
    assert_eq!(written.as_str(), "a=7,b=8");

    log_info!(TAG, "test_bytes_format_and_write PASSED");
    Ok(())
}

pub fn test_hex_helper_edges() -> Result<()> {
    log_info!(TAG, "Starting test_hex_helper_edges");

    // Empty input on every helper.
    assert_eq!(bytes_to_hex(&[]), "");
    assert_eq!(bytes_to_hex_into_slice(&[], &mut []), 0);
    assert!(hex_to_bytes("")?.is_empty());
    assert_eq!(hex_to_bytes_into_slice("", &mut [])?, 0);

    // Uppercase input decodes the same as lowercase.
    assert_eq!(hex_to_bytes("ABCDEF")?, hex_to_bytes("abcdef")?);

    // Odd length is rejected on both decoders.
    assert!(matches!(hex_to_bytes("ABC"), Err(Error::StringConversionError)));
    let mut out = [0u8; 4];
    assert!(matches!(
        hex_to_bytes_into_slice("ABC", &mut out),
        Err(Error::StringConversionError)
    ));

    // Non-hex characters are rejected.
    assert!(hex_to_bytes("zz").is_err());
    assert!(hex_to_bytes_into_slice("zz", &mut out).is_err());

    // An oversized destination is fine: only `2 * bytes.len()` is written and
    // that count is what comes back. (An *under*sized destination is a
    // documented `assert!` panic, which can't be tested here - this workspace
    // builds with `panic = "abort"`, so `#[should_panic]` is unavailable.)
    let mut oversized = [b'.'; 12];
    let written = bytes_to_hex_into_slice(&[0x01, 0x23, 0xAB, 0xFF], &mut oversized);
    log_debug!(TAG, "bytes_to_hex_into_slice wrote {} bytes", written);
    assert_eq!(written, 8);
    assert_eq!(&oversized[..8], b"0123abff");
    assert_eq!(&oversized[8..], b"....");

    // Round-trip through both APIs.
    let data = [0x00u8, 0x0F, 0xF0, 0xFF];
    let hex = bytes_to_hex(&data);
    assert_eq!(hex, "000ff0ff");
    assert_eq!(hex_to_bytes(&hex)?, data);

    let mut round = [0u8; 4];
    assert_eq!(hex_to_bytes_into_slice(&hex, &mut round)?, 4);
    assert_eq!(round, data);

    log_info!(TAG, "test_hex_helper_edges PASSED");
    Ok(())
}

/// `Bytes<SIZE>` implements `osal-rs-serde`'s `Serialize`/`Deserialize` when
/// the `serde` feature is on (and the plain byte-slice `Serialize` from
/// `osal_rs::traits` when it is off). These cover the former; the latter is
/// exercised through `QueueStreamed` in `std_queue_tests.rs`.
#[cfg(feature = "serde")]
pub fn test_bytes_serde_round_trip() -> Result<()> {
    use osal_rs_serde::{from_bytes, to_dyn_bytes};

    log_info!(TAG, "Starting test_bytes_serde_round_trip");

    // Text content: serialized as a UTF-8 string.
    let text: Bytes<16> = "payload".into();
    let mut encoded = Vec::new();
    let written = to_dyn_bytes(&text, &mut encoded).unwrap();
    log_debug!(TAG, "serialized {:?} into {} bytes", text.as_str(), written);
    assert!(written > 0);

    let decoded: Bytes<16> = from_bytes(&encoded).unwrap();
    assert_eq!(decoded.as_str(), "payload");

    // Binary content: not valid UTF-8, so it takes the raw-bytes branch.
    let binary = Bytes::<4>::from_bytes(&[0xFF, 0x00, 0xFE, 0x01]);
    let mut binary_encoded = Vec::new();
    assert!(to_dyn_bytes(&binary, &mut binary_encoded).unwrap() > 0);

    // An empty buffer has no content to encode but must still round-trip.
    let empty = Bytes::<8>::new();
    let mut empty_encoded = Vec::new();
    to_dyn_bytes(&empty, &mut empty_encoded).unwrap();
    let empty_decoded: Bytes<8> = from_bytes(&empty_encoded).unwrap();
    assert!(empty_decoded.is_empty());

    log_info!(TAG, "test_bytes_serde_round_trip PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Utils Tests ==========");
    test_bytes_construction()?;
    test_bytes_str_conversion()?;
    test_bytes_append_prepend()?;
    test_bytes_mutation()?;
    test_hex_helpers()?;
    test_error_display_all_variants()?;
    test_osal_rs_bool_and_constants()?;
    test_bytes_trait_conversions()?;
    test_bytes_into_vec()?;
    test_as_sync_str_trait_object()?;
    test_bytes_null_pointer_constructors()?;
    test_bytes_truncating_constructors()?;
    test_bytes_append_prepend_truncation()?;
    test_bytes_replace_variants()?;
    test_bytes_push_pop_edges()?;
    test_bytes_invalid_utf8()?;
    test_bytes_clear_and_capacity()?;
    test_bytes_format_and_write()?;
    test_hex_helper_edges()?;
    #[cfg(feature = "serde")]
    test_bytes_serde_round_trip()?;
    log_info!(TAG, "========== All Utils Tests PASSED ==========");
    Ok(())
}
