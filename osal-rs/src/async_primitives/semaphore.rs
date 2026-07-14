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

//! Async wrapper for OSAL semaphores.
//!
//! [`AsyncSemaphore`] exposes a `wait_async()` method that returns a `Future`.
//! The synchronous `signal()` wakes any task sleeping in `wait_async`.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use alloc::sync::Arc;

use crate::os::Semaphore;
use crate::os::types::UBaseType;
use crate::traits::SemaphoreFn;
use crate::utils::OsalRsBool;

use super::waker_slot::WakerSlot;

/// An async-capable semaphore wrapping an `Arc<Semaphore>`.
pub struct AsyncSemaphore {
    inner: Arc<Semaphore>,
    waker: Arc<WakerSlot>,
}

impl AsyncSemaphore {
    /// Creates an `AsyncSemaphore` with `max_count` and `initial_count`.
    ///
    /// # Errors
    ///
    /// Returns the underlying OSAL error if the semaphore cannot be created.
    pub fn new(max_count: u32, initial_count: u32) -> crate::utils::Result<Self> {
        let inner = Semaphore::new(
            max_count as UBaseType,
            initial_count as UBaseType,
        )?;
        Ok(Self {
            inner: Arc::new(inner),
            waker: Arc::new(WakerSlot::new()),
        })
    }

    /// Signals (increments) the semaphore and wakes any async waiter.
    pub fn signal(&self) -> OsalRsBool {
        let result = self.inner.signal();
        self.waker.wake();
        result
    }

    /// Returns a `Future` that resolves when the semaphore count becomes > 0.
    pub fn wait_async(&self) -> WaitFuture<'_> {
        WaitFuture { sem: self }
    }
}

/// Future returned by [`AsyncSemaphore::wait_async`].
pub struct WaitFuture<'a> {
    sem: &'a AsyncSemaphore,
}

impl<'a> Future for WaitFuture<'a> {
    type Output = OsalRsBool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Non-blocking attempt.
        if self.sem.inner.wait(Duration::ZERO) == OsalRsBool::True {
            return Poll::Ready(OsalRsBool::True);
        }

        // Store waker, then retry to close the TOCTOU window.
        self.sem.waker.store(cx.waker());

        if self.sem.inner.wait(Duration::ZERO) == OsalRsBool::True {
            Poll::Ready(OsalRsBool::True)
        } else {
            Poll::Pending
        }
    }
}
