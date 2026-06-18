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

//! Async wrapper for OSAL queues.
//!
//! [`AsyncQueue`] wraps an [`Arc<Queue>`] and exposes `fetch_async` /
//! `post_async` methods that return `Future`s.  The synchronous `post` /
//! `fetch` counterparts on the wrapper also wake any sleeping async task
//! waiting on the other end.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use alloc::sync::Arc;

use crate::os::Queue;
use crate::traits::QueueFn;
use crate::utils::{Error, Result};

use super::waker_slot::WakerSlot;

/// An async-capable queue wrapping an `Arc<Queue>`.
///
/// Both the synchronous and asynchronous APIs are available.  When using the
/// synchronous API, pending async waiters are woken automatically.
pub struct AsyncQueue {
    inner: Arc<Queue>,
    rx_waker: Arc<WakerSlot>,
    tx_waker: Arc<WakerSlot>,
}

impl AsyncQueue {
    /// Creates an `AsyncQueue` that owns a new OSAL queue.
    ///
    /// `size` is the maximum number of items; `message_size` is each item's
    /// size in bytes.
    ///
    /// # Errors
    ///
    /// Returns the underlying OSAL error if the queue cannot be created.
    pub fn new(size: u32, message_size: u32) -> Result<Self> {
        let inner = Queue::new(size as crate::os::types::UBaseType, message_size as crate::os::types::UBaseType)?;
        Ok(Self {
            inner: Arc::new(inner),
            rx_waker: Arc::new(WakerSlot::new()),
            tx_waker: Arc::new(WakerSlot::new()),
        })
    }

    /// Posts `item` synchronously and wakes any async task waiting to receive.
    pub fn post(&self, item: &[u8], timeout_ms: u64) -> Result<()> {
        let ticks = crate::os::types::TickType::try_from(
            Duration::from_millis(timeout_ms)
                .as_millis()
                .try_into()
                .unwrap_or(crate::os::types::TickType::MAX),
        )
        .unwrap_or(crate::os::types::TickType::MAX);
        let result = self.inner.post(item, ticks);
        if result.is_ok() {
            self.rx_waker.wake();
        }
        result
    }

    /// Fetches an item synchronously and wakes any async task waiting to send.
    pub fn fetch(&self, buf: &mut [u8], timeout_ms: u64) -> Result<()> {
        let ticks = crate::os::types::TickType::try_from(
            Duration::from_millis(timeout_ms)
                .as_millis()
                .try_into()
                .unwrap_or(crate::os::types::TickType::MAX),
        )
        .unwrap_or(crate::os::types::TickType::MAX);
        let result = self.inner.fetch(buf, ticks);
        if result.is_ok() {
            self.tx_waker.wake();
        }
        result
    }

    /// Returns a `Future` that resolves when an item has been fetched into `buf`.
    pub fn fetch_async<'a>(&'a self, buf: &'a mut [u8]) -> FetchFuture<'a> {
        FetchFuture {
            queue: self,
            buf,
        }
    }

    /// Returns a `Future` that resolves when `item` has been posted to the queue.
    pub fn post_async<'a>(&'a self, item: &'a [u8]) -> PostFuture<'a> {
        PostFuture {
            queue: self,
            item,
        }
    }
}

/// Future returned by [`AsyncQueue::fetch_async`].
pub struct FetchFuture<'a> {
    queue: &'a AsyncQueue,
    buf: &'a mut [u8],
}

impl<'a> Future for FetchFuture<'a> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Non-blocking attempt first.
        match this.queue.inner.fetch(this.buf, 0) {
            Ok(()) => {
                this.queue.tx_waker.wake();
                return Poll::Ready(Ok(()));
            }
            Err(Error::Timeout) => {}
            Err(e) => return Poll::Ready(Err(e)),
        }

        // Store waker, then try again to close the TOCTOU window.
        this.queue.rx_waker.store(cx.waker());

        match this.queue.inner.fetch(this.buf, 0) {
            Ok(()) => {
                this.queue.tx_waker.wake();
                Poll::Ready(Ok(()))
            }
            Err(Error::Timeout) => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

/// Future returned by [`AsyncQueue::post_async`].
pub struct PostFuture<'a> {
    queue: &'a AsyncQueue,
    item: &'a [u8],
}

impl<'a> Future for PostFuture<'a> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match this.queue.inner.post(this.item, 0) {
            Ok(()) => {
                this.queue.rx_waker.wake();
                return Poll::Ready(Ok(()));
            }
            Err(Error::Timeout) => {}
            Err(e) => return Poll::Ready(Err(e)),
        }

        this.queue.tx_waker.store(cx.waker());

        match this.queue.inner.post(this.item, 0) {
            Ok(()) => {
                this.queue.rx_waker.wake();
                Poll::Ready(Ok(()))
            }
            Err(Error::Timeout) => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
