extern crate alloc;

use osal_rs::os::*;
use osal_rs::utils::Result;

pub fn test_mutex_creation() -> Result<()> {
    let mutex = Mutex::new(0u32);
    assert!(mutex.is_ok());
    Ok(())
}

pub fn test_mutex_lock_unlock() -> Result<()> {
    let mutex = Mutex::new(42u32)?;
    
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
    Ok(())
}

pub fn test_mutex_modify_data() -> Result<()> {
    let mutex = Mutex::new(0u32)?;
    
    {
        let mut guard = mutex.lock()?;
        *guard = 100;
    }
    
    {
        let guard = mutex.lock()?;
        assert_eq!(*guard, 100);
    }
    Ok(())
}

pub fn test_mutex_multiple_locks() -> Result<()> {
    let mutex = Mutex::new(0u32)?;
    
    for i in 0..10 {
        let mut guard = mutex.lock()?;
        *guard += 1;
        assert_eq!(*guard, i + 1);
    }
    
    let guard = mutex.lock()?;
    assert_eq!(*guard, 10);
    Ok(())
}

pub fn test_mutex_guard_drop() -> Result<()> {
    let mutex = Mutex::new(42u32)?;
    
    {
        let _guard = mutex.lock()?;
    }
    
    let guard = mutex.lock();
    assert!(guard.is_ok());
    Ok(())
}

pub fn test_mutex_with_struct() -> Result<()> {
    #[derive(Debug, PartialEq)]
    struct TestData {
        value: u32,
        flag: bool,
    }
    
    let mutex = Mutex::new(TestData { value: 0, flag: false })?;
    
    {
        let mut guard = mutex.lock()?;
        guard.value = 123;
        guard.flag = true;
    }
    
    {
        let guard = mutex.lock()?;
        assert_eq!(guard.value, 123);
        assert_eq!(guard.flag, true);
    }
    Ok(())
}

pub fn test_mutex_recursive() -> Result<()> {
    let mutex = Mutex::new(0u32)?;
    
    let _guard1 = mutex.lock()?;
    let _guard2 = mutex.lock()?;
    let _guard3 = mutex.lock()?;
    Ok(())
}

pub fn test_mutex_drop() -> Result<()> {
    let mutex = Mutex::new(42u32)?;
    drop(mutex);
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    test_mutex_creation()?;
    test_mutex_lock_unlock()?;
    test_mutex_modify_data()?;
    test_mutex_multiple_locks()?;
    test_mutex_guard_drop()?;
    test_mutex_with_struct()?;
    test_mutex_recursive()?;
    test_mutex_drop()?;
    Ok(())
}
