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

//! Tests for the backend-agnostic async layer (feature `async`) against the
//! POSIX backend: `block_on`, `AsyncMutex`, `AsyncQueue`, `AsyncSemaphore`.
//!
//! `AsyncMutex`/`AsyncSemaphore` are backed by `posix::Semaphore`, whose
//! stub `wait()` always returns `True` immediately — so both resolve on the
//! first poll here even though they don't provide real mutual exclusion
//! (same caveat as `std_mutex_tests.rs`/`std_semaphore_tests.rs`).
//!
//! `AsyncQueue::fetch_async` is intentionally NOT tested: it is backed by
//! `posix::Queue::fetch`, which always returns `Err(Timeout)` because
//! nothing is ever actually stored. `FetchFuture::poll` would therefore
//! return `Poll::Pending` forever, and `block_on`'s retry loop would spin
//! indefinitely (its own wait is on a semaphore that also always succeeds
//! immediately) — see `src/posix/queue.rs` and `src/async_executor/executor.rs`.

#![cfg(all(feature = "posix", feature = "async"))]

use osal_rs::os::{block_on, AsyncMutex, AsyncQueue, AsyncSemaphore};
use osal_rs::utils::{OsalRsBool, Result};
use osal_rs::{log_debug, log_info};

const TAG: &str = "AsyncTests";

#[test]
fn test_block_on_ready_future() -> Result<()> {
    log_info!(TAG, "Starting test_block_on_ready_future");
    let result = block_on(async { 41u32 + 1 });
    log_debug!(TAG, "block_on result: {}", result);
    assert_eq!(result, 42);
    log_info!(TAG, "test_block_on_ready_future PASSED");
    Ok(())
}

#[test]
fn test_async_mutex_lock() -> Result<()> {
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

#[test]
fn test_async_semaphore_wait_and_signal() -> Result<()> {
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

#[test]
fn test_async_queue_post() -> Result<()> {
    log_info!(TAG, "Starting test_async_queue_post");
    let queue = AsyncQueue::new(4, 4)?;
    let payload = [1u8, 2, 3, 4];

    block_on(queue.post_async(&payload)).unwrap();
    queue.post(&payload, 0)?;

    log_info!(TAG, "test_async_queue_post PASSED");
    Ok(())
}
