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

//! Async mutual-exclusion lock built on an OSAL binary semaphore.
//!
//! # Design
//!
//! [`AsyncMutex<T>`] uses a binary semaphore (max=1, initial=1 — starts
//! unlocked) instead of `os::Mutex`.  This avoids a circular dependency:
//! an async mutex would need to protect a waker with a mutex, which would
//! in turn need a waker, and so on.  An OSAL semaphore breaks the cycle.
//!
//! [`AsyncMutexGuard`] releases the lock and wakes the next waiter when
//! dropped, providing RAII semantics identical to `std::sync::MutexGuard`.

use core::cell::UnsafeCell;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use alloc::sync::Arc;

use crate::os::Semaphore;
use crate::traits::SemaphoreFn;
use crate::utils::OsalRsBool;

use super::waker_slot::WakerSlot;

/// An async mutex protecting a value of type `T`.
///
/// # Fairness
///
/// This is a simple non-fair mutex: woken tasks compete to re-acquire the
/// lock, so starvation is theoretically possible under high contention.
/// For most embedded use-cases this is acceptable.
///
/// # Panics
///
/// Panics if the OS fails to allocate the internal binary semaphore.
pub struct AsyncMutex<T> {
    sem: Arc<Semaphore>,
    waker: Arc<WakerSlot>,
    data: UnsafeCell<T>,
}

// SAFETY: T is Send, and we protect access through the semaphore.
unsafe impl<T: Send> Send for AsyncMutex<T> {}
unsafe impl<T: Send> Sync for AsyncMutex<T> {}

impl<T> AsyncMutex<T> {
    /// Creates a new `AsyncMutex` protecting `value`.
    ///
    /// # Panics
    ///
    /// Panics if the underlying OSAL semaphore cannot be created.
    pub fn new(value: T) -> Self {
        let sem = Semaphore::new(1, 1)
            .expect("AsyncMutex: failed to create binary semaphore");
        Self {
            sem: Arc::new(sem),
            waker: Arc::new(WakerSlot::new()),
            data: UnsafeCell::new(value),
        }
    }

    /// Returns a `Future` that resolves to an [`AsyncMutexGuard`].
    pub fn lock(&self) -> LockFuture<'_, T> {
        LockFuture { mutex: self }
    }
}

/// RAII guard returned by [`AsyncMutex::lock`].
///
/// Releases the lock and wakes the next async waiter when dropped.
pub struct AsyncMutexGuard<'a, T> {
    mutex: &'a AsyncMutex<T>,
}

impl<'a, T> Deref for AsyncMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: we hold the semaphore, so we have exclusive access.
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for AsyncMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: we hold the semaphore, so we have exclusive mutable access.
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T> Drop for AsyncMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.sem.signal();
        self.mutex.waker.wake();
    }
}

/// Future returned by [`AsyncMutex::lock`].
pub struct LockFuture<'a, T> {
    mutex: &'a AsyncMutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = AsyncMutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Non-blocking try: wait with zero timeout.
        if self.mutex.sem.wait(Duration::ZERO) == OsalRsBool::True {
            return Poll::Ready(AsyncMutexGuard { mutex: self.mutex });
        }

        // Store waker, then double-check to close the TOCTOU window.
        self.mutex.waker.store(cx.waker());

        if self.mutex.sem.wait(Duration::ZERO) == OsalRsBool::True {
            Poll::Ready(AsyncMutexGuard { mutex: self.mutex })
        } else {
            Poll::Pending
        }
    }
}
