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

//! Failure-path tests: the "already deleted" guards, double-`delete()`
//! no-ops, rejected dimensions and buffer sizes, unbounded (`MAX_DELAY`)
//! waits, notification conflict/clear semantics, a timer callback that fails,
//! and `QueueStreamed`'s (de)serialization errors.
//!
//! Mirrors `osal-rs/tests/std_error_paths_tests.rs`; keep the two in sync.
//! The backends deliberately agree on all of the above - the FreeRTOS side
//! grew the same `is_null()` guards, idempotent `delete()`, dimension checks
//! and slice bounds-checks the POSIX side already had, so these assertions
//! are identical on both.
//!
//! Three POSIX tests still have no counterpart here, for reasons that are
//! about the platform rather than the wrapper:
//!
//! * The `*_deadline_crosses_second_boundary` tests exist purely to drive the
//!   `timespec` normalisation in the POSIX backend's *absolute* deadline
//!   arithmetic. FreeRTOS takes relative tick counts and has no such code.
//! * `test_timer_clone_after_original_deleted`. `posix::Timer` shares one
//!   `Arc<TimerShared>` between clones, so a clone observes the original's
//!   `delete()`; `freertos::Timer` clones copy the raw handle, so they cannot.
//! * The contended `lock_from_isr` tests. FreeRTOS's recursive mutexes must
//!   not be taken from ISR context at all, so "contended `trylock`" has no
//!   meaningful FreeRTOS counterpart.

extern crate alloc;

use core::ptr::null_mut;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

use alloc::sync::Arc;

use osal_rs::os::*;
use osal_rs::utils::{Error, OsalRsBool, Result, MAX_DELAY};
use osal_rs::{log_debug, log_info};

const TAG: &str = "ErrorPathTests";

/// Head start given to a helper task before the main task blocks.
const HEAD_START_MS: u64 = 40;

fn millis(ms: u64) -> types::TickType {
    Duration::from_millis(ms).to_ticks()
}

/// Drops any notification left pending on the calling task.
///
/// Unlike the POSIX suite - where every `#[test]` gets its own thread, and so
/// its own notification slot - the whole FreeRTOS suite runs sequentially on
/// one task. Earlier tests (`timer_tests`, `thread_tests`) leave notifications
/// behind, so the notification tests below start from a known-clean slot.
fn drain_pending_notification() {
    let _ = Thread::get_current().wait_notification(0, u32::MAX, 0);
}

// ---------------------------------------------------------------------------
// "already deleted" guards
// ---------------------------------------------------------------------------

pub fn test_raw_mutex_after_delete() -> Result<()> {
    log_info!(TAG, "Starting test_raw_mutex_after_delete");

    let mut mutex = RawMutex::new()?;
    assert!(!mutex.is_null());

    mutex.delete();
    assert!(mutex.is_null());

    // Every operation must refuse rather than hand a NULL handle to FreeRTOS.
    assert_eq!(mutex.lock(), OsalRsBool::False);
    assert_eq!(mutex.lock_from_isr(), OsalRsBool::False);
    assert_eq!(mutex.unlock(), OsalRsBool::False);
    assert_eq!(mutex.unlock_from_isr(), OsalRsBool::False);

    // Deleting twice is a no-op, including via `Drop` when this goes out of
    // scope.
    mutex.delete();
    assert!(mutex.is_null());

    log_info!(TAG, "test_raw_mutex_after_delete PASSED");
    Ok(())
}

pub fn test_semaphore_after_delete() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_after_delete");

    let mut sem = Semaphore::new(2, 1)?;
    assert!(!sem.is_null());

    sem.delete();
    assert!(sem.is_null());

    assert_eq!(sem.wait(Duration::from_millis(1)), OsalRsBool::False);
    assert_eq!(sem.wait_from_isr(), OsalRsBool::False);
    assert_eq!(sem.signal(), OsalRsBool::False);
    assert_eq!(sem.signal_from_isr(), OsalRsBool::False);

    sem.delete();
    assert!(sem.is_null());

    log_info!(TAG, "test_semaphore_after_delete PASSED");
    Ok(())
}

pub fn test_event_group_after_delete() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_after_delete");

    let mut events = EventGroup::new()?;
    events.set(0b101);
    assert!(!events.is_null());

    events.delete();
    assert!(events.is_null());

    // Reads report "no bits", writes report the null handle.
    assert_eq!(events.get(), 0);
    assert_eq!(events.get_from_isr(), 0);
    assert_eq!(events.set(0b1), 0);
    assert_eq!(events.clear(0b1), 0);
    assert_eq!(events.wait(0b1, false, 1), 0);
    assert_eq!(events.wait_with_to_tick(0b1, false, Duration::from_millis(1)), 0);
    assert!(matches!(events.set_from_isr(0b1), Err(Error::NullPtr)));
    assert!(matches!(events.clear_from_isr(0b1), Err(Error::NullPtr)));

    events.delete();
    assert!(events.is_null());

    log_info!(TAG, "test_event_group_after_delete PASSED");
    Ok(())
}

pub fn test_queue_after_delete() -> Result<()> {
    log_info!(TAG, "Starting test_queue_after_delete");

    let mut queue = Queue::new(2, 4)?;
    queue.post(&[1u8, 2, 3, 4], 0)?;
    assert!(!queue.is_null());

    queue.delete();
    assert!(queue.is_null());

    let mut buffer = [0u8; 4];
    assert!(matches!(queue.fetch(&mut buffer, 0), Err(Error::NullPtr)));
    assert!(matches!(queue.fetch_from_isr(&mut buffer), Err(Error::NullPtr)));
    assert!(matches!(queue.post(&[1u8, 2, 3, 4], 0), Err(Error::NullPtr)));
    assert!(matches!(
        queue.post_from_isr(&[1u8, 2, 3, 4]),
        Err(Error::NullPtr)
    ));
    assert!(matches!(
        queue.fetch_with_to_tick(&mut buffer, Duration::from_millis(1)),
        Err(Error::NullPtr)
    ));
    assert!(matches!(
        queue.post_with_to_tick(&[1u8, 2, 3, 4], Duration::from_millis(1)),
        Err(Error::NullPtr)
    ));

    queue.delete();
    assert!(queue.is_null());

    log_info!(TAG, "test_queue_after_delete PASSED");
    Ok(())
}

pub fn test_timer_after_delete() -> Result<()> {
    log_info!(TAG, "Starting test_timer_after_delete");

    let mut timer = Timer::new("dead-timer", millis(20), true, None, |_, param| {
        Ok(param.unwrap_or(Arc::new(())))
    })?;
    assert!(!timer.is_null());

    assert_eq!(timer.delete(0), OsalRsBool::True);
    assert!(timer.is_null());

    // With the handle cleared, every operation short-circuits to False rather
    // than asking the timer daemon to act on a deleted timer.
    assert_eq!(timer.start(0), OsalRsBool::False);
    assert_eq!(timer.stop(0), OsalRsBool::False);
    assert_eq!(timer.reset(0), OsalRsBool::False);
    assert_eq!(timer.change_period(millis(50), 0), OsalRsBool::False);
    assert_eq!(timer.delete(0), OsalRsBool::False);

    // The `_with_to_tick` wrappers forward to the same guards.
    assert_eq!(timer.start_with_to_tick(Duration::from_millis(1)), OsalRsBool::False);
    assert_eq!(timer.stop_with_to_tick(Duration::from_millis(1)), OsalRsBool::False);
    assert_eq!(timer.reset_with_to_tick(Duration::from_millis(1)), OsalRsBool::False);
    assert_eq!(
        timer.change_period_with_to_tick(Duration::from_millis(50), Duration::from_millis(1)),
        OsalRsBool::False
    );
    assert_eq!(timer.delete_with_to_tick(Duration::from_millis(1)), OsalRsBool::False);

    log_info!(TAG, "test_timer_after_delete PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// rejected dimensions and buffer sizes
// ---------------------------------------------------------------------------

pub fn test_queue_invalid_construction() -> Result<()> {
    log_info!(TAG, "Starting test_queue_invalid_construction");

    // Neither dimension may be zero.
    assert!(matches!(Queue::new(0, 4), Err(Error::InvalidQueueSize)));
    assert!(matches!(Queue::new(4, 0), Err(Error::InvalidQueueSize)));
    assert!(matches!(Queue::new(0, 0), Err(Error::InvalidQueueSize)));
    assert!(Queue::new(1, 1).is_ok());

    log_info!(TAG, "test_queue_invalid_construction PASSED");
    Ok(())
}

pub fn test_queue_undersized_buffers() -> Result<()> {
    log_info!(TAG, "Starting test_queue_undersized_buffers");

    let queue = Queue::new(2, 4)?;
    let mut too_small = [0u8; 2];

    // Every entry point validates against the queue's item size before
    // handing the pointer to FreeRTOS, which would otherwise copy four bytes
    // into (or out of) a two-byte slice.
    assert!(matches!(
        queue.fetch(&mut too_small, 0),
        Err(Error::InvalidQueueSize)
    ));
    assert!(matches!(
        queue.fetch_from_isr(&mut too_small),
        Err(Error::InvalidQueueSize)
    ));
    assert!(matches!(
        queue.post(&[1u8, 2], 0),
        Err(Error::InvalidQueueSize)
    ));
    assert!(matches!(
        queue.post_from_isr(&[1u8, 2]),
        Err(Error::InvalidQueueSize)
    ));

    log_info!(TAG, "test_queue_undersized_buffers PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// Mutex
// ---------------------------------------------------------------------------

pub fn test_mutex_accessors_without_locking() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_accessors_without_locking");

    // `get_mut` needs no lock: `&mut Mutex<T>` already proves exclusivity.
    let mut mutex = Mutex::new(0u32);
    *mutex.get_mut() += 4;
    assert_eq!(*mutex.lock()?, 4);

    // `into_inner` consumes the mutex and hands the value back.
    let inner = mutex.into_inner()?;
    assert_eq!(inner, 4);

    // `new_arc` is the shared-ownership shortcut.
    let shared = Mutex::new_arc(0u32);
    let clone = shared.clone();
    *clone.lock()? += 1;
    assert_eq!(*shared.lock()?, 1);

    // `Debug`/`Display` must not deadlock by trying to read the payload.
    log_debug!(TAG, "Mutex debug: {:?} display: {}", shared, shared);

    log_info!(TAG, "test_mutex_accessors_without_locking PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// Semaphore
// ---------------------------------------------------------------------------

pub fn test_semaphore_blocking_wait_forever() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_blocking_wait_forever");

    // `MAX_DELAY` converts to `TickType::MAX`, the "wait forever" sentinel.
    assert_eq!(MAX_DELAY.to_ticks(), types::TickType::MAX);

    let sem = Arc::new(Semaphore::new(1, 0)?);

    let signaller_sem = sem.clone();

    let mut signaller = Thread::new("sem-signaller", 2048, 5);
    let spawned = signaller.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        signaller_sem.signal();
        Ok(Arc::new(()))
    })?;

    assert_eq!(sem.wait(MAX_DELAY), OsalRsBool::True);
    spawned.join(null_mut())?;

    log_info!(TAG, "test_semaphore_blocking_wait_forever PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// EventGroup
// ---------------------------------------------------------------------------

pub fn test_event_group_blocking_wait_forever() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_blocking_wait_forever");

    let events = Arc::new(EventGroup::new()?);

    let setter_events = events.clone();

    let mut setter = Thread::new("eg-setter", 2048, 5);
    let spawned = setter.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        setter_events.set(0b110);
        Ok(Arc::new(()))
    })?;

    let bits = events.wait(0b010, true, types::TickType::MAX);
    log_debug!(TAG, "unbounded wait returned 0b{:b}", bits);
    assert_eq!(bits & 0b010, 0b010);

    spawned.join(null_mut())?;

    log_info!(TAG, "test_event_group_blocking_wait_forever PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// Queue
// ---------------------------------------------------------------------------

pub fn test_queue_full_and_empty_from_isr() -> Result<()> {
    log_info!(TAG, "Starting test_queue_full_and_empty_from_isr");

    let queue = Queue::new(1, 4)?;
    let mut buffer = [0u8; 4];

    // Empty: the non-blocking fetch reports a timeout.
    assert!(matches!(
        queue.fetch_from_isr(&mut buffer),
        Err(Error::Timeout)
    ));

    queue.post_from_isr(&[7u8, 7, 7, 7])?;

    // Full: the non-blocking post reports the queue is full. This variant
    // never blocks, so `errQUEUE_FULL` is the only way it can fail.
    assert!(matches!(
        queue.post_from_isr(&[8u8, 8, 8, 8]),
        Err(Error::QueueFull)
    ));

    queue.fetch_from_isr(&mut buffer)?;
    assert_eq!(buffer, [7, 7, 7, 7]);

    log_info!(TAG, "test_queue_full_and_empty_from_isr PASSED");
    Ok(())
}

pub fn test_queue_blocking_forever_both_directions() -> Result<()> {
    log_info!(TAG, "Starting test_queue_blocking_forever_both_directions");

    // `TickType::MAX` selects an unbounded block on both the fetch and the
    // post side.
    let queue = Arc::new(Queue::new(1, 4)?);

    // --- fetch blocks until a producer posts ---
    let producer_queue = queue.clone();

    let mut producer = Thread::new("q-producer", 2048, 5);
    let spawned_producer = producer.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        producer_queue.post(&[1u8, 2, 3, 4], 0)?;
        Ok(Arc::new(()))
    })?;

    let mut buffer = [0u8; 4];
    queue.fetch(&mut buffer, types::TickType::MAX)?;
    assert_eq!(buffer, [1, 2, 3, 4]);
    spawned_producer.join(null_mut())?;

    // --- post blocks until a consumer drains the single slot ---
    queue.post(&[9u8, 9, 9, 9], 0)?;

    let consumer_queue = queue.clone();

    let mut consumer = Thread::new("q-consumer", 2048, 5);
    let spawned_consumer = consumer.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        let mut scratch = [0u8; 4];
        consumer_queue.fetch(&mut scratch, millis(100))?;
        Ok(Arc::new(()))
    })?;

    queue.post(&[5u8, 6, 7, 8], types::TickType::MAX)?;
    spawned_consumer.join(null_mut())?;

    queue.fetch(&mut buffer, millis(100))?;
    assert_eq!(buffer, [5, 6, 7, 8]);

    log_info!(TAG, "test_queue_blocking_forever_both_directions PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// Timer
// ---------------------------------------------------------------------------

pub fn test_timer_clone_after_original_deleted() -> Result<()> {
    log_info!(TAG, "Starting test_timer_clone_after_original_deleted");

    // `Timer` is `Clone` and clones share one `TimerShared`, so a clone can
    // outlive the `delete()` that tore the timer down. Its `shared` is still
    // `Some`, but no longer `ready`.
    let mut original = Timer::new("clone-src", millis(20), true, None, |_, param| {
        Ok(param.unwrap_or(Arc::new(())))
    })?;
    let mut clone = original.clone();

    assert_eq!(original.delete(0), OsalRsBool::True);

    // The clone still holds the shared state, so it gets past the `Option`
    // guard - and is then stopped by the `ready` flag instead.
    assert!(clone.is_null());
    assert_eq!(clone.start(0), OsalRsBool::False);
    assert_eq!(clone.stop(0), OsalRsBool::False);
    assert_eq!(clone.reset(0), OsalRsBool::False);
    assert_eq!(clone.change_period(millis(40), 0), OsalRsBool::False);

    // Deleting the clone is accepted (it does own a `shared`) but must not
    // ask the timer daemon to delete the already-deleted timer again.
    assert_eq!(clone.delete(0), OsalRsBool::True);
    assert_eq!(clone.delete(0), OsalRsBool::False);

    log_info!(TAG, "test_timer_clone_after_original_deleted PASSED");
    Ok(())
}

pub fn test_timer_callback_returning_error() -> Result<()> {
    log_info!(TAG, "Starting test_timer_callback_returning_error");

    static FIRES: AtomicU32 = AtomicU32::new(0);
    FIRES.store(0, Ordering::SeqCst);

    // A callback that fails must not stop an auto-reload timer: the failed
    // return is simply not adopted as the next `param`.
    let mut timer = Timer::new("failing-cb", millis(20), true, None, |_, _param| {
        FIRES.fetch_add(1, Ordering::SeqCst);
        Err(Error::Unhandled("callback failed on purpose"))
    })?;

    assert_eq!(timer.start(0), OsalRsBool::True);
    System::delay(millis(120));
    assert_eq!(timer.stop(0), OsalRsBool::True);

    let fires = FIRES.load(Ordering::SeqCst);
    log_debug!(TAG, "failing callback fired {} time(s)", fires);
    assert!(fires >= 2, "auto-reload must survive a failing callback");

    timer.delete(0);

    log_info!(TAG, "test_timer_callback_returning_error PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// Thread
// ---------------------------------------------------------------------------

pub fn test_thread_null_handle_guards() -> Result<()> {
    log_info!(TAG, "Starting test_thread_null_handle_guards");

    // Never spawned: the handle is still null.
    let unspawned = Thread::new("never-spawned", 1024, 3);
    assert!(unspawned.is_null());

    assert!(matches!(
        unspawned.notify(ThreadNotification::Increment),
        Err(Error::NullPtr)
    ));

    let mut woken = 0;
    assert!(matches!(
        unspawned.notify_from_isr(ThreadNotification::Increment, &mut woken),
        Err(Error::NullPtr)
    ));

    assert!(matches!(
        unspawned.wait_notification(0, 0, 0),
        Err(Error::NullPtr)
    ));
    assert!(matches!(
        unspawned.wait_notification_with_to_tick(0, 0, Duration::from_millis(1)),
        Err(Error::NullPtr)
    ));

    assert!(matches!(
        unspawned.join(null_mut()),
        Err(Error::NullPtr)
    ));

    // Suspend/resume/delete are silent no-ops rather than passing NULL to the
    // kernel.
    unspawned.suspend();
    unspawned.resume();
    unspawned.delete();

    // Metadata still reflects the constructor arguments, but the state is
    // `Invalid` because there is no live task.
    let metadata = unspawned.get_metadata();
    log_debug!(TAG, "unspawned metadata: {:?}", metadata.state);
    assert_eq!(metadata.state, ThreadState::Invalid);
    assert_eq!(metadata.name.as_str(), "never-spawned");
    assert_eq!(metadata.priority, 3);

    // `new_with_handle*` reject the null handle for the same reason.
    struct Lowest;
    impl ToPriority for Lowest {
        fn to_priority(&self) -> types::UBaseType {
            1
        }
    }

    assert!(Thread::new_with_handle(null_mut(), "null", 1024, 1).is_err());
    assert!(
        Thread::new_with_handle_and_to_priority(null_mut(), "null", 1024, Lowest)
            .is_err()
    );

    log_info!(TAG, "test_thread_null_handle_guards PASSED");
    Ok(())
}

pub fn test_thread_notification_without_overwrite_conflict() -> Result<()> {
    log_info!(TAG, "Starting test_thread_notification_without_overwrite_conflict");

    drain_pending_notification();
    let current = Thread::get_current();

    // First one lands and leaves a notification pending...
    current.notify(ThreadNotification::SetValueWithoutOverwrite(11))?;
    // ...so the second must fail instead of clobbering the unread value.
    assert!(matches!(
        current.notify(ThreadNotification::SetValueWithoutOverwrite(22)),
        Err(Error::QueueFull)
    ));

    assert_eq!(current.wait_notification(0, u32::MAX, 0)?, 11);

    // Once consumed, the same call succeeds again.
    current.notify(ThreadNotification::SetValueWithoutOverwrite(33))?;
    assert_eq!(current.wait_notification(0, u32::MAX, 0)?, 33);

    log_info!(TAG, "test_thread_notification_without_overwrite_conflict PASSED");
    Ok(())
}

pub fn test_thread_notification_all_actions() -> Result<()> {
    log_info!(TAG, "Starting test_thread_notification_all_actions");

    drain_pending_notification();
    let current = Thread::get_current();

    // `NoAction` marks a notification pending without changing the value.
    current.notify(ThreadNotification::NoAction)?;
    assert_eq!(current.wait_notification(0, 0, 0)?, 0);

    // `SetBits` ORs in.
    current.notify(ThreadNotification::SetBits(0b0101))?;
    assert_eq!(current.wait_notification(0, 0, 0)?, 0b0101);
    current.notify(ThreadNotification::SetBits(0b1010))?;
    assert_eq!(current.wait_notification(0, 0, 0)?, 0b1111);

    // `bits_to_clear_on_entry` wipes bits before the wait...
    current.notify(ThreadNotification::SetBits(0))?;
    assert_eq!(current.wait_notification(0b0001, 0, 0)?, 0b1110);

    // ...and `bits_to_clear_on_exit` wipes them after reading.
    current.notify(ThreadNotification::SetBits(0))?;
    assert_eq!(current.wait_notification(0, 0b0010, 0)?, 0b1110);
    current.notify(ThreadNotification::SetBits(0))?;
    assert_eq!(current.wait_notification(0, 0, 0)?, 0b1100);

    // `Increment` bumps by one.
    current.notify(ThreadNotification::SetValueWithOverwrite(41))?;
    assert_eq!(current.wait_notification(0, 0, 0)?, 41);
    current.notify(ThreadNotification::Increment)?;
    assert_eq!(current.wait_notification(0, u32::MAX, 0)?, 42);

    // Nothing pending any more: a bounded wait times out.
    assert!(matches!(
        current.wait_notification(0, 0, millis(5)),
        Err(Error::Timeout)
    ));

    log_info!(TAG, "test_thread_notification_all_actions PASSED");
    Ok(())
}

pub fn test_thread_wait_notification_forever() -> Result<()> {
    log_info!(TAG, "Starting test_thread_wait_notification_forever");

    drain_pending_notification();

    let waiter = Arc::new(Thread::get_current());

    let notifier_target = waiter.clone();

    let mut notifier = Thread::new("notify-source", 2048, 5);
    let spawned = notifier.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        notifier_target.notify(ThreadNotification::SetValueWithOverwrite(0x5A5A))?;
        Ok(Arc::new(()))
    })?;

    let value = waiter.wait_notification(0, u32::MAX, types::TickType::MAX)?;
    log_debug!(TAG, "unbounded wait_notification got 0x{:X}", value);
    assert_eq!(value, 0x5A5A);

    spawned.join(null_mut())?;

    log_info!(TAG, "test_thread_wait_notification_forever PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// QueueStreamed serialization failures (serde backend)
// ---------------------------------------------------------------------------

/// Payload types that deliberately break the serialization contract, so
/// `QueueStreamed`'s encode/decode error paths can be reached. Both wrap the
/// underlying `osal_rs_serde` failure into `Error::Unhandled`.
#[cfg(feature = "serde")]
mod broken_payloads {
    use osal_rs::os::BytesHasLen;
    use osal_rs_serde::{Deserialize, Deserializer, Error as SerdeError, Serialize, Serializer};

    /// Encodes as a single byte, but never decodes: drives the
    /// deserialization branch of `fetch`/`fetch_from_isr`.
    #[derive(Debug, Default, PartialEq)]
    pub struct NeverDecodes(pub u8);

    impl BytesHasLen for NeverDecodes {
        fn len(&self) -> usize {
            1
        }
    }

    impl Serialize for NeverDecodes {
        fn serialize<S: Serializer>(
            &self,
            name: &str,
            serializer: &mut S,
        ) -> core::result::Result<(), S::Error> {
            serializer.serialize_u8(name, self.0)
        }
    }

    impl Deserialize for NeverDecodes {
        fn deserialize<D: Deserializer>(
            _deserializer: &mut D,
            _name: &str,
        ) -> core::result::Result<Self, D::Error> {
            Err(SerdeError::InvalidData.into())
        }
    }

    /// Reports a length of 1 but encodes a `u32`, so the buffer
    /// `QueueStreamed` sizes from `BytesHasLen::len` is too small: drives the
    /// serialization branch of `post`/`post_from_isr`.
    #[derive(Debug, Default, PartialEq)]
    pub struct UnderReportsLen(pub u32);

    impl BytesHasLen for UnderReportsLen {
        fn len(&self) -> usize {
            1
        }
    }

    impl Serialize for UnderReportsLen {
        fn serialize<S: Serializer>(
            &self,
            name: &str,
            serializer: &mut S,
        ) -> core::result::Result<(), S::Error> {
            serializer.serialize_u32(name, self.0)
        }
    }

    impl Deserialize for UnderReportsLen {
        fn deserialize<D: Deserializer>(
            deserializer: &mut D,
            name: &str,
        ) -> core::result::Result<Self, D::Error> {
            Ok(Self(deserializer.deserialize_u32(name)?))
        }
    }
}

#[cfg(feature = "serde")]
pub fn test_queue_streamed_deserialization_failure() -> Result<()> {
    use broken_payloads::NeverDecodes;

    log_info!(TAG, "Starting test_queue_streamed_deserialization_failure");

    let queue = QueueStreamed::<NeverDecodes>::new(2, 1)?;

    // Encoding works, so the message really does reach the queue...
    queue.post(&NeverDecodes(7), 0)?;

    // ...but decoding it back fails, and the error is surfaced rather than
    // leaving `buffer` half-written.
    let mut buffer = NeverDecodes::default();
    let err = queue.fetch(&mut buffer, millis(10));
    log_debug!(TAG, "fetch decode error: {:?}", err);
    assert!(matches!(err, Err(Error::Unhandled(_))));

    queue.post_from_isr(&NeverDecodes(9))?;
    let err = queue.fetch_from_isr(&mut buffer);
    log_debug!(TAG, "fetch_from_isr decode error: {:?}", err);
    assert!(matches!(err, Err(Error::Unhandled(_))));

    log_info!(TAG, "test_queue_streamed_deserialization_failure PASSED");
    Ok(())
}

#[cfg(feature = "serde")]
pub fn test_queue_streamed_serialization_failure() -> Result<()> {
    use broken_payloads::UnderReportsLen;

    log_info!(TAG, "Starting test_queue_streamed_serialization_failure");

    let queue = QueueStreamed::<UnderReportsLen>::new(2, 1)?;

    // The scratch buffer is sized from `BytesHasLen::len()`, which is a byte
    // short of what the encoder needs, so nothing is ever posted.
    let err = queue.post(&UnderReportsLen(0xDEAD_BEEF), 0);
    log_debug!(TAG, "post encode error: {:?}", err);
    assert!(matches!(err, Err(Error::Unhandled(_))));

    let err = queue.post_from_isr(&UnderReportsLen(0xDEAD_BEEF));
    log_debug!(TAG, "post_from_isr encode error: {:?}", err);
    assert!(matches!(err, Err(Error::Unhandled(_))));

    // Nothing made it in, so a fetch just times out.
    let mut buffer = UnderReportsLen::default();
    assert!(matches!(
        queue.fetch(&mut buffer, millis(10)),
        Err(Error::Timeout)
    ));

    log_info!(TAG, "test_queue_streamed_serialization_failure PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Error Path Tests ==========");
    test_raw_mutex_after_delete()?;
    test_semaphore_after_delete()?;
    test_event_group_after_delete()?;
    test_queue_after_delete()?;
    test_timer_after_delete()?;
    test_timer_clone_after_original_deleted()?;
    test_queue_invalid_construction()?;
    test_queue_undersized_buffers()?;
    test_mutex_accessors_without_locking()?;
    test_semaphore_blocking_wait_forever()?;
    test_event_group_blocking_wait_forever()?;
    test_queue_full_and_empty_from_isr()?;
    test_queue_blocking_forever_both_directions()?;
    test_timer_callback_returning_error()?;
    test_thread_null_handle_guards()?;
    test_thread_notification_without_overwrite_conflict()?;
    test_thread_notification_all_actions()?;
    test_thread_wait_notification_forever()?;
    #[cfg(feature = "serde")]
    test_queue_streamed_deserialization_failure()?;
    #[cfg(feature = "serde")]
    test_queue_streamed_serialization_failure()?;
    log_info!(TAG, "========== All Error Path Tests PASSED ==========");
    Ok(())
}
