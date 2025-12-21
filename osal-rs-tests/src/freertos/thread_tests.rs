extern crate alloc;

use alloc::sync::Arc;
use alloc::boxed::Box;
use core::any::Any;
use core::time::Duration;
use osal_rs::os::*;
use osal_rs::os::ThreadNotification;
use osal_rs::utils::Result;
use osal_rs::{log_debug, log_info};

const TAG: &str = "ThreadTests";

pub fn test_thread_creation() -> Result<()> {
    log_info!(TAG, "Starting test_thread_creation");
    let thread = Thread::new(
        "test_thread",
        1024,
        5,
        |_thread, _param| {
            Ok(_param.unwrap_or_else(|| Arc::new(())))
        }
    );

    let metadata = thread.get_metadata();
    log_debug!(TAG, "Thread metadata: name={}, stack={}, priority={}", metadata.name, metadata.stack_depth, metadata.priority);
    assert!(!metadata.name.is_empty());
    assert_eq!(metadata.stack_depth, 1024);
    assert_eq!(metadata.priority, 5);
    log_info!(TAG, "test_thread_creation PASSED");
    Ok(())
}

pub fn test_thread_spawn() -> Result<()> {
    log_info!(TAG, "Starting test_thread_spawn");
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
        log_debug!(TAG, "Spawned thread handle: {:?}", metadata.thread);
        assert!(!metadata.thread.is_null());
        spawned.delete();
        log_debug!(TAG, "Thread deleted successfully");
    }
    log_info!(TAG, "test_thread_spawn PASSED");
    Ok(())
}

pub fn test_thread_with_param() -> Result<()> {
    log_info!(TAG, "Starting test_thread_with_param");
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
        log_debug!(TAG, "Thread spawned with parameter");
        spawned.delete();
    }
    log_info!(TAG, "test_thread_with_param PASSED");
    Ok(())
}

pub fn test_thread_suspend_resume() -> Result<()> {
    log_info!(TAG, "Starting test_thread_suspend_resume");
    let mut thread = Thread::new(
        "suspend_test",
        1024,
        5,
        |_thread, _param| {
            Ok(_param.unwrap_or_else(|| Arc::new(())))
        }
    );

    let spawned = thread.spawn(None)?;
    log_debug!(TAG, "Suspending thread...");
    spawned.suspend();
    log_debug!(TAG, "Resuming thread...");
    spawned.resume();
    spawned.delete();
    log_info!(TAG, "test_thread_suspend_resume PASSED");
    Ok(())
}

pub fn test_thread_get_metadata() -> Result<()> {
    log_info!(TAG, "Starting test_thread_get_metadata");
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
    
    log_debug!(TAG, "Metadata - name: {}, priority: {}", metadata.name, metadata.priority);
    assert_eq!(metadata.name, "metadata_test");
    assert_eq!(metadata.priority, 5);
    
    spawned.delete();
    log_info!(TAG, "test_thread_get_metadata PASSED");
    Ok(())
}

pub fn test_thread_notification() -> Result<()> {
    log_info!(TAG, "Starting test_thread_notification");
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
    
    log_debug!(TAG, "Sending notification: 0x12345678");
    let notify_result = spawned.notify(ThreadNotification::SetValueWithOverwrite(0x12345678));
    assert!(notify_result.is_ok());
    
    spawned.delete();
    log_info!(TAG, "test_thread_notification PASSED");
    Ok(())
}

pub fn test_thread_get_current() -> Result<()> {
    log_info!(TAG, "Starting test_thread_get_current");
    let current = Thread::get_current();
    let metadata = current.get_metadata();
    log_debug!(TAG, "Current thread: {}", metadata.name);
    assert!(!metadata.thread.is_null());
    log_info!(TAG, "test_thread_get_current PASSED");
    Ok(())
}

// Thread function (not a closure)
fn thread_function(_thread: Box<dyn ThreadFn>, param: Option<Arc<dyn Any + Send + Sync>>) -> Result<Arc<dyn Any + Send + Sync>> {
    log_debug!(TAG, "Thread function executing");
    
    // Check if we received a parameter
    if let Some(p) = param.as_ref() {
        if let Some(val) = p.downcast_ref::<u32>() {
            log_debug!(TAG, "Received parameter value: {}", *val);
            assert_eq!(*val, 99);
        }
    }
    
    // Simulate some work
    System::delay(Duration::from_millis(10).to_ticks());
    
    Ok(param.unwrap_or_else(|| Arc::new(())))
}

pub fn test_thread_with_function() -> Result<()> {
    log_info!(TAG, "Starting test_thread_with_function");
    let mut thread = Thread::new(
        "function_test",
        1024,
        5,
        thread_function
    );

    let result = thread.spawn(None);
    assert!(result.is_ok());
    
    if let Ok(spawned) = result {
        log_debug!(TAG, "Thread spawned with function (no param)");
        System::delay(Duration::from_millis(20).to_ticks());
        spawned.delete();
    }
    log_info!(TAG, "test_thread_with_function PASSED");
    Ok(())
}

pub fn test_thread_with_function_and_param() -> Result<()> {
    log_info!(TAG, "Starting test_thread_with_function_and_param");
    let test_value: u32 = 99;
    let param: Arc<dyn Any + Send + Sync> = Arc::new(test_value);
    
    let mut thread = Thread::new(
        "function_param_test",
        1024,
        5,
        thread_function
    );

    let result = thread.spawn(Some(param));
    assert!(result.is_ok());
    
    if let Ok(spawned) = result {
        log_debug!(TAG, "Thread spawned with function and parameter");
        System::delay(Duration::from_millis(20).to_ticks());
        spawned.delete();
    }
    log_info!(TAG, "test_thread_with_function_and_param PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Thread Tests ==========");
    test_thread_creation()?;
    test_thread_spawn()?;
    test_thread_with_param()?;
    test_thread_suspend_resume()?;
    test_thread_get_metadata()?;
    test_thread_notification()?;
    test_thread_get_current()?;
    test_thread_with_function()?;
    test_thread_with_function_and_param()?;
    log_info!(TAG, "========== All Thread Tests PASSED ==========");
    Ok(())
}
