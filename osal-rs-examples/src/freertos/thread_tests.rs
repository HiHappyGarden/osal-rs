extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::any::Any;
use core::time::Duration;
use osal_rs::os::*;
use osal_rs::os::ThreadNotification;
use osal_rs::utils::Result;

pub fn test_thread_creation() -> Result<()> {
    let thread = Thread::new(
        "test_thread",
        1024,
        5,
        |_thread, _param| {
            Ok(_param.unwrap_or_else(|| Arc::new(())))
        }
    );

    let metadata = thread.get_metadata();
    assert!(!metadata.name.is_empty());
    assert_eq!(metadata.stack_depth, 1024);
    assert_eq!(metadata.priority, 5);
    Ok(())
}

pub fn test_thread_spawn() -> Result<()> {
    let mut thread = Thread::new(
        "spawn_test",
        1024,
        5,
        |_thread, _param| {
            Ok(_param.unwrap_or_else(|| Arc::new(())))
        }
    );

    let result = thread.spawn(None);
    assert!(result.is_ok());
    
    if let Ok(spawned) = result {
        let metadata = spawned.get_metadata();
        assert!(!metadata.thread.is_null());
        spawned.delete();
    }
    Ok(())
}

pub fn test_thread_with_param() -> Result<()> {
    let test_value: u32 = 42;
    let param: Arc<dyn Any + Send + Sync> = Arc::new(test_value);
    
    let mut thread = Thread::new(
        "param_test",
        1024,
        5,
        |_thread, param| {
            if let Some(p) = param.as_ref() {
                if let Some(val) = p.downcast_ref::<u32>() {
                    assert_eq!(*val, 42);
                }
            }
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    );

    let result = thread.spawn(Some(param));
    assert!(result.is_ok());
    
    if let Ok(spawned) = result {
        spawned.delete();
    }
    Ok(())
}

pub fn test_thread_suspend_resume() -> Result<()> {
    let mut thread = Thread::new(
        "suspend_test",
        1024,
        5,
        |_thread, _param| {
            Ok(_param.unwrap_or_else(|| Arc::new(())))
        }
    );

    let spawned = thread.spawn(None)?;
    spawned.suspend();
    spawned.resume();
    spawned.delete();
    Ok(())
}

pub fn test_thread_get_metadata() -> Result<()> {
    let mut thread = Thread::new(
        "metadata_test",
        1024,
        5,
        |_thread, _param| {
            Ok(_param.unwrap_or_else(|| Arc::new(())))
        }
    );

    let spawned = thread.spawn(None)?;
    let metadata = spawned.get_metadata();
    
    assert_eq!(metadata.name, "metadata_test");
    assert_eq!(metadata.priority, 5);
    
    spawned.delete();
    Ok(())
}

pub fn test_thread_notification() -> Result<()> {
    let mut thread = Thread::new(
        "notify_test",
        1024,
        5,
        |thread, _param| {
            let _notification = thread.wait_notification(0, 0xFFFFFFFF, Duration::from_millis(1000).to_ticks());
            Ok(Arc::new(()))
        }
    );

    let spawned = thread.spawn(None)?;
    
    let notify_result = spawned.notify(ThreadNotification::SetValueWithOverwrite(0x12345678));
    assert!(notify_result.is_ok());
    
    spawned.delete();
    Ok(())
}

pub fn test_thread_get_current() -> Result<()> {
    let current = Thread::get_current();
    let metadata = current.get_metadata();
    assert!(!metadata.thread.is_null());
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    test_thread_creation()?;
    test_thread_spawn()?;
    test_thread_with_param()?;
    test_thread_suspend_resume()?;
    test_thread_get_metadata()?;
    test_thread_notification()?;
    test_thread_get_current()?;
    Ok(())
}
