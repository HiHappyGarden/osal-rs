#[cfg(test)]
mod tests {
    extern crate alloc;
    
    use alloc::vec;
    use osal_rs::os::*;
    use osal_rs::utils::Result;
    use core::time::Duration;

    #[test]
    fn test_queue_creation() {
        let queue = Queue::new(10, 4);
        assert!(queue.is_ok());
        
        if let Ok(mut q) = queue {
            q.delete();
        }
    }

    #[test]
    fn test_queue_post_fetch() {
        let queue = Queue::new(10, 4).unwrap();
        
        let data: u32 = 0x12345678;
        let bytes = data.to_le_bytes();
        
        let post_result = queue.post(&bytes, Duration::from_millis(100));
        assert!(post_result.is_ok());
        
        let mut received = [0u8; 4];
        let fetch_result = queue.fetch(&mut received, Duration::from_millis(100));
        assert!(fetch_result.is_ok());
        
        let received_data = u32::from_le_bytes(received);
        assert_eq!(received_data, data);
    }

    #[test]
    fn test_queue_timeout() {
        let queue = Queue::new(10, 4).unwrap();
        
        let mut buffer = [0u8; 4];
        let result = queue.fetch(&mut buffer, Duration::from_millis(10));
        assert!(result.is_err());
    }

    #[test]
    fn test_queue_multiple_items() {
        let queue = Queue::new(5, 4).unwrap();
        
        for i in 0..5u32 {
            let bytes = i.to_le_bytes();
            let result = queue.post(&bytes, Duration::from_millis(100));
            assert!(result.is_ok());
        }
        
        for i in 0..5u32 {
            let mut received = [0u8; 4];
            let result = queue.fetch(&mut received, Duration::from_millis(100));
            assert!(result.is_ok());
            
            let received_data = u32::from_le_bytes(received);
            assert_eq!(received_data, i);
        }
    }

    #[test]
    fn test_queue_streamed() {
        let queue = QueueStreamed::<u32>::new(10).unwrap();
        
        let data: u32 = 42;
        let post_result = queue.post(&data, Duration::from_millis(100));
        assert!(post_result.is_ok());
        
        let fetch_result = queue.fetch(Duration::from_millis(100));
        assert!(fetch_result.is_ok());
        
        if let Ok(received) = fetch_result {
            assert_eq!(received, data);
        }
    }

    #[test]
    fn test_queue_streamed_multiple() {
        let queue = QueueStreamed::<u32>::new(5).unwrap();
        
        for i in 10..15u32 {
            let result = queue.post(&i, Duration::from_millis(100));
            assert!(result.is_ok());
        }
        
        for i in 10..15u32 {
            let result = queue.fetch(Duration::from_millis(100));
            assert!(result.is_ok());
            
            if let Ok(received) = result {
                assert_eq!(received, i);
            }
        }
    }

    #[test]
    fn test_queue_drop() {
        let queue = Queue::new(10, 4).unwrap();
        drop(queue);
    }
}
