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

//! Single-slot, atomic waker storage.
//!
//! [`WakerSlot`] stores at most one [`Waker`] at a time using an
//! `AtomicPtr<Waker>`.  It is `Send + Sync` and does not require an OS mutex.
//!
//! # Why not `os::Mutex<Option<Waker>>`?
//!
//! Using an OS mutex to protect the waker inside an async primitive would
//! create a circular dependency: the async mutex *itself* would need a waker
//! to notify waiters, but storing that waker would require the mutex.
//! `AtomicPtr` avoids this entirely.

use core::sync::atomic::{AtomicPtr, Ordering};
use core::task::Waker;

use alloc::boxed::Box;

/// A single-slot, thread-safe storage for a [`Waker`].
///
/// At most one waker is stored at any given time.  A new call to [`store`]
/// replaces the previous waker (the old one is dropped).
///
/// [`store`]: WakerSlot::store
pub struct WakerSlot(AtomicPtr<Waker>);

impl WakerSlot {
    /// Creates an empty slot.
    pub const fn new() -> Self {
        Self(AtomicPtr::new(core::ptr::null_mut()))
    }

    /// Stores `waker`, replacing and dropping any previously stored waker.
    ///
    /// This is called by async futures from their `poll` implementation.
    pub fn store(&self, waker: &Waker) {
        let new_ptr = Box::into_raw(Box::new(waker.clone()));
        let old_ptr = self.0.swap(new_ptr, Ordering::AcqRel);
        if !old_ptr.is_null() {
            // SAFETY: old_ptr was created by Box::into_raw in a previous store.
            unsafe { drop(Box::from_raw(old_ptr)) };
        }
    }

    /// Wakes and removes the stored waker, if any.
    ///
    /// This is called by signalling paths (e.g. after posting to a queue)
    /// to notify the task waiting on the corresponding future.
    pub fn wake(&self) {
        let ptr = self.0.swap(core::ptr::null_mut(), Ordering::AcqRel);
        if !ptr.is_null() {
            // SAFETY: ptr was created by Box::into_raw in store().
            let waker = unsafe { Box::from_raw(ptr) };
            waker.wake();
        }
    }
}

impl Default for WakerSlot {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: WakerSlot only holds a Box<Waker> behind an AtomicPtr; all
// mutations go through swap which provides the necessary memory ordering.
unsafe impl Send for WakerSlot {}
unsafe impl Sync for WakerSlot {}

impl Drop for WakerSlot {
    fn drop(&mut self) {
        // Clean up any waker that was never consumed.
        let ptr = *self.0.get_mut();
        if !ptr.is_null() {
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }
}
