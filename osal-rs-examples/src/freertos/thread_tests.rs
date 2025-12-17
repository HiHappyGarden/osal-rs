#[cfg(test)]
mod tests {
    extern crate alloc;
    
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use core::any::Any;
    use osal_rs::os::*;
    use osal_rs::utils::Result;

    #[test]
    fn test_thread_creation() {
        let thread = Thread::new(
            "test_thread",
            1024,
            5,
            |_thread, _param| {
                Ok(None)
            }
        );

        let metadata = thread.get_metadata();
        assert!(!metadata.name.is_empty());
        assert_eq!(metadata.stack_depth, 1024);
        assert_eq!(metadata.priority, 5);
    }

    #[test]
    fn test_thread_spawn() {
        let mut thread = Thread::new(
            "spawn_test",
            1024,
            5,
            |_thread, _param| {
                Ok(None)
            }
        );

        let result = thread.spawn(None);
        assert!(result.is_ok());
        
        if let Ok(spawned) = result {
            let metadata = spawned.get_metadata();
            assert!(!metadata.thread.is_null());
            spawned.delete();
        }
    }

    #[test]
    fn test_thread_with_param() {
        let test_value: u32 = 42;
        let param: Arc<dyn Any + Send + Sync> = Arc::new(test_value);
        
        let mut thread = Thread::new(
            "param_test",
            1024,
            5,
            |_thread, param| {
                if let Some(p) = param {
                    if let Some(val) = p.downcast_ref::<u32>() {
                        assert_eq!(*val, 42);
                    }
                }
                Ok(None)
            }
        );

        let result = thread.spawn(Some(param));
        assert!(result.is_ok());
        
        if let Ok(spawned) = result {
            spawned.delete();
        }
    }

    #[test]
    fn test_thread_suspend_resume() {
        let mut thread = Thread::new(
            "suspend_test",
            1024,
            5,
            |_thread, _param| {
                Ok(None)
            }
        );

        let spawned = thread.spawn(None).unwrap();
        spawned.suspend();
        spawned.resume();
        spawned.delete();
    }

    #[test]
    fn test_thread_get_metadata() {
        let mut thread = Thread::new(
            "metadata_test",
            1024,
            5,
            |_thread, _param| {
                Ok(None)
            }
        );

        let spawned = thread.spawn(None).unwrap();
        let metadata = spawned.get_metadata();
        
        assert_eq!(metadata.name, "metadata_test");
        assert_eq!(metadata.priority, 5);
        
        spawned.delete();
    }

    #[test]
    fn test_thread_notification() {
        let mut thread = Thread::new(
            "notify_test",
            1024,
            5,
            |thread, _param| {
                let notification = thread.wait_notification(0, 0xFFFFFFFF, Duration::from_millis(1000));
                assert!(notification.is_ok());
                Ok(None)
            }
        );

        let spawned = thread.spawn(None).unwrap();
        
        let notify_result = spawned.notify(ThreadNotification::SetValue(0x12345678));
        assert!(notify_result.is_ok());
        
        spawned.delete();
    }

    #[test]
    fn test_thread_get_current() {
        let current = Thread::get_current();
        let metadata = current.get_metadata();
        assert!(!metadata.thread.is_null());
    }
}
