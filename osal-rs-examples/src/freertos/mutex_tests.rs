#[cfg(test)]
mod tests {
    extern crate alloc;
    
    use osal_rs::os::*;
    use osal_rs::utils::Result;

    #[test]
    fn test_mutex_creation() {
        let mutex = Mutex::new(0u32);
        assert!(mutex.is_ok());
    }

    #[test]
    fn test_mutex_lock_unlock() {
        let mutex = Mutex::new(42u32).unwrap();
        
        {
            let guard = mutex.lock();
            assert!(guard.is_ok());
            
            if let Ok(g) = guard {
                assert_eq!(*g, 42);
            }
        }
        
        {
            let guard = mutex.lock();
            assert!(guard.is_ok());
        }
    }

    #[test]
    fn test_mutex_modify_data() {
        let mutex = Mutex::new(0u32).unwrap();
        
        {
            let mut guard = mutex.lock().unwrap();
            *guard = 100;
        }
        
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(*guard, 100);
        }
    }

    #[test]
    fn test_mutex_multiple_locks() {
        let mutex = Mutex::new(0u32).unwrap();
        
        for i in 0..10 {
            let mut guard = mutex.lock().unwrap();
            *guard += 1;
            assert_eq!(*guard, i + 1);
        }
        
        let guard = mutex.lock().unwrap();
        assert_eq!(*guard, 10);
    }

    #[test]
    fn test_mutex_guard_drop() {
        let mutex = Mutex::new(42u32).unwrap();
        
        {
            let _guard = mutex.lock().unwrap();
        }
        
        let guard = mutex.lock();
        assert!(guard.is_ok());
    }

    #[test]
    fn test_mutex_with_struct() {
        #[derive(Debug, PartialEq)]
        struct TestData {
            value: u32,
            flag: bool,
        }
        
        let mutex = Mutex::new(TestData { value: 0, flag: false }).unwrap();
        
        {
            let mut guard = mutex.lock().unwrap();
            guard.value = 123;
            guard.flag = true;
        }
        
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(guard.value, 123);
            assert_eq!(guard.flag, true);
        }
    }

    #[test]
    fn test_mutex_recursive() {
        let mutex = Mutex::new(0u32).unwrap();
        
        let _guard1 = mutex.lock().unwrap();
        let _guard2 = mutex.lock().unwrap();
        let _guard3 = mutex.lock().unwrap();
    }

    #[test]
    fn test_mutex_drop() {
        let mutex = Mutex::new(42u32).unwrap();
        drop(mutex);
    }
}
