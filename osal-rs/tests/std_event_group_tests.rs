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

//! Ported 1:1 from osal-rs-tests' FreeRTOS suite (`event_group_tests.rs`) to
//! run against the POSIX backend. `posix::EventGroup` is currently a stub
//! (`set`/`get`/`clear`/`wait` all return 0 unconditionally) — most
//! assertions below describe the intended behavior and fail until a real
//! implementation lands in `src/posix/event_group.rs`.

#![cfg(feature = "posix")]

use osal_rs::os::*;
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;
use core::time::Duration;
use osal_rs::{log_debug, log_info};

const TAG: &str = "EventGroupTests";

const BIT_0: EventBits = 1 << 0;
const BIT_1: EventBits = 1 << 1;
const BIT_2: EventBits = 1 << 2;
const BIT_3: EventBits = 1 << 3;

#[test]
fn test_event_group_creation() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_creation");
    let event_group = EventGroup::new();
    assert!(event_group.is_ok());
    log_info!(TAG, "test_event_group_creation PASSED");
    Ok(())
}

#[test]
fn test_event_group_set_get() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_set_get");
    let event_group = EventGroup::new()?;

    let result = event_group.set(BIT_0);
    log_debug!(TAG, "Set BIT_0, result: 0x{:X}", result);
    assert_ne!(result, 0);

    let bits = event_group.get();
    log_debug!(TAG, "Current bits: 0x{:X}", bits);
    assert_eq!(bits & BIT_0, BIT_0);
    log_info!(TAG, "test_event_group_set_get PASSED");
    Ok(())
}

#[test]
fn test_event_group_multiple_bits() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_multiple_bits");
    let event_group = EventGroup::new()?;

    event_group.set(BIT_0 | BIT_1 | BIT_2);

    let bits = event_group.get();
    log_debug!(TAG, "Set bits: 0x{:X}", bits);
    assert_eq!(bits & BIT_0, BIT_0);
    assert_eq!(bits & BIT_1, BIT_1);
    assert_eq!(bits & BIT_2, BIT_2);
    log_info!(TAG, "test_event_group_multiple_bits PASSED");
    Ok(())
}

#[test]
fn test_event_group_clear() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_clear");
    let event_group = EventGroup::new()?;

    event_group.set(BIT_0 | BIT_1 | BIT_2);

    log_debug!(TAG, "Clearing BIT_1");
    event_group.clear(BIT_1);

    let bits = event_group.get();
    log_debug!(TAG, "Remaining bits: 0x{:X}", bits);
    assert_eq!(bits & BIT_0, BIT_0);
    assert_eq!(bits & BIT_1, 0);
    assert_eq!(bits & BIT_2, BIT_2);
    log_info!(TAG, "test_event_group_clear PASSED");
    Ok(())
}

#[test]
fn test_event_group_clear_all() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_clear_all");
    let event_group = EventGroup::new()?;

    event_group.set(BIT_0 | BIT_1 | BIT_2 | BIT_3);

    log_debug!(TAG, "Clearing all bits");
    event_group.clear(BIT_0 | BIT_1 | BIT_2 | BIT_3);

    let bits = event_group.get();
    log_debug!(TAG, "All bits cleared: 0x{:X}", bits);
    assert_eq!(bits, 0);
    log_info!(TAG, "test_event_group_clear_all PASSED");
    Ok(())
}

#[test]
fn test_event_group_wait() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_wait");
    let event_group = EventGroup::new()?;

    event_group.set(BIT_0 | BIT_1);

    log_debug!(TAG, "Waiting for BIT_0 and BIT_1");
    let result = event_group.wait(BIT_0 | BIT_1, Duration::from_millis(100).to_ticks());
    log_debug!(TAG, "Wait result: 0x{:X}", result);
    assert_eq!(result & BIT_0, BIT_0);
    assert_eq!(result & BIT_1, BIT_1);
    log_info!(TAG, "test_event_group_wait PASSED");
    Ok(())
}

#[test]
fn test_event_group_wait_timeout() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_wait_timeout");
    let event_group = EventGroup::new()?;

    let result = event_group.wait(BIT_0, Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Wait timeout result: 0x{:X}", result);
    assert_eq!(result, 0);
    log_info!(TAG, "test_event_group_wait_timeout PASSED");
    Ok(())
}

#[test]
fn test_event_group_wait_partial() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_wait_partial");
    let event_group = EventGroup::new()?;

    event_group.set(BIT_0);

    log_debug!(TAG, "Waiting for BIT_0 | BIT_1 (only BIT_0 set)");
    let result = event_group.wait(BIT_0 | BIT_1, Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Partial wait result: 0x{:X}", result);
    assert_eq!(result & BIT_0, BIT_0);
    log_info!(TAG, "test_event_group_wait_partial PASSED");
    Ok(())
}

#[test]
fn test_event_group_sequential_operations() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_sequential_operations");
    let event_group = EventGroup::new()?;

    event_group.set(BIT_0);
    assert_eq!(event_group.get() & BIT_0, BIT_0);

    event_group.set(BIT_1);
    assert_eq!(event_group.get() & (BIT_0 | BIT_1), BIT_0 | BIT_1);

    log_debug!(TAG, "Clearing BIT_0");
    event_group.clear(BIT_0);
    assert_eq!(event_group.get() & BIT_0, 0);
    assert_eq!(event_group.get() & BIT_1, BIT_1);

    event_group.set(BIT_2);
    assert_eq!(event_group.get() & (BIT_1 | BIT_2), BIT_1 | BIT_2);
    log_info!(TAG, "test_event_group_sequential_operations PASSED");
    Ok(())
}

#[test]
fn test_event_group_all_bits() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_all_bits");
    let event_group = EventGroup::new()?;

    let all_bits = 0x00FFFFFF;
    event_group.set(all_bits);

    let bits = event_group.get();
    log_debug!(TAG, "All bits set: 0x{:X}", bits);
    assert_eq!(bits & all_bits, all_bits);
    log_info!(TAG, "test_event_group_all_bits PASSED");
    Ok(())
}

#[test]
fn test_event_group_drop() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_drop");
    let event_group = EventGroup::new()?;
    event_group.set(BIT_0 | BIT_1);
    drop(event_group);
    log_info!(TAG, "test_event_group_drop PASSED");
    Ok(())
}
