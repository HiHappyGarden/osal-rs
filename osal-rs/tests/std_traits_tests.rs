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

//! Tests for the backend-independent items in `osal_rs::traits`: the generic
//! [`RawMutexGuard`] RAII wrapper, the `MutexGuardFn::update` implementations,
//! the `BytesHasLen` blanket impl for arrays, and the plain-data helpers
//! (`ThreadNotification` -> `(u32, u32)`, `ThreadMetadata::default`).
//!
//! These exercise trait-level code that has no backend-specific body, so the
//! same content is mirrored in `osal-rs-tests/src/freertos/traits_tests.rs`.
//! Gated on `posix` to match this crate's convention of running its `tests/`
//! suite against the POSIX backend.

#![cfg(feature = "posix")]

use osal_rs::os::*;
use osal_rs::utils::{OsalRsBool, Result};
use osal_rs::{log_debug, log_info};

const TAG: &str = "TraitsTests";

/// Leaks a fresh `RawMutex` so it can be handed to `RawMutexGuard`, which
/// requires a `&'static` reference (it is designed for module-level statics).
fn leaked_raw_mutex() -> &'static RawMutex {
    Box::leak(Box::new(RawMutex::new().expect("RawMutex::new")))
}

#[test]
fn test_raw_mutex_guard_acquire() -> Result<()> {
    log_info!(TAG, "Starting test_raw_mutex_guard_acquire");

    let mutex = leaked_raw_mutex();

    {
        let _guard = RawMutexGuard::acquire(mutex);
        // Held by this thread; the recursive pthread mutex lets the same
        // thread observe it as lockable, so assert on the unlock side below.
        log_debug!(TAG, "inside RawMutexGuard::acquire critical section");
    }

    // After the guard dropped, the mutex is free again: a fresh lock/unlock
    // round-trip must still succeed.
    assert_eq!(mutex.lock(), OsalRsBool::True);
    assert_eq!(mutex.unlock(), OsalRsBool::True);

    log_info!(TAG, "test_raw_mutex_guard_acquire PASSED");
    Ok(())
}

#[test]
fn test_raw_mutex_guard_acquire_from_isr() -> Result<()> {
    log_info!(TAG, "Starting test_raw_mutex_guard_acquire_from_isr");

    let mutex = leaked_raw_mutex();

    {
        let _guard = RawMutexGuard::acquire_from_isr(mutex);
        log_debug!(TAG, "inside RawMutexGuard::acquire_from_isr critical section");
    }

    // The ISR variant must unlock through `unlock_from_isr` on drop, leaving
    // the mutex acquirable again.
    assert_eq!(mutex.lock_from_isr(), OsalRsBool::True);
    assert_eq!(mutex.unlock_from_isr(), OsalRsBool::True);

    log_info!(TAG, "test_raw_mutex_guard_acquire_from_isr PASSED");
    Ok(())
}

#[test]
fn test_raw_mutex_guard_nested() -> Result<()> {
    log_info!(TAG, "Starting test_raw_mutex_guard_nested");

    // The backing pthread mutex is recursive, so nesting two guards on the
    // same thread is legal and each drop unlocks exactly one level.
    let mutex = leaked_raw_mutex();

    {
        let _outer = RawMutexGuard::acquire(mutex);
        {
            let _inner = RawMutexGuard::acquire(mutex);
            log_debug!(TAG, "nested RawMutexGuard depth 2");
        }
        log_debug!(TAG, "nested RawMutexGuard depth 1");
    }

    assert_eq!(mutex.lock(), OsalRsBool::True);
    assert_eq!(mutex.unlock(), OsalRsBool::True);

    log_info!(TAG, "test_raw_mutex_guard_nested PASSED");
    Ok(())
}

#[test]
fn test_mutex_guard_update() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_guard_update");

    let mutex = Mutex::new(0u32);

    {
        let mut guard = mutex.lock()?;
        guard.update(&42);
        assert_eq!(*guard, 42);
    }

    {
        let mut guard = mutex.lock_from_isr()?;
        guard.update(&7);
        assert_eq!(*guard, 7);
    }

    // `lock_from_isr_explicit` returns the same guard type, so `update` must
    // behave identically through it.
    {
        let mut guard = mutex.lock_from_isr_explicit()?;
        guard.update(&99);
        assert_eq!(*guard, 99);
    }

    assert_eq!(*mutex.lock()?, 99);

    log_info!(TAG, "test_mutex_guard_update PASSED");
    Ok(())
}

#[test]
fn test_mutex_guard_update_non_copy() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_guard_update_non_copy");

    // `update` clones, so it must also work for heap-allocated payloads.
    let mutex = Mutex::new(String::from("initial"));

    {
        let mut guard = mutex.lock()?;
        guard.update(&String::from("updated"));
        assert_eq!(guard.as_str(), "updated");
    }

    let taken = mutex.into_inner()?;
    log_debug!(TAG, "into_inner: {}", taken);
    assert_eq!(taken, "updated");

    log_info!(TAG, "test_mutex_guard_update_non_copy PASSED");
    Ok(())
}

#[test]
fn test_bytes_has_len_provided_method() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_has_len_provided_method");

    // `is_empty` is a provided method defined purely in terms of `len`, so an
    // implementor only has to supply the latter.
    struct Payload(usize);

    impl BytesHasLen for Payload {
        fn len(&self) -> usize {
            self.0
        }
    }

    assert_eq!(Payload(4).len(), 4);
    assert!(!Payload(4).is_empty());
    assert_eq!(Payload(0).len(), 0);
    assert!(Payload(0).is_empty());

    log_info!(TAG, "test_bytes_has_len_provided_method PASSED");
    Ok(())
}

/// The blanket impl is `impl<T: Serialize, const N: usize> BytesHasLen for
/// [T; N]`, and plain `u8` only satisfies `Serialize` through `osal-rs-serde`
/// - so the array form only exists with the `serde` feature on.
#[cfg(feature = "serde")]
#[test]
fn test_bytes_has_len_blanket_impl() -> Result<()> {
    log_info!(TAG, "Starting test_bytes_has_len_blanket_impl");

    // Length is the const generic, independent of the contents.
    let four = [1u8, 2, 3, 4];
    assert_eq!(BytesHasLen::len(&four), 4);
    assert!(!BytesHasLen::is_empty(&four));

    let none: [u8; 0] = [];
    assert_eq!(BytesHasLen::len(&none), 0);
    assert!(BytesHasLen::is_empty(&none));

    log_info!(TAG, "test_bytes_has_len_blanket_impl PASSED");
    Ok(())
}

#[test]
fn test_thread_notification_into_tuple() -> Result<()> {
    log_info!(TAG, "Starting test_thread_notification_into_tuple");

    // Every variant maps to the (action, value) pair the backends pass down
    // to the OS notify call.
    let cases: [(ThreadNotification, (u32, u32)); 5] = [
        (ThreadNotification::NoAction, (0, 0)),
        (ThreadNotification::SetBits(0b1011), (1, 0b1011)),
        (ThreadNotification::Increment, (2, 0)),
        (ThreadNotification::SetValueWithOverwrite(0xDEAD), (3, 0xDEAD)),
        (ThreadNotification::SetValueWithoutOverwrite(0xBEEF), (4, 0xBEEF)),
    ];

    for (notification, expected) in cases {
        let actual: (u32, u32) = notification.into();
        log_debug!(TAG, "{:?} -> {:?}", notification, actual);
        assert_eq!(actual, expected);
    }

    log_info!(TAG, "test_thread_notification_into_tuple PASSED");
    Ok(())
}

#[test]
fn test_thread_metadata_default() -> Result<()> {
    log_info!(TAG, "Starting test_thread_metadata_default");

    let metadata = ThreadMetadata::default();
    log_debug!(TAG, "default metadata: {:?}", metadata);

    assert_eq!(metadata.name.len(), 0);
    assert_eq!(metadata.stack_depth, 0);
    assert_eq!(metadata.priority, 0);
    assert_eq!(metadata.thread_number, 0);
    assert_eq!(metadata.state, ThreadState::Invalid);
    assert_eq!(metadata.current_priority, 0);
    assert_eq!(metadata.base_priority, 0);
    assert_eq!(metadata.run_time_counter, 0);
    assert_eq!(metadata.stack_high_water_mark, 0);

    // `Clone` must produce an equivalent value.
    let cloned = metadata.clone();
    assert_eq!(cloned.state, metadata.state);
    assert_eq!(cloned.name.len(), metadata.name.len());

    log_info!(TAG, "test_thread_metadata_default PASSED");
    Ok(())
}

#[test]
fn test_thread_state_variants() -> Result<()> {
    log_info!(TAG, "Starting test_thread_state_variants");

    // `ThreadState` is compared by value throughout `System::suspend_all` /
    // `resume_all`, so equality and `Debug` must both behave.
    let states = [
        ThreadState::Running,
        ThreadState::Ready,
        ThreadState::Blocked,
        ThreadState::Suspended,
        ThreadState::Deleted,
        ThreadState::Invalid,
    ];

    for (i, state) in states.iter().enumerate() {
        log_debug!(TAG, "state[{}] = {:?}", i, state);
        assert_eq!(*state, states[i]);
        for (j, other) in states.iter().enumerate() {
            if i != j {
                assert_ne!(*state, *other);
            }
        }
    }

    log_info!(TAG, "test_thread_state_variants PASSED");
    Ok(())
}

#[test]
fn test_to_priority_custom_impl() -> Result<()> {
    log_info!(TAG, "Starting test_to_priority_custom_impl");

    // `ToPriority` is the extension point for application-defined priority
    // enums; `Thread::new_with_to_priority` accepts anything implementing it.
    #[derive(Clone, Copy)]
    enum TaskPriority {
        Low,
        High,
    }

    impl ToPriority for TaskPriority {
        fn to_priority(&self) -> types::UBaseType {
            match self {
                TaskPriority::Low => 1,
                TaskPriority::High => 9,
            }
        }
    }

    assert_eq!(TaskPriority::Low.to_priority(), 1);
    assert_eq!(TaskPriority::High.to_priority(), 9);

    let thread = Thread::new_with_to_priority("prio", 2048, TaskPriority::High);
    log_debug!(TAG, "thread built from ToPriority: {}", thread);

    log_info!(TAG, "test_to_priority_custom_impl PASSED");
    Ok(())
}
