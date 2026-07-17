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

//! Ported 1:1 from osal-rs-tests' FreeRTOS suite (`queue_tests.rs`) to run
//! against the POSIX backend. `posix::Queue` is currently a stub — `post`
//! always succeeds but nothing is actually stored, and `fetch` always
//! returns `Err(Error::Timeout)` — so round-trip assertions fail until a
//! real implementation lands in `src/posix/queue.rs`.

#![cfg(feature = "posix")]

use osal_rs::os::*;
use osal_rs::utils::Result;
use core::time::Duration;
use osal_rs::{log_debug, log_info};

const TAG: &str = "QueueTests";

#[test]
fn test_queue_creation() -> Result<()> {
    log_info!(TAG, "Starting test_queue_creation");
    let queue = Queue::new(10, 4);
    assert!(queue.is_ok());

    if let Ok(mut q) = queue {
        log_debug!(TAG, "Queue created successfully, deleting...");
        q.delete();
    }
    log_info!(TAG, "test_queue_creation PASSED");
    Ok(())
}

#[test]
fn test_queue_post_fetch() -> Result<()> {
    log_info!(TAG, "Starting test_queue_post_fetch");
    let queue = Queue::new(10, 4)?;

    let data: u32 = 0x12345678;
    let bytes = data.to_le_bytes();

    log_debug!(TAG, "Posting data: 0x{:X}", data);
    let post_result = queue.post(&bytes, Duration::from_millis(100).to_ticks());
    assert!(post_result.is_ok());

    let mut received = [0u8; 4];
    let fetch_result = queue.fetch(&mut received, Duration::from_millis(100).to_ticks());
    assert!(fetch_result.is_ok());

    let received_data = u32::from_le_bytes(received);
    log_debug!(TAG, "Received data: 0x{:X}", received_data);
    assert_eq!(received_data, data);
    log_info!(TAG, "test_queue_post_fetch PASSED");
    Ok(())
}

#[test]
fn test_queue_timeout() -> Result<()> {
    log_info!(TAG, "Starting test_queue_timeout");
    let queue = Queue::new(10, 4)?;

    let mut buffer = [0u8; 4];
    let result = queue.fetch(&mut buffer, Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Fetch timeout result: {:?}", result.is_err());
    assert!(result.is_err());
    log_info!(TAG, "test_queue_timeout PASSED");
    Ok(())
}

#[test]
fn test_queue_multiple_items() -> Result<()> {
    log_info!(TAG, "Starting test_queue_multiple_items");
    let queue = Queue::new(5, 4)?;

    log_debug!(TAG, "Posting 5 items...");
    for i in 0..5u32 {
        let bytes = i.to_le_bytes();
        let result = queue.post(&bytes, Duration::from_millis(100).to_ticks());
        assert!(result.is_ok());
    }

    log_debug!(TAG, "Fetching 5 items...");
    for i in 0..5u32 {
        let mut received = [0u8; 4];
        let result = queue.fetch(&mut received, Duration::from_millis(100).to_ticks());
        assert!(result.is_ok());

        let received_data = u32::from_le_bytes(received);
        assert_eq!(received_data, i);
    }
    log_info!(TAG, "test_queue_multiple_items PASSED");
    Ok(())
}

#[test]
fn test_queue_drop() -> Result<()> {
    log_info!(TAG, "Starting test_queue_drop");
    let queue = Queue::new(10, 4)?;
    drop(queue);
    log_info!(TAG, "test_queue_drop PASSED");
    Ok(())
}
