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

//! Backend-agnostic Waker built on an OSAL Semaphore.
//!
//! The waker wraps an `Arc<Semaphore>` using a `RawWakerVTable`.
//! When `wake()` is called the semaphore is signalled, which unblocks
//! the executor that is waiting on it.

use core::task::{RawWaker, RawWakerVTable, Waker};
use core::mem::forget;

use alloc::sync::Arc;

use crate::os::Semaphore;
use crate::traits::SemaphoreFn;

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    clone_waker,
    wake_waker,
    wake_by_ref_waker,
    drop_waker,
);

unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
    // SAFETY: ptr was created from Arc::into_raw; incrementing is safe here.
    unsafe { Arc::increment_strong_count(&raw const ptr ) };
    RawWaker::new(ptr, &VTABLE)
}

unsafe fn wake_waker(ptr: *const ()) {
    // SAFETY: ptr was created from Arc::into_raw; this call consumes it.
    let arc = unsafe { Arc::from_raw(&raw const ptr) };
    arc.signal();
}

unsafe fn wake_by_ref_waker(ptr: *const ()) {
    // SAFETY: ptr was created from Arc::into_raw; borrowed temporarily.
    let arc = unsafe { Arc::from_raw(&raw const ptr) };
    arc.signal();
    forget(arc);
}

unsafe fn drop_waker(ptr: *const ()) {
    unsafe { drop(Arc::from_raw(&raw const ptr)) };
}

/// Creates a [`Waker`] that signals `sem` when woken.
///
/// Ownership of the `Arc` is transferred into the waker; the semaphore is
/// kept alive as long as any clone of this waker exists.
pub(crate) fn waker_from_semaphore(sem: Arc<Semaphore>) -> Waker {
    let ptr = &raw const Arc::into_raw(sem);
    // SAFETY: the vtable correctly manages the Arc refcount.
    unsafe { Waker::from_raw(RawWaker::new(ptr, &VTABLE)) }
}
