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

//! Tests for the backend-agnostic async layer (feature `async`): `block_on`,
//! `WakerSlot`, `AsyncMutex`, `AsyncQueue`, `AsyncSemaphore`.
//!
//! Mirrors `osal-rs/tests/std_async_tests.rs`; keep the two in sync. The async
//! layer itself is backend-independent, so almost everything ports across. Two
//! differences to keep in mind when editing:
//!
//! * FreeRTOS's `ThreadFn::join` is `vTaskDelete`, not a barrier - it would
//!   *kill* a helper task mid-work. Helpers here publish an `AtomicBool` when
//!   they are done and are reclaimed with `delete()` afterwards, matching the
//!   convention in `thread_tests.rs`.
//! * `test_async_queue_async_error_paths` is **not** mirrored: it asserts the
//!   `Error::InvalidQueueSize` that `posix::Queue` returns for an undersized
//!   buffer, and `freertos::Queue` has no equivalent check (`xQueueReceive`
//!   copies the queue's item size regardless of the slice length).

extern crate alloc;

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};
use core::time::Duration;

use alloc::sync::Arc;

use osal_rs::os::*;
use osal_rs::utils::{Error, OsalRsBool, Result};
use osal_rs::{log_debug, log_info};

const TAG: &str = "AsyncTests";

/// Head start given to a helper task before the main task starts polling, so
/// the main task reliably hits the `Poll::Pending` branch.
const HEAD_START_MS: u64 = 40;

/// How long a helper task withholds the resource once it owns it.
const HOLD_MS: u64 = 120;

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

// ---------------------------------------------------------------------------
// block_on / executor
// ---------------------------------------------------------------------------

pub fn test_block_on_ready_future() -> Result<()> {
    log_info!(TAG, "Starting test_block_on_ready_future");
    let result = block_on(async { 41u32 + 1 });
    log_debug!(TAG, "block_on result: {}", result);
    assert_eq!(result, 42);
    log_info!(TAG, "test_block_on_ready_future PASSED");
    Ok(())
}

pub fn test_block_on_nested_awaits() -> Result<()> {
    log_info!(TAG, "Starting test_block_on_nested_awaits");

    async fn double(v: u32) -> u32 {
        v * 2
    }

    let result = block_on(async { double(double(3).await).await });
    log_debug!(TAG, "nested block_on result: {}", result);
    assert_eq!(result, 12);

    log_info!(TAG, "test_block_on_nested_awaits PASSED");
    Ok(())
}

/// Future that returns `Pending` on its first poll after waking itself
/// through `Waker::wake_by_ref`, then `Ready` on the second.
struct WakeByRefFuture {
    polls: u32,
}

impl Future for WakeByRefFuture {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        self.polls += 1;
        if self.polls >= 2 {
            Poll::Ready(self.polls)
        } else {
            // Wakes without consuming the waker - the `wake_by_ref` vtable
            // entry, which no library-internal path exercises.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn test_block_on_repolls_after_wake_by_ref() -> Result<()> {
    log_info!(TAG, "Starting test_block_on_repolls_after_wake_by_ref");

    // Drives `block_on`'s `Poll::Pending` -> wait -> re-poll loop entirely on
    // one task: the future signals the executor semaphore before parking, so
    // the wait returns immediately.
    let polls = block_on(WakeByRefFuture { polls: 0 });
    log_debug!(TAG, "future polled {} times", polls);
    assert_eq!(polls, 2);

    log_info!(TAG, "test_block_on_repolls_after_wake_by_ref PASSED");
    Ok(())
}

/// Future that clones its waker into a slot on the first poll and completes
/// once something else wakes that clone.
struct WakeClonedFuture<'a> {
    slot: &'a WakerSlot,
    ready: &'a AtomicU32,
}

impl<'a> Future for WakeClonedFuture<'a> {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        let value = self.ready.load(Ordering::Acquire);
        if value != 0 {
            return Poll::Ready(value);
        }
        // `store` clones the waker - the `clone` vtable entry.
        self.slot.store(cx.waker());
        let value = self.ready.load(Ordering::Acquire);
        if value != 0 {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

pub fn test_block_on_woken_from_another_thread() -> Result<()> {
    log_info!(TAG, "Starting test_block_on_woken_from_another_thread");

    let slot = Arc::new(WakerSlot::new());
    let ready = Arc::new(AtomicU32::new(0));
    let done = Arc::new(AtomicBool::new(false));

    let waker_slot = slot.clone();
    let waker_ready = ready.clone();
    let waker_done = done.clone();

    let mut waker_thread = Thread::new("async-waker", 2048, 5);
    let spawned = waker_thread.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        waker_ready.store(0xABCD, Ordering::Release);
        // Consumes the stored clone - the `wake` vtable entry.
        waker_slot.wake();
        waker_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    let value = block_on(WakeClonedFuture {
        slot: &slot,
        ready: &ready,
    });
    log_debug!(TAG, "woken with value 0x{:X}", value);
    assert_eq!(value, 0xABCD);

    await_helper(&spawned, &done)?;

    log_info!(TAG, "test_block_on_woken_from_another_thread PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// WakerSlot
// ---------------------------------------------------------------------------

pub fn test_waker_slot_store_and_wake() -> Result<()> {
    log_info!(TAG, "Starting test_waker_slot_store_and_wake");

    let slot = WakerSlot::new();

    // Waking an empty slot is a no-op, not a null deref.
    slot.wake();

    slot.store(&Waker::noop().clone());
    slot.wake();

    // A second store replaces (and drops) the first waker.
    slot.store(&Waker::noop().clone());
    slot.store(&Waker::noop().clone());
    slot.wake();

    // Waking twice: the second call finds the slot empty again.
    slot.wake();

    log_info!(TAG, "test_waker_slot_store_and_wake PASSED");
    Ok(())
}

pub fn test_waker_slot_default_and_drop() -> Result<()> {
    log_info!(TAG, "Starting test_waker_slot_default_and_drop");

    let slot = WakerSlot::default();
    slot.wake();

    // Dropping a slot that still holds an unconsumed waker must release it
    // rather than leak the boxed waker.
    let pending = WakerSlot::default();
    pending.store(&Waker::noop().clone());
    drop(pending);

    log_info!(TAG, "test_waker_slot_default_and_drop PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// AsyncMutex
// ---------------------------------------------------------------------------

pub fn test_async_mutex_lock() -> Result<()> {
    log_info!(TAG, "Starting test_async_mutex_lock");
    let value = block_on(async {
        let mutex = AsyncMutex::new(0u32);
        {
            let mut guard = mutex.lock().await;
            *guard += 41;
        }
        let guard = mutex.lock().await;
        *guard
    });
    log_debug!(TAG, "AsyncMutex value: {}", value);
    assert_eq!(value, 41);
    log_info!(TAG, "test_async_mutex_lock PASSED");
    Ok(())
}

pub fn test_async_mutex_guard_deref_mut() -> Result<()> {
    log_info!(TAG, "Starting test_async_mutex_guard_deref_mut");

    let mutex = AsyncMutex::new(0u32);

    block_on(async {
        let mut guard = mutex.lock().await;
        // `DerefMut` then `Deref` on the same guard.
        *guard += 7;
        assert_eq!(*guard, 7);
    });

    let final_value = block_on(async { *mutex.lock().await });
    log_debug!(TAG, "AsyncMutex final value: {}", final_value);
    assert_eq!(final_value, 7);

    log_info!(TAG, "test_async_mutex_guard_deref_mut PASSED");
    Ok(())
}

pub fn test_async_mutex_contended_lock_parks_and_resumes() -> Result<()> {
    log_info!(TAG, "Starting test_async_mutex_contended_lock_parks_and_resumes");

    let mutex = Arc::new(AsyncMutex::new(0u32));
    let done = Arc::new(AtomicBool::new(false));

    let holder_mutex = mutex.clone();
    let holder_done = done.clone();

    let mut holder = Thread::new("async-mutex-holder", 2048, 5);
    let spawned = holder.spawn_simple(move || {
        let mutex = holder_mutex.clone();
        block_on(async move {
            let mut guard = mutex.lock().await;
            *guard = 1;
            // Hold the lock long enough that the main task is guaranteed to
            // see `Poll::Pending` and park its waker.
            System::delay(millis(HOLD_MS));
            *guard = 2;
        });
        holder_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    System::delay(millis(HEAD_START_MS));

    // Blocks in `block_on`'s wait until `AsyncMutexGuard::drop` signals the
    // semaphore and wakes the parked waker.
    let observed = block_on(async { *mutex.lock().await });
    log_debug!(TAG, "value observed after contention: {}", observed);
    assert_eq!(observed, 2, "must have waited for the holder to finish");

    await_helper(&spawned, &done)?;

    log_info!(TAG, "test_async_mutex_contended_lock_parks_and_resumes PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// AsyncSemaphore
// ---------------------------------------------------------------------------

pub fn test_async_semaphore_wait_and_signal() -> Result<()> {
    log_info!(TAG, "Starting test_async_semaphore_wait_and_signal");
    let sem = AsyncSemaphore::new(1, 0)?;

    let signal_result = sem.signal();
    log_debug!(TAG, "signal result: {:?}", signal_result);
    assert_eq!(signal_result, OsalRsBool::True);

    let wait_result = block_on(sem.wait_async());
    log_debug!(TAG, "wait_async result: {:?}", wait_result);
    assert_eq!(wait_result, OsalRsBool::True);

    log_info!(TAG, "test_async_semaphore_wait_and_signal PASSED");
    Ok(())
}

pub fn test_async_semaphore_signal_at_max_count() -> Result<()> {
    log_info!(TAG, "Starting test_async_semaphore_signal_at_max_count");

    let sem = AsyncSemaphore::new(1, 1)?;
    // Already at `max_count`: the underlying signal fails, and the waker is
    // still woken (harmlessly) on the way out.
    assert_eq!(sem.signal(), OsalRsBool::False);
    assert_eq!(block_on(sem.wait_async()), OsalRsBool::True);

    log_info!(TAG, "test_async_semaphore_signal_at_max_count PASSED");
    Ok(())
}

pub fn test_async_semaphore_wait_parks_until_signalled() -> Result<()> {
    log_info!(TAG, "Starting test_async_semaphore_wait_parks_until_signalled");

    let sem = Arc::new(AsyncSemaphore::new(1, 0)?);
    let signalled = Arc::new(AtomicU32::new(0));
    let done = Arc::new(AtomicBool::new(false));

    let signaller_sem = sem.clone();
    let signaller_mark = signalled.clone();
    let signaller_done = done.clone();

    let mut signaller = Thread::new("async-sem-signal", 2048, 5);
    let spawned = signaller.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        signaller_mark.store(1, Ordering::Release);
        signaller_sem.signal();
        signaller_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    // Empty semaphore: first poll parks, the signal above resumes it.
    let result = block_on(sem.wait_async());
    log_debug!(TAG, "parked wait_async result: {:?}", result);
    assert_eq!(result, OsalRsBool::True);
    assert_eq!(
        signalled.load(Ordering::Acquire),
        1,
        "must have waited for the signaller"
    );

    await_helper(&spawned, &done)?;

    log_info!(TAG, "test_async_semaphore_wait_parks_until_signalled PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// AsyncQueue
// ---------------------------------------------------------------------------

pub fn test_async_queue_post_and_fetch_sync() -> Result<()> {
    log_info!(TAG, "Starting test_async_queue_post_and_fetch_sync");

    let queue = AsyncQueue::new(4, 4)?;
    let payload = [1u8, 2, 3, 4];

    queue.post(&payload, 100)?;

    let mut buffer = [0u8; 4];
    queue.fetch(&mut buffer, 100)?;
    log_debug!(TAG, "fetched {:?}", buffer);
    assert_eq!(buffer, payload);

    // Empty again: a bounded fetch times out rather than blocking forever.
    assert!(matches!(queue.fetch(&mut buffer, 10), Err(Error::Timeout)));

    log_info!(TAG, "test_async_queue_post_and_fetch_sync PASSED");
    Ok(())
}

pub fn test_async_queue_post_and_fetch_async() -> Result<()> {
    log_info!(TAG, "Starting test_async_queue_post_and_fetch_async");

    let queue = AsyncQueue::new(4, 4)?;
    let payload = [9u8, 8, 7, 6];

    block_on(queue.post_async(&payload))?;

    let mut buffer = [0u8; 4];
    // The item is already queued, so this resolves on the first poll.
    block_on(queue.fetch_async(&mut buffer))?;
    log_debug!(TAG, "fetch_async got {:?}", buffer);
    assert_eq!(buffer, payload);

    log_info!(TAG, "test_async_queue_post_and_fetch_async PASSED");
    Ok(())
}

pub fn test_async_queue_fetch_async_parks_until_posted() -> Result<()> {
    log_info!(TAG, "Starting test_async_queue_fetch_async_parks_until_posted");

    let queue = Arc::new(AsyncQueue::new(2, 4)?);
    let done = Arc::new(AtomicBool::new(false));

    let producer_queue = queue.clone();
    let producer_done = done.clone();

    let mut producer = Thread::new("async-q-producer", 2048, 5);
    let spawned = producer.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        // Synchronous `post` wakes the parked `FetchFuture`.
        producer_queue.post(&[0xAAu8, 0xBB, 0xCC, 0xDD], 100)?;
        producer_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    let mut buffer = [0u8; 4];
    block_on(queue.fetch_async(&mut buffer))?;
    log_debug!(TAG, "parked fetch_async got {:?}", buffer);
    assert_eq!(buffer, [0xAA, 0xBB, 0xCC, 0xDD]);

    await_helper(&spawned, &done)?;

    log_info!(TAG, "test_async_queue_fetch_async_parks_until_posted PASSED");
    Ok(())
}

pub fn test_async_queue_post_async_parks_until_drained() -> Result<()> {
    log_info!(TAG, "Starting test_async_queue_post_async_parks_until_drained");

    // Single slot, filled up front: the next post cannot complete until a
    // consumer drains it.
    let queue = Arc::new(AsyncQueue::new(1, 4)?);
    queue.post(&[1u8, 1, 1, 1], 100)?;

    let drained = Arc::new(AtomicU32::new(0));
    let done = Arc::new(AtomicBool::new(false));

    let consumer_queue = queue.clone();
    let consumer_mark = drained.clone();
    let consumer_done = done.clone();

    let mut consumer = Thread::new("async-q-consumer", 2048, 5);
    let spawned = consumer.spawn_simple(move || {
        System::delay(millis(HEAD_START_MS));
        let mut buffer = [0u8; 4];
        // Synchronous `fetch` wakes the parked `PostFuture`.
        consumer_queue.fetch(&mut buffer, 100)?;
        consumer_mark.store(1, Ordering::Release);
        consumer_done.store(true, Ordering::Release);
        Ok(Arc::new(()))
    })?;

    block_on(queue.post_async(&[2u8, 2, 2, 2]))?;
    assert_eq!(
        drained.load(Ordering::Acquire),
        1,
        "must have waited for the consumer to free a slot"
    );

    await_helper(&spawned, &done)?;

    let mut buffer = [0u8; 4];
    queue.fetch(&mut buffer, 100)?;
    log_debug!(TAG, "queue drained to {:?}", buffer);
    assert_eq!(buffer, [2, 2, 2, 2]);

    log_info!(TAG, "test_async_queue_post_async_parks_until_drained PASSED");
    Ok(())
}

pub fn test_async_queue_saturating_timeout_conversion() -> Result<()> {
    log_info!(TAG, "Starting test_async_queue_saturating_timeout_conversion");

    let queue = AsyncQueue::new(1, 4)?;

    // A millisecond timeout far beyond `TickType::MAX` must saturate rather
    // than wrap; the post itself succeeds immediately since the queue is empty.
    queue.post(&[4u8, 3, 2, 1], u64::MAX)?;

    let mut buffer = [0u8; 4];
    queue.fetch(&mut buffer, u64::MAX)?;
    log_debug!(TAG, "saturated-timeout fetch got {:?}", buffer);
    assert_eq!(buffer, [4, 3, 2, 1]);

    log_info!(TAG, "test_async_queue_saturating_timeout_conversion PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Async Tests ==========");
    test_block_on_ready_future()?;
    test_block_on_nested_awaits()?;
    test_block_on_repolls_after_wake_by_ref()?;
    test_block_on_woken_from_another_thread()?;
    test_waker_slot_store_and_wake()?;
    test_waker_slot_default_and_drop()?;
    test_async_mutex_lock()?;
    test_async_mutex_guard_deref_mut()?;
    test_async_mutex_contended_lock_parks_and_resumes()?;
    test_async_semaphore_wait_and_signal()?;
    test_async_semaphore_signal_at_max_count()?;
    test_async_semaphore_wait_parks_until_signalled()?;
    test_async_queue_post_and_fetch_sync()?;
    test_async_queue_post_and_fetch_async()?;
    test_async_queue_fetch_async_parks_until_posted()?;
    test_async_queue_post_async_parks_until_drained()?;
    test_async_queue_saturating_timeout_conversion()?;
    log_info!(TAG, "========== All Async Tests PASSED ==========");
    Ok(())
}
