/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

extern crate alloc;

use osal_rs::os::*;
use osal_rs::utils::Result;
use core::time::Duration;
use osal_rs::{log_debug, log_info};

const TAG: &str = "QueueTests";

pub fn test_queue_creation() -> Result<()> {
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

pub fn test_queue_post_fetch() -> Result<()> {
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

pub fn test_queue_timeout() -> Result<()> {
    log_info!(TAG, "Starting test_queue_timeout");
    let queue = Queue::new(10, 4)?;
    
    let mut buffer = [0u8; 4];
    let result = queue.fetch(&mut buffer, Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Fetch timeout result: {:?}", result.is_err());
    assert!(result.is_err());
    log_info!(TAG, "test_queue_timeout PASSED");
    Ok(())
}

pub fn test_queue_multiple_items() -> Result<()> {
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

// Note: QueueStreamed requires types that implement ToBytes, BytesHasLen, and FromBytes traits
// u32 does not implement these traits, so these tests are commented out for embedded use
/*
pub fn test_queue_streamed() -> Result<()> {
    let queue = QueueStreamed::<u32>::new(10)?;
    
    let data: u32 = 42;
    let post_result = queue.post(&data, Duration::from_millis(100).to_ticks());
    assert!(post_result.is_ok());
    
    let fetch_result = queue.fetch(Duration::from_millis(100).to_ticks());
    assert!(fetch_result.is_ok());
    
    if let Ok(received) = fetch_result {
        assert_eq!(received, data);
    }
    Ok(())
}

pub fn test_queue_streamed_multiple() -> Result<()> {
    let queue = QueueStreamed::<u32>::new(5)?;
    
    for i in 10..15u32 {
        let result = queue.post(&i, Duration::from_millis(100).to_ticks());
        assert!(result.is_ok());
    }
    
    for i in 10..15u32 {
        let result = queue.fetch(Duration::from_millis(100).to_ticks());
        assert!(result.is_ok());
        
        if let Ok(received) = result {
            assert_eq!(received, i);
        }
    }
    Ok(())
}
*/

pub fn test_queue_drop() -> Result<()> {
    log_info!(TAG, "Starting test_queue_drop");
    let queue = Queue::new(10, 4)?;
    drop(queue);
    log_info!(TAG, "test_queue_drop PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Queue Tests ==========");
    test_queue_creation()?;
    test_queue_post_fetch()?;
    test_queue_timeout()?;
    test_queue_multiple_items()?;
    // test_queue_streamed()?;  // Commented - requires types with ToBytes/FromBytes traits
    // test_queue_streamed_multiple()?;  // Commented - requires types with ToBytes/FromBytes traits
    test_queue_drop()?;
    log_info!(TAG, "========== All Queue Tests PASSED ==========");
    Ok(())
}
