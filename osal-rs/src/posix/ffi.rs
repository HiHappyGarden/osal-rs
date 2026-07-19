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

//! Foreign Function Interface (FFI) bindings for POSIX threads (pthreads).
//!
//! This module provides raw FFI declarations for the pthread functions that
//! back the safe Rust wrappers in the rest of the `posix` module. It talks
//! directly to the platform's `libpthread`/`libc` with hand-written
//! declarations — no `libc` crate, no `bindgen`, nothing external.
//!
//! # Safety
//!
//! All items in this module are `unsafe` and require careful handling:
//! - `attr` pointers must reference a validly sized/aligned [`pthread_attr_t`]
//! - `thread` pointers must be valid for writes of a [`pthread_t`]
//! - `start_routine`/`arg` must satisfy the same contract as `pthread_create(3)`
//!
//! Use the safe wrappers in parent modules instead of calling these directly.

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_long, c_void};

use crate::os::types::ThreadHandle;

// Size (in bytes) of glibc's opaque `pthread_attr_t`, taken from
// `bits/pthreadtypes-arch.h` (`__SIZEOF_PTHREAD_ATTR_T`) for each
// architecture this crate supports (same architecture set as
// `osal_rs_build::TypeGenerator::generate_types`):
// - 64-bit: x86_64/amd64, aarch64/arm64, riscv64
// - 32-bit: i586/i686, armv7l/armv6l/arm, riscv32
#[cfg(target_arch = "x86_64")]
const PTHREAD_ATTR_T_SIZE: usize = 56;
#[cfg(target_arch = "x86")]
const PTHREAD_ATTR_T_SIZE: usize = 36;
#[cfg(target_arch = "aarch64")]
const PTHREAD_ATTR_T_SIZE: usize = 64;
#[cfg(target_arch = "arm")]
const PTHREAD_ATTR_T_SIZE: usize = 36;
#[cfg(target_arch = "riscv64")]
const PTHREAD_ATTR_T_SIZE: usize = 56;
#[cfg(target_arch = "riscv32")]
const PTHREAD_ATTR_T_SIZE: usize = 32;

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv64",
    target_arch = "riscv32",
)))]
compile_error!(
    "osal-rs: pthread_attr_t layout is not known for this target_arch; add its \
     bits/pthreadtypes-arch.h __SIZEOF_PTHREAD_ATTR_T value to posix/ffi.rs"
);

/// Opaque storage for `pthread_attr_t`.
///
/// Rust never reads/writes its fields directly; only its address is handed
/// to `pthread_attr_*`/`pthread_create`, so a correctly sized-and-aligned
/// byte buffer is a valid stand-in for the real glibc struct.
#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub(super) struct pthread_attr_t {
    _opaque: [u8; PTHREAD_ATTR_T_SIZE],
}

impl Default for pthread_attr_t {
    fn default() -> Self {
        Self {
            _opaque: [0u8; PTHREAD_ATTR_T_SIZE],
        }
    }
}

/// Thread entry point matching the C signature `void *(*)(void *)`.
pub(super) type ThreadStartRoutine = unsafe extern "C" fn(arg: *mut c_void) -> *mut c_void;

/// Size (in bytes) of glibc's `sigset_t` (`bits/sigset.h`).
///
/// Unlike [`pthread_attr_t`], this is the same on every architecture: glibc
/// defines it as `1024 / (8 * sizeof(unsigned long int))` words, which is
/// 128 bytes whether `sizeof(unsigned long)` is 4 (32-bit targets) or 8
/// (64-bit targets).
const SIGSET_T_SIZE: usize = 128;

/// Opaque storage for `sigset_t`.
///
/// As with [`pthread_attr_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `sig*set`/`sigsuspend`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real glibc
/// struct.
#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub(super) struct sigset_t {
    _opaque: [u8; SIGSET_T_SIZE],
}

impl Default for sigset_t {
    fn default() -> Self {
        Self {
            _opaque: [0u8; SIGSET_T_SIZE],
        }
    }
}

/// Signal handler function pointer, as accepted/returned by `signal(2)`.
///
/// Represented as `usize` rather than a Rust fn-pointer type (mirroring the
/// `libc` crate's `sighandler_t`) so `SIG_DFL`/`SIG_IGN`/`SIG_ERR` — which
/// are sentinel integer values, not real code addresses — round-trip
/// without needing an `Option<fn>` niche that only fits a null handler.
pub(super) type sighandler_t = usize;

/// Scheduling policy: real-time first-in-first-out (`<sched.h>`).
///
/// Value is stable across every glibc-supported architecture (defined in
/// the generic `bits/sched.h`, not per-arch).
#[cfg(feature = "sched_fifo")]
pub(super) const SCHED_FIFO: c_int = 1;

/// Scheduling-inheritance attribute: use the policy/priority set on the
/// `pthread_attr_t` itself instead of inheriting the creating thread's.
#[cfg(feature = "sched_fifo")]
pub(super) const PTHREAD_EXPLICIT_SCHED: c_int = 1;

/// Mirrors glibc's `struct sched_param` (`<bits/sched.h>`), which on Linux
/// has no fields beyond `sched_priority`.
#[cfg(feature = "sched_fifo")]
#[repr(C)]
#[derive(Copy, Clone)]
pub(super) struct sched_param {
    pub(super) sched_priority: c_int,
}

/// `sysconf(3)` parameter names (`<bits/confname.h>`). Like [`SIGRTMIN`'s
/// accessor](__libc_current_sigrtmin), these are glibc's own generic
/// namespace, stable across every architecture it supports — not a
/// kernel/arch ABI detail.
pub(super) const _SC_PAGESIZE: c_int = 30;
pub(super) const _SC_AVPHYS_PAGES: c_int = 86;

unsafe extern "C" {


    pub(super) fn get_pthread_stack_min() -> usize;

    /// Initialize a thread attributes object with default values.
    pub(super) fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int;

    /// Set the stack size attribute, in bytes.
    pub(super) fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: usize) -> c_int;

    /// Set whether a thread inherits the creating thread's scheduling
    /// policy/priority (`PTHREAD_INHERIT_SCHED`) or uses `attr`'s own,
    /// explicitly-set policy/priority (`PTHREAD_EXPLICIT_SCHED`).
    #[cfg(feature = "sched_fifo")]
    pub(super) fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: c_int) -> c_int;

    /// Set the scheduling policy attribute (e.g. [`SCHED_FIFO`]).
    #[cfg(feature = "sched_fifo")]
    pub(super) fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: c_int) -> c_int;

    /// Set the scheduling parameters (priority) attribute.
    #[cfg(feature = "sched_fifo")]
    pub(super) fn pthread_attr_setschedparam(attr: *mut pthread_attr_t, param: *const sched_param) -> c_int;

    /// Create a new thread running `start_routine(arg)`, writing its ID to `thread`.
    pub(super) fn pthread_create(
        thread: *mut ThreadHandle,
        attr: *const pthread_attr_t,
        start_routine: Option<ThreadStartRoutine>,
        arg: *mut c_void,
    ) -> c_int;

    /// Return the calling thread's own ID (`pthread_self(3)`).
    pub(super) fn pthread_self() -> ThreadHandle;

    /// Wait for `thread` to terminate. If `retval` is non-null, the value
    /// passed to `pthread_exit(3)` (or returned by the start routine) by the
    /// target thread is stored in `*retval`.
    pub(super) fn pthread_join(thread: ThreadHandle, retval: *mut *mut c_void) -> c_int;

    /// Set the name (glibc extension, `<= 15` chars + NUL) of an existing thread.
    pub(super) fn pthread_setname_np(thread: ThreadHandle, name: *const c_char) -> c_int;

    /// Send signal `sig` to `thread` (`pthread_kill(3)`).
    ///
    /// Used to implement [`suspend`](crate::posix::thread::Thread)/`resume`,
    /// since pthreads has no native suspend/resume API of its own.
    pub(super) fn pthread_kill(thread: ThreadHandle, sig: c_int) -> c_int;

    /// Return the first real-time signal number glibc has not reserved for
    /// its own internal use (`SIGRTMIN(3)`).
    ///
    /// The kernel's raw `SIGRTMIN` is reserved by NPTL for thread
    /// cancellation/setuid bookkeeping; glibc exposes the first
    /// application-usable one through this function rather than a fixed
    /// constant, since the number of signals it reserves is an
    /// implementation detail that could change.
    pub(super) fn __libc_current_sigrtmin() -> c_int;

    /// Set `set` to contain every signal (`sigfillset(3)`).
    pub(super) fn sigfillset(set: *mut sigset_t) -> c_int;

    /// Remove `signum` from `set` (`sigdelset(3)`).
    pub(super) fn sigdelset(set: *mut sigset_t, signum: c_int) -> c_int;

    /// Atomically replace the calling thread's signal mask with `mask` and
    /// suspend it until a signal is delivered (`sigsuspend(3)`).
    ///
    /// Always returns -1/`EINTR`; the original mask is restored once the
    /// signal's handler returns.
    pub(super) fn sigsuspend(mask: *const sigset_t) -> c_int;

    /// Install `handler` as the action for `signum`, returning the previous
    /// handler (`signal(2)`).
    pub(super) fn signal(signum: c_int, handler: sighandler_t) -> sighandler_t;

    /// Query a system configuration value (`sysconf(3)`), e.g. [`_SC_PAGESIZE`]
    /// or [`_SC_AVPHYS_PAGES`].
    pub(super) fn sysconf(name: c_int) -> c_long;
}