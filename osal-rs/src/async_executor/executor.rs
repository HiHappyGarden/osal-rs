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

//! Single-future executor that blocks the calling RTOS thread.
//!
//! # Design
//!
//! `block_on` creates a binary semaphore (max=1, initial=0) and builds a
//! [`Waker`] around it (see [`waker_from_semaphore`]).  It polls the future
//! in a loop:
//!
//! - [`Poll::Ready`] — returns the value immediately.
//! - [`Poll::Pending`] — waits on the semaphore indefinitely; when the waker
//!   signals the semaphore the loop polls again.
//!
//! This runs entirely on the calling thread; no thread pool is involved.

use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll};

use alloc::sync::Arc;

use crate::os::Semaphore;
use crate::traits::SemaphoreFn;
use crate::utils::MAX_DELAY;

use super::waker::waker_from_semaphore;

/// Runs `future` to completion on the calling thread, blocking until done.
///
/// This is the backend-agnostic entry point for driving async code without
/// Tokio.  It may be called from any RTOS task; each call creates its own
/// private waker semaphore.
///
/// # Panics
///
/// Panics if the OS fails to allocate the internal synchronisation semaphore.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::block_on;
///
/// let result = block_on(async { 42_u32 });
/// assert_eq!(result, 42);
/// ```
pub fn block_on<F: Future>(future: F) -> F::Output {
    // Binary semaphore: max=1, starts empty.  The executor waits on it;
    // the waker signals it when the future needs to be re-polled.
    let sem = Arc::new(
        Semaphore::new(1, 0).expect("block_on: failed to create waker semaphore"),
    );

    let waker = waker_from_semaphore(Arc::clone(&sem));
    let mut cx = Context::from_waker(&waker);

    let mut future = pin!(future);

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                sem.wait(MAX_DELAY);
            }
        }
    }
}
