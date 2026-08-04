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

//! Failure-path tests: unbounded (`MAX_DELAY`) waits, the notification
//! conflict/clear semantics, full/empty `_from_isr` results, a timer callback
//! that fails, and `QueueStreamed`'s (de)serialization errors.
//!
//! Mirrors the **portable subset** of `osal-rs/tests/std_error_paths_tests.rs`.
//! The POSIX file is larger on purpose - a good part of it drives defensive
//! code that only `src/posix/` has, and mirroring it here would produce a
//! suite that trips `configASSERT` on target. Specifically **not** mirrored:
//!
//! * *"after `delete()`"* tests (`RawMutex`, `Semaphore`, `EventGroup`,
//!   `Queue`). The POSIX backend checks `is_null()` at the top of every
//!   operation and reports `False`/`Error::NullPtr`; the FreeRTOS backend
//!   hands the raw handle straight to the kernel, so the same calls on a
//!   deleted object pass `NULL` to FreeRTOS. Double `delete()` is unsafe here
//!   for the same reason (`vSemaphoreDelete(NULL)`).
//! * `test_queue_invalid_construction`. `posix::Queue::new` rejects a zero
//!   size/message size with `Error::InvalidQueueSize`; `xQueueGenericCreate`
//!   asserts on it instead.
//! * `test_queue_undersized_buffers`. `posix::Queue` validates the slice
//!   against the queue's message size; `xQueueReceive`/`xQueueSendToBack` copy
//!   a fixed item size regardless.
//! * The `Timer` "after delete"/clone tests. `posix::Timer` keeps an
//!   `Option<Arc<TimerShared>>` that survives `delete()`; `freertos::Timer` is
//!   a bare handle.
//! * The `*_deadline_crosses_second_boundary` tests, which exist purely to
//!   drive the `timespec` normalisation in the POSIX backend's absolute
//!   deadline arithmetic. FreeRTOS takes relative tick counts.
//! * The contended `lock_from_isr` tests. FreeRTOS's recursive mutexes must
//!   not be taken from ISR context at all, so the "contended `trylock`"
//!   scenario has no meaningful FreeRTOS counterpart.

extern crate alloc;

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::time::Duration;

use alloc::sync::Arc;

use osal_rs::os::*;
use osal_rs::utils::{Error, OsalRsBool, Result, MAX_DELAY};
use osal_rs::{log_debug, log_info};

const TAG: &str = "ErrorPathTests";

/// Head start given to a helper task before the main task blocks.
const HEAD_START_MS: u64 = 40;

/// Upper bound on how long [`await_helper`] waits before giving up.
const HELPER_TIMEOUT_MS: u64 = 2_000;

fn millis(ms: u64) -> types::TickType {
    Duration::from_millis(ms).to_ticks()
}

/// Blocks until `done` is set, then reclaims `helper`.
///
/// Deliberately *not* `helper.join()`: on FreeRTOS that is `vTaskDelete`, so
/// joining a still-running helper would delete it rather than wait for it.
fn await_helper(helper: &Thread, done: &AtomicBool) -> Result<()> {
    let mut waited = 0u64;
    while !done.load(Ordering::Acquire) {
        System::delay(millis(5));
        waited += 5;
        assert!(waited < HELPER_TIMEOUT_MS, "helper task never finished");
    }
    helper.delete();
    Ok(())
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
    let done = Arc::new(AtomicBool::new(false));

    let signaller_sem = sem.clone();
    let signaller_done = done.clone();

    let mut signaller = Thread::new("sem-signaller", 2048, 5);
    let spawned = signaller.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        signaller_sem.signal();
        signaller_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    assert_eq!(sem.wait(MAX_DELAY), OsalRsBool::True);
    await_helper(&spawned, &done)?;

    log_info!(TAG, "test_semaphore_blocking_wait_forever PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// EventGroup
// ---------------------------------------------------------------------------

pub fn test_event_group_blocking_wait_forever() -> Result<()> {
    log_info!(TAG, "Starting test_event_group_blocking_wait_forever");

    let events = Arc::new(EventGroup::new()?);
    let done = Arc::new(AtomicBool::new(false));

    let setter_events = events.clone();
    let setter_done = done.clone();

    let mut setter = Thread::new("eg-setter", 2048, 5);
    let spawned = setter.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        setter_events.set(0b110);
        setter_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    let bits = events.wait(0b010, true, types::TickType::MAX);
    log_debug!(TAG, "unbounded wait returned 0b{:b}", bits);
    assert_eq!(bits & 0b010, 0b010);

    await_helper(&spawned, &done)?;

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

    // Full: the non-blocking post fails. Note the divergence from the POSIX
    // backend, which distinguishes this case as `Error::QueueFull` - here
    // `xQueueSendToBackFromISR` just returns `errQUEUE_FULL`, which the
    // backend maps to `Error::Timeout` like any other refusal.
    assert!(matches!(
        queue.post_from_isr(&[8u8, 8, 8, 8]),
        Err(Error::Timeout)
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
    let producer_done = Arc::new(AtomicBool::new(false));
    let producer_queue = queue.clone();
    let producer_mark = producer_done.clone();

    let mut producer = Thread::new("q-producer", 2048, 5);
    let spawned_producer = producer.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        producer_queue.post(&[1u8, 2, 3, 4], 0)?;
        producer_mark.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    let mut buffer = [0u8; 4];
    queue.fetch(&mut buffer, types::TickType::MAX)?;
    assert_eq!(buffer, [1, 2, 3, 4]);
    await_helper(&spawned_producer, &producer_done)?;

    // --- post blocks until a consumer drains the single slot ---
    queue.post(&[9u8, 9, 9, 9], 0)?;

    let consumer_done = Arc::new(AtomicBool::new(false));
    let consumer_queue = queue.clone();
    let consumer_mark = consumer_done.clone();

    let mut consumer = Thread::new("q-consumer", 2048, 5);
    let spawned_consumer = consumer.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        let mut scratch = [0u8; 4];
        consumer_queue.fetch(&mut scratch, millis(100))?;
        consumer_mark.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    queue.post(&[5u8, 6, 7, 8], types::TickType::MAX)?;
    await_helper(&spawned_consumer, &consumer_done)?;

    queue.fetch(&mut buffer, millis(100))?;
    assert_eq!(buffer, [5, 6, 7, 8]);

    log_info!(TAG, "test_queue_blocking_forever_both_directions PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// Timer
// ---------------------------------------------------------------------------

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

    // Suspend/resume/delete are silent no-ops rather than passing NULL to the
    // kernel. `join` is `vTaskDelete` here, so it is also a no-op returning
    // `Ok(0)` - unlike the POSIX backend, which reports `Error::NullPtr`.
    unspawned.suspend();
    unspawned.resume();
    unspawned.delete();
    assert_eq!(unspawned.join(core::ptr::null_mut())?, 0);

    // Metadata for a handle-less thread is the all-zero default (the POSIX
    // backend instead echoes back the constructor's name/priority).
    let metadata = unspawned.get_metadata();
    log_debug!(TAG, "unspawned metadata state: {:?}", metadata.state);
    assert_eq!(metadata.state, ThreadState::Invalid);

    // `new_with_handle*` reject the null handle for the same reason.
    struct Lowest;
    impl ToPriority for Lowest {
        fn to_priority(&self) -> types::UBaseType {
            1
        }
    }

    assert!(Thread::new_with_handle(core::ptr::null_mut(), "null", 1024, 1).is_err());
    assert!(
        Thread::new_with_handle_and_to_priority(core::ptr::null_mut(), "null", 1024, Lowest)
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
    let done = Arc::new(AtomicBool::new(false));

    let notifier_target = waiter.clone();
    let notifier_done = done.clone();

    let mut notifier = Thread::new("notify-source", 2048, 5);
    let spawned = notifier.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        notifier_target.notify(ThreadNotification::SetValueWithOverwrite(0x5A5A))?;
        notifier_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    let value = waiter.wait_notification(0, u32::MAX, types::TickType::MAX)?;
    log_debug!(TAG, "unbounded wait_notification got 0x{:X}", value);
    assert_eq!(value, 0x5A5A);

    await_helper(&spawned, &done)?;

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
