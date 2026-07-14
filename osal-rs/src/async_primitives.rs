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

//! Async primitives built on OSAL synchronisation objects.
//!
//! All types are backend-agnostic: they import from `crate::os::*` and work
//! transparently with FreeRTOS or POSIX backends.
//!
//! # Available types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncQueue`]     | Queue with `fetch_async` / `post_async` |
//! | [`AsyncSemaphore`] | Semaphore with `wait_async`            |
//! | [`AsyncMutex<T>`]  | Mutex with `lock` returning a `Future` |
//!
//! # Waker storage
//!
//! Internally, all futures store their waker in a [`WakerSlot`] — a
//! lock-free, single-slot atomic waker that avoids RTOS overhead for the
//! short critical section needed to update the waker pointer.

pub(crate) mod waker_slot;

mod mutex;
mod queue;
mod semaphore;

pub use mutex::{AsyncMutex, AsyncMutexGuard, LockFuture};
pub use queue::{AsyncQueue, FetchFuture, PostFuture};
pub use semaphore::{AsyncSemaphore, WaitFuture};
pub use waker_slot::WakerSlot;
