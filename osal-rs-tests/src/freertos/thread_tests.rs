/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

extern crate alloc;

use alloc::sync::Arc;
use core::any::Any;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;
use osal_rs::os::*;
use osal_rs::os::ThreadNotification;
use osal_rs::utils::Result;
use osal_rs::{log_debug, log_info};

const TAG: &str = "ThreadTests";

struct FixedPriority(types::UBaseType);

impl ToPriority for FixedPriority {
    fn to_priority(&self) -> types::UBaseType {
        self.0
    }
}

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
    assert_eq!(metadata.name.as_str(), "metadata_test");
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

/// Exercises real two-thread synchronization via `ThreadFn::notify` /
/// `ThreadFn::wait_notification`, in both directions:
///
/// 1. The worker thread blocks on `wait_notification` until the main thread
///    wakes it up with a specific value.
/// 2. The worker stores the received value, then notifies the main thread
///    back (rendezvous) with a different value.
/// 3. The main thread blocks on its own `wait_notification` until the
///    worker's reply arrives, and both values are checked.
///
/// This is a stronger check than `test_thread_notification`, which only
/// verifies a one-way, fire-and-forget notification.
pub fn test_thread_two_thread_notification_sync() -> Result<()> {
    log_info!(TAG, "Starting test_thread_two_thread_notification_sync");
    static WORKER_RECEIVED: AtomicU32 = AtomicU32::new(0);

    let main_thread = Thread::get_current();

    let mut thread = Thread::new(
        "sync_worker",
        1024,
        5
    );

    let spawned = thread.spawn(None, move |thread, param| {
        // Blocks until the main thread wakes us up with a value.
        let value = thread.wait_notification(0, 0xFFFFFFFF, Duration::from_millis(1000).to_ticks())?;
        WORKER_RECEIVED.store(value, Ordering::SeqCst);

        // Rendezvous: tell the main thread we're done processing.
        main_thread.notify(ThreadNotification::SetValueWithOverwrite(0xA5A5))?;
        Ok(param.unwrap_or_else(|| Arc::new(())))
    })?;

    log_debug!(TAG, "Waking worker with 0xC0FFEE");
    spawned.notify(ThreadNotification::SetValueWithOverwrite(0xC0FFEE))?;

    log_debug!(TAG, "Waiting for worker's rendezvous reply");
    let reply = Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(1000).to_ticks())?;
    log_debug!(TAG, "Reply: 0x{:X}, worker received: 0x{:X}", reply, WORKER_RECEIVED.load(Ordering::SeqCst));
    assert_eq!(reply, 0xA5A5);
    assert_eq!(WORKER_RECEIVED.load(Ordering::SeqCst), 0xC0FFEE);

    spawned.delete();
    log_info!(TAG, "test_thread_two_thread_notification_sync PASSED");
    Ok(())
}

pub fn test_thread_new_with_to_priority() -> Result<()> {
    log_info!(TAG, "Starting test_thread_new_with_to_priority");
    let mut thread = Thread::new_with_to_priority("priority_test", 1024, FixedPriority(3));

    let spawned = thread.spawn_simple(|| {
        System::delay(Duration::from_millis(50).to_ticks());
    })?;

    let metadata = spawned.get_metadata();
    log_debug!(TAG, "Spawned priority: {}", metadata.priority);
    assert_eq!(metadata.priority, 3);

    spawned.delete();
    log_info!(TAG, "test_thread_new_with_to_priority PASSED");
    Ok(())
}

/// Exercises `new_with_handle_and_to_priority` and `get_metadata_from_handle`
/// against a *real* handle obtained from an actually-spawned thread, rather
/// than a fabricated pointer — passing a bogus non-null handle to
/// `vTaskGetInfo` would dereference invalid memory on real hardware.
pub fn test_thread_handle_based_constructors() -> Result<()> {
    log_info!(TAG, "Starting test_thread_handle_based_constructors");

    let mut source = Thread::new("handle_source", 1024, 2);
    let spawned = source.spawn_simple(|| {
        System::delay(Duration::from_millis(50).to_ticks());
    })?;
    let handle = *spawned;

    let wrapped = Thread::new_with_handle_and_to_priority(handle, "wrapped", 1024, FixedPriority(6))?;
    assert_eq!(*wrapped, handle);

    let null_handle: types::ThreadHandle = core::ptr::null();
    let null_result = Thread::new_with_handle_and_to_priority(null_handle, "invalid", 128, FixedPriority(1));
    assert!(null_result.is_err());

    let metadata = Thread::get_metadata_from_handle(handle);
    log_debug!(TAG, "Looked-up metadata for real handle: thread_null={}", metadata.thread.is_null());
    assert!(!metadata.thread.is_null());

    spawned.delete();
    log_info!(TAG, "test_thread_handle_based_constructors PASSED");
    Ok(())
}

pub fn test_thread_notify_from_isr_and_wait_with_to_tick() -> Result<()> {
    log_info!(TAG, "Starting test_thread_notify_from_isr_and_wait_with_to_tick");
    static WORKER_RECEIVED: AtomicU32 = AtomicU32::new(0);

    let main_thread = Thread::get_current();

    let mut thread = Thread::new("isr_notify_worker", 1024, 5);
    let spawned = thread.spawn(None, move |_thread, param| {
        // `wait_notification_with_to_tick` is an inherent method on the
        // concrete `Thread`, not part of the `ThreadFn` trait object passed
        // in here, so fetch a concrete handle via `Thread::get_current()`.
        let value = Thread::get_current().wait_notification_with_to_tick(0, 0xFFFFFFFF, Duration::from_millis(1000))?;
        WORKER_RECEIVED.store(value, Ordering::SeqCst);
        main_thread.notify(ThreadNotification::SetValueWithOverwrite(1))?;
        Ok(param.unwrap_or_else(|| Arc::new(())))
    })?;

    let mut higher_priority_task_woken: types::BaseType = 0;
    let notify_result = spawned.notify_from_isr(ThreadNotification::SetValueWithOverwrite(0x99), &mut higher_priority_task_woken);
    log_debug!(TAG, "notify_from_isr ok: {}", notify_result.is_ok());
    assert!(notify_result.is_ok());

    let reply = Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(1000).to_ticks())?;
    assert_eq!(reply, 1);
    assert_eq!(WORKER_RECEIVED.load(Ordering::SeqCst), 0x99);

    spawned.delete();
    log_info!(TAG, "test_thread_notify_from_isr_and_wait_with_to_tick PASSED");
    Ok(())
}

/// `ThreadFn::join` deletes the thread as part of joining it (see
/// `freertos/thread.rs`), so unlike the other lifecycle tests this one must
/// NOT call `delete()` afterward — that would be a double-delete on an
/// already-invalid handle.
pub fn test_thread_join() -> Result<()> {
    log_info!(TAG, "Starting test_thread_join");
    let mut thread = Thread::new("join_test", 1024, 5);
    let spawned = thread.spawn_simple(|| {
        System::delay(Duration::from_millis(10).to_ticks());
    })?;

    let join_result = spawned.join(core::ptr::null_mut());
    log_debug!(TAG, "join result: {:?}", join_result);
    assert!(join_result.is_ok());

    log_info!(TAG, "test_thread_join PASSED");
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
    test_thread_two_thread_notification_sync()?;
    test_thread_new_with_to_priority()?;
    test_thread_handle_based_constructors()?;
    test_thread_notify_from_isr_and_wait_with_to_tick()?;
    test_thread_join()?;
    test_thread_get_current()?;
    test_thread_spawn_simple()?;
    test_thread_spawn_simple_with_shared_data()?;
    log_info!(TAG, "========== All Thread Tests PASSED ==========");
    Ok(())
}
