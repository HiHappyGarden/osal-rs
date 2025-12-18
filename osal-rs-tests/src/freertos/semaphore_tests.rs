extern crate alloc;

use osal_rs::os::*;
use osal_rs::utils::{Result, OsalRsBool};
use core::time::Duration;

pub fn test_semaphore_creation() -> Result<()> {
    let semaphore = Semaphore::new(5, 0);
    assert!(semaphore.is_ok());
    Ok(())
}

pub fn test_semaphore_creation_with_count() -> Result<()> {
    let semaphore = Semaphore::new_with_count(3);
    assert!(semaphore.is_ok());
    Ok(())
}

pub fn test_semaphore_signal_wait() -> Result<()> {
    let semaphore = Semaphore::new(5, 0)?;
    
    let signal_result = semaphore.signal();
    assert_eq!(signal_result, OsalRsBool::True);
    
    let wait_result = semaphore.wait(Duration::from_millis(100));
    assert_eq!(wait_result, OsalRsBool::True);
    Ok(())
}

pub fn test_semaphore_wait_timeout() -> Result<()> {
    let semaphore = Semaphore::new(5, 0)?;
    
    let wait_result = semaphore.wait(Duration::from_millis(10));
    assert_eq!(wait_result, OsalRsBool::False);
    Ok(())
}

pub fn test_semaphore_multiple_signals() -> Result<()> {
    let semaphore = Semaphore::new(10, 0)?;
    
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
    Ok(())
}

pub fn test_semaphore_max_count() -> Result<()> {
    let semaphore = Semaphore::new(3, 0)?;
    
    for _ in 0..3 {
        semaphore.signal();
    }
    
    for _ in 0..3 {
        let result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(result, OsalRsBool::True);
    }
    Ok(())
}

pub fn test_semaphore_initial_count() -> Result<()> {
    let semaphore = Semaphore::new(5, 3)?;
    
    for _ in 0..3 {
        let result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(result, OsalRsBool::True);
    }
    
    let result = semaphore.wait(Duration::from_millis(10));
    assert_eq!(result, OsalRsBool::False);
    Ok(())
}

pub fn test_semaphore_binary() -> Result<()> {
    let semaphore = Semaphore::new(1, 1)?;
    
    let result = semaphore.wait(Duration::from_millis(100));
    assert_eq!(result, OsalRsBool::True);
    
    let result = semaphore.wait(Duration::from_millis(10));
    assert_eq!(result, OsalRsBool::False);
    
    semaphore.signal();
    
    let result = semaphore.wait(Duration::from_millis(100));
    assert_eq!(result, OsalRsBool::True);
    Ok(())
}

pub fn test_semaphore_drop() -> Result<()> {
    let semaphore = Semaphore::new(5, 2)?;
    drop(semaphore);
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    test_semaphore_creation()?;
    test_semaphore_creation_with_count()?;
    test_semaphore_signal_wait()?;
    test_semaphore_wait_timeout()?;
    test_semaphore_multiple_signals()?;
    test_semaphore_max_count()?;
    test_semaphore_initial_count()?;
    test_semaphore_binary()?;
    test_semaphore_drop()?;
    Ok(())
}
