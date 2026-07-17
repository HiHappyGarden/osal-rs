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

use core::mem::size_of;
use core::time::Duration;
use osal_rs::os::*;
use osal_rs::os::types::UBaseType;
use osal_rs::utils::Result;
use osal_rs::{log_debug, log_info};

#[cfg(not(feature = "serde"))]
use osal_rs::os::{Deserialize, Serialize};

#[cfg(not(feature = "serde"))]
use osal_rs::utils::Error;

#[cfg(not(feature = "serde"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct QueueMessage {
    bytes: [u8; 4],
}

#[cfg(not(feature = "serde"))]
impl BytesHasLen for QueueMessage {
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(not(feature = "serde"))]
impl Serialize for QueueMessage {
    fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(not(feature = "serde"))]
impl Deserialize for QueueMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 4 {
            return Err(Error::ReadError("invalid queue message length"));
        }

        Ok(Self {
            bytes: [bytes[0], bytes[1], bytes[2], bytes[3]],
        })
    }
}

#[cfg(feature = "serde")]
type QueueMessage = [u8; 4];

fn sample_message() -> QueueMessage {
    #[cfg(not(feature = "serde"))]
    {
        QueueMessage { bytes: [1, 2, 3, 4] }
    }

    #[cfg(feature = "serde")]
    {
        [1, 2, 3, 4]
    }
}

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
fn test_queue_from_isr() -> Result<()> {
    log_info!(TAG, "Starting test_queue_from_isr");
    let queue = Queue::new(4, 4)?;

    let data: u32 = 0xDEADBEEF;
    let bytes = data.to_le_bytes();

    let post_result = queue.post_from_isr(&bytes);
    log_debug!(TAG, "post_from_isr ok: {}", post_result.is_ok());
    assert!(post_result.is_ok());

    let mut received = [0u8; 4];
    let fetch_result = queue.fetch_from_isr(&mut received);
    log_debug!(TAG, "fetch_from_isr ok: {}", fetch_result.is_ok());
    assert!(fetch_result.is_ok());
    assert_eq!(u32::from_le_bytes(received), data);

    log_info!(TAG, "test_queue_from_isr PASSED");
    Ok(())
}

#[test]
fn test_queue_with_to_tick() -> Result<()> {
    log_info!(TAG, "Starting test_queue_with_to_tick");
    let queue = Queue::new(4, 4)?;

    let data: u32 = 0xC0FFEE;
    let bytes = data.to_le_bytes();

    queue.post_with_to_tick(&bytes, Duration::from_millis(100))?;

    let mut received = [0u8; 4];
    queue.fetch_with_to_tick(&mut received, Duration::from_millis(100))?;
    log_debug!(TAG, "Received via with_to_tick: 0x{:X}", u32::from_le_bytes(received));
    assert_eq!(u32::from_le_bytes(received), data);

    log_info!(TAG, "test_queue_with_to_tick PASSED");
    Ok(())
}

#[test]
fn test_queue_streamed() -> Result<()> {
    log_info!(TAG, "Starting test_queue_streamed");
    let mut streamed = QueueStreamed::<QueueMessage>::new(4, size_of::<QueueMessage>() as UBaseType)?;

    let message = sample_message();
    streamed.post(&message, Duration::from_millis(100).to_ticks())?;

    let mut received = sample_message();
    streamed.fetch(&mut received, Duration::from_millis(100).to_ticks())?;
    log_debug!(TAG, "QueueStreamed post/fetch round-tripped a message");

    streamed.post_from_isr(&message)?;
    streamed.fetch_from_isr(&mut received)?;
    log_debug!(TAG, "QueueStreamed post_from_isr/fetch_from_isr round-tripped a message");

    streamed.delete();
    log_info!(TAG, "test_queue_streamed PASSED");
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
