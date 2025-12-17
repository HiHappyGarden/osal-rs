#[cfg(test)]
mod tests {
    extern crate alloc;
    
    use osal_rs::os::*;
    use osal_rs::utils::{Result, OsalRsBool};
    use core::time::Duration;

    #[test]
    fn test_semaphore_creation() {
        let semaphore = Semaphore::new(5, 0);
        assert!(semaphore.is_ok());
    }

    #[test]
    fn test_semaphore_creation_with_count() {
        let semaphore = Semaphore::new_with_count(3);
        assert!(semaphore.is_ok());
    }

    #[test]
    fn test_semaphore_signal_wait() {
        let semaphore = Semaphore::new(5, 0).unwrap();
        
        let signal_result = semaphore.signal();
        assert_eq!(signal_result, OsalRsBool::True);
        
        let wait_result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(wait_result, OsalRsBool::True);
    }

    #[test]
    fn test_semaphore_wait_timeout() {
        let semaphore = Semaphore::new(5, 0).unwrap();
        
        let wait_result = semaphore.wait(Duration::from_millis(10));
        assert_eq!(wait_result, OsalRsBool::False);
    }

    #[test]
    fn test_semaphore_multiple_signals() {
        let semaphore = Semaphore::new(10, 0).unwrap();
        
        for _ in 0..5 {
            let result = semaphore.signal();
            assert_eq!(result, OsalRsBool::True);
        }
        
        for _ in 0..5 {
            let result = semaphore.wait(Duration::from_millis(100));
            assert_eq!(result, OsalRsBool::True);
        }
        
        let result = semaphore.wait(Duration::from_millis(10));
        assert_eq!(result, OsalRsBool::False);
    }

    #[test]
    fn test_semaphore_max_count() {
        let semaphore = Semaphore::new(3, 0).unwrap();
        
        for _ in 0..3 {
            semaphore.signal();
        }
        
        for _ in 0..3 {
            let result = semaphore.wait(Duration::from_millis(100));
            assert_eq!(result, OsalRsBool::True);
        }
    }

    #[test]
    fn test_semaphore_initial_count() {
        let semaphore = Semaphore::new(5, 3).unwrap();
        
        for _ in 0..3 {
            let result = semaphore.wait(Duration::from_millis(100));
            assert_eq!(result, OsalRsBool::True);
        }
        
        let result = semaphore.wait(Duration::from_millis(10));
        assert_eq!(result, OsalRsBool::False);
    }

    #[test]
    fn test_semaphore_binary() {
        let semaphore = Semaphore::new(1, 1).unwrap();
        
        let result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(result, OsalRsBool::True);
        
        let result = semaphore.wait(Duration::from_millis(10));
        assert_eq!(result, OsalRsBool::False);
        
        semaphore.signal();
        
        let result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(result, OsalRsBool::True);
    }

    #[test]
    fn test_semaphore_drop() {
        let semaphore = Semaphore::new(5, 2).unwrap();
        drop(semaphore);
    }
}
