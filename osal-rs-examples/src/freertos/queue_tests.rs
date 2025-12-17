extern crate alloc;

use alloc::vec;
use osal_rs::os::*;
use osal_rs::utils::Result;
use core::time::Duration;

pub fn test_queue_creation() -> Result<()> {
    let queue = Queue::new(10, 4);
    assert!(queue.is_ok());
    
    if let Ok(mut q) = queue {
        q.delete();
    }
    Ok(())
}

pub fn test_queue_post_fetch() -> Result<()> {
    let queue = Queue::new(10, 4)?;
    
    let data: u32 = 0x12345678;
    let bytes = data.to_le_bytes();
    
    let post_result = queue.post(&bytes, Duration::from_millis(100).to_ticks());
    assert!(post_result.is_ok());
    
    let mut received = [0u8; 4];
    let fetch_result = queue.fetch(&mut received, Duration::from_millis(100).to_ticks());
    assert!(fetch_result.is_ok());
    
    let received_data = u32::from_le_bytes(received);
    assert_eq!(received_data, data);
    Ok(())
}

pub fn test_queue_timeout() -> Result<()> {
    let queue = Queue::new(10, 4)?;
    
    let mut buffer = [0u8; 4];
    let result = queue.fetch(&mut buffer, Duration::from_millis(10).to_ticks());
    assert!(result.is_err());
    Ok(())
}

pub fn test_queue_multiple_items() -> Result<()> {
    let queue = Queue::new(5, 4)?;
    
    for i in 0..5u32 {
        let bytes = i.to_le_bytes();
        let result = queue.post(&bytes, Duration::from_millis(100).to_ticks());
        assert!(result.is_ok());
    }
    
    for i in 0..5u32 {
        let mut received = [0u8; 4];
        let result = queue.fetch(&mut received, Duration::from_millis(100).to_ticks());
        assert!(result.is_ok());
        
        let received_data = u32::from_le_bytes(received);
        assert_eq!(received_data, i);
    }
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
    let queue = Queue::new(10, 4)?;
    drop(queue);
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    test_queue_creation()?;
    test_queue_post_fetch()?;
    test_queue_timeout()?;
    test_queue_multiple_items()?;
    // test_queue_streamed()?;  // Commented - requires types with ToBytes/FromBytes traits
    // test_queue_streamed_multiple()?;  // Commented - requires types with ToBytes/FromBytes traits
    test_queue_drop()?;
    Ok(())
}
