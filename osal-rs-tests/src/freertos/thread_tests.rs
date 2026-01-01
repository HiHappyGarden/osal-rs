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

use alloc::sync::Arc;
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
        5
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
        5
    );

    let result = thread.spawn(None, |_thread, _param| {
        Ok(_param.unwrap_or_else(|| Arc::new(())))
    });
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
        5
    );

    let result = thread.spawn(Some(param), |_thread, param| {
        if let Some(p) = param.as_ref() {
            if let Some(val) = p.downcast_ref::<u32>() {
                assert_eq!(*val, 42);
            }
        }
        Ok(param.unwrap_or_else(|| Arc::new(())))
    });
    assert!(result.is_ok());
    
    if let Ok(spawned) = result {
        log_debug!(TAG, "Thread spawned with parameter");
        System::delay(Duration::from_millis(50).to_ticks());
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
        5
    );

    let spawned = thread.spawn(None, |_thread, _param| {
        System::delay(Duration::from_millis(100).to_ticks());
        Ok(_param.unwrap_or_else(|| Arc::new(())))
    })?;
    
    log_debug!(TAG, "Suspending thread...");
    spawned.suspend();
    System::delay(Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Resuming thread...");
    spawned.resume();
    System::delay(Duration::from_millis(50).to_ticks());
    spawned.delete();
    log_info!(TAG, "test_thread_suspend_resume PASSED");
    Ok(())
}

pub fn test_thread_get_metadata() -> Result<()> {
    log_info!(TAG, "Starting test_thread_get_metadata");
    let mut thread = Thread::new(
        "metadata_test",
        1024,
        5
    );

    let spawned = thread.spawn(None, |_thread, _param| {
        System::delay(Duration::from_millis(50).to_ticks());
        Ok(_param.unwrap_or_else(|| Arc::new(())))
    })?;
    
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
        5
    );

    let spawned = thread.spawn(None, |thread, _param| {
        let notification = thread.wait_notification(0, 0xFFFFFFFF, Duration::from_millis(1000).to_ticks())?;
        log_debug!(TAG, "Received notification: 0x{:X}", notification);
        assert_eq!(notification, 0x12345678);
        Ok(Arc::new(()))
    })?;
    
    System::delay(Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Sending notification: 0x12345678");
    let notify_result = spawned.notify(ThreadNotification::SetValueWithOverwrite(0x12345678));
    assert!(notify_result.is_ok());
    
    System::delay(Duration::from_millis(50).to_ticks());
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

pub fn test_thread_spawn_simple() -> Result<()> {
    log_info!(TAG, "Starting test_thread_spawn_simple");
    let mut thread = Thread::new(
        "simple_test",
        1024,
        5
    );

    let result = thread.spawn_simple(|| {
        log_debug!(TAG, "Simple thread executing");
        System::delay(Duration::from_millis(10).to_ticks());
    });
    
    assert!(result.is_ok());
    
    if let Ok(spawned) = result {
        log_debug!(TAG, "Simple thread spawned successfully");
        System::delay(Duration::from_millis(50).to_ticks());
        spawned.delete();
    }
    log_info!(TAG, "test_thread_spawn_simple PASSED");
    Ok(())
}

pub fn test_thread_spawn_simple_with_shared_data() -> Result<()> {
    log_info!(TAG, "Starting test_thread_spawn_simple_with_shared_data");
    
    let counter = Mutex::new_arc(0u32);
    let counter_clone = Arc::clone(&counter);
    
    let mut thread = Thread::new(
        "shared_data_test",
        1024,
        5
    );

    let result = thread.spawn_simple(move || {
        for _ in 0..5 {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
            log_debug!(TAG, "Counter: {}", *num);
        }
    });
    
    assert!(result.is_ok());
    
    if let Ok(spawned) = result {
        System::delay(Duration::from_millis(100).to_ticks());
        spawned.delete();
    }
    
    let final_count = *counter.lock().unwrap();
    log_debug!(TAG, "Final counter value: {}", final_count);
    assert_eq!(final_count, 5);
    
    log_info!(TAG, "test_thread_spawn_simple_with_shared_data PASSED");
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
    test_thread_spawn_simple()?;
    test_thread_spawn_simple_with_shared_data()?;
    log_info!(TAG, "========== All Thread Tests PASSED ==========");
    Ok(())
}
