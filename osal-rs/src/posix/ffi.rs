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
//! - `thread` pointers must be valid for writes of a [`crate::os::types::ThreadHandle`]
//! - `start_routine`/`arg` must satisfy the same contract as `pthread_create(3)`
//!
//! Use the safe wrappers in parent modules instead of calling these directly.
//!
//! # No doc examples here
//!
//! This module (and everything in it) is private and unreachable from
//! outside the crate - none of it is re-exported through [`crate::os`]. Doc
//! examples are compiled as if by an external user of the crate, so no
//! runnable `# Examples` are possible here; see the safe wrappers in
//! [`crate::posix::mutex`], [`crate::posix::thread`], etc. instead, which
//! wrap these bindings and are fully doc-tested.

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

/// Minimum stack size (in bytes) glibc allows for a thread
/// (`PTHREAD_STACK_MIN`, `bits/pthread_stack_min.h`), for each architecture
/// this crate supports (same architecture set as [`PTHREAD_ATTR_T_SIZE`]).
///
/// Every supported architecture uses glibc's generic Linux value (16384)
/// except `aarch64`, which needs a larger minimum (131072) to fit its wider
/// signal frames.
#[cfg(any(target_arch = "x86_64", target_arch = "x86", target_arch = "arm", target_arch = "riscv64", target_arch = "riscv32"))]
pub(super) const PTHREAD_STACK_MIN: usize = 16384;
#[cfg(target_arch = "aarch64")]
pub(super) const PTHREAD_STACK_MIN: usize = 131072;

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

/// Size (in bytes) of glibc's opaque `pthread_mutex_t`, taken from
/// `bits/pthreadtypes-arch.h` (`__SIZEOF_PTHREAD_MUTEX_T`) for each
/// architecture this crate supports (same architecture set as
/// [`PTHREAD_ATTR_T_SIZE`]).
#[cfg(target_arch = "x86_64")]
const PTHREAD_MUTEX_T_SIZE: usize = 40;
#[cfg(target_arch = "x86")]
const PTHREAD_MUTEX_T_SIZE: usize = 24;
#[cfg(target_arch = "aarch64")]
const PTHREAD_MUTEX_T_SIZE: usize = 48;
#[cfg(target_arch = "arm")]
const PTHREAD_MUTEX_T_SIZE: usize = 24;
#[cfg(target_arch = "riscv64")]
const PTHREAD_MUTEX_T_SIZE: usize = 40;
#[cfg(target_arch = "riscv32")]
const PTHREAD_MUTEX_T_SIZE: usize = 32;

/// Opaque storage for `pthread_mutex_t`.
///
/// As with [`pthread_attr_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_mutex_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real glibc
/// struct.
#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub struct pthread_mutex_t {
    _opaque: [u8; PTHREAD_MUTEX_T_SIZE],
}

impl Default for pthread_mutex_t {
    fn default() -> Self {
        Self {
            _opaque: [0u8; PTHREAD_MUTEX_T_SIZE],
        }
    }
}

impl pthread_mutex_t {
    pub(super) fn is_empty(&self) -> bool {
        self._opaque.iter().all(|&b| b == 0)
    }
}

/// Size (in bytes) of glibc's opaque `pthread_mutexattr_t`
/// (`__SIZEOF_PTHREAD_MUTEXATTR_T`), for each architecture this crate
/// supports.
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "riscv64",
    target_arch = "riscv32",
))]
const PTHREAD_MUTEXATTR_T_SIZE: usize = 4;
#[cfg(target_arch = "aarch64")]
const PTHREAD_MUTEXATTR_T_SIZE: usize = 8;

/// Opaque storage for `pthread_mutexattr_t`.
///
/// As with [`pthread_attr_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_mutexattr_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real glibc
/// struct.
#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub(super) struct pthread_mutexattr_t {
    _opaque: [u8; PTHREAD_MUTEXATTR_T_SIZE],
}

impl Default for pthread_mutexattr_t {
    fn default() -> Self {
        Self {
            _opaque: [0u8; PTHREAD_MUTEXATTR_T_SIZE],
        }
    }
}

/// Size (in bytes) of glibc's opaque `pthread_cond_t` (`__SIZEOF_PTHREAD_COND_T`).
///
/// Unlike [`PTHREAD_MUTEX_T_SIZE`], glibc keeps this the same 48 bytes on
/// every architecture this crate supports (`bits/pthreadtypes-arch.h`).
const PTHREAD_COND_T_SIZE: usize = 48;

/// Opaque storage for `pthread_cond_t`.
///
/// As with [`pthread_mutex_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_cond_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real glibc
/// struct.
#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub(super) struct pthread_cond_t {
    _opaque: [u8; PTHREAD_COND_T_SIZE],
}

impl Default for pthread_cond_t {
    fn default() -> Self {
        Self {
            _opaque: [0u8; PTHREAD_COND_T_SIZE],
        }
    }
}

impl pthread_cond_t {
    pub(super) fn is_empty(&self) -> bool {
        self._opaque.iter().all(|&b| b == 0)
    }
}


/// Size (in bytes) of glibc's opaque `pthread_condattr_t`
/// (`__SIZEOF_PTHREAD_CONDATTR_T`), for each architecture this crate
/// supports. Follows the same per-architecture split as
/// [`PTHREAD_MUTEXATTR_T_SIZE`].
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "riscv64",
    target_arch = "riscv32",
))]
const PTHREAD_CONDATTR_T_SIZE: usize = 4;
#[cfg(target_arch = "aarch64")]
const PTHREAD_CONDATTR_T_SIZE: usize = 8;

/// Opaque storage for `pthread_condattr_t`.
///
/// As with [`pthread_mutex_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_condattr_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real glibc
/// struct.
#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub(super) struct pthread_condattr_t {
    _opaque: [u8; PTHREAD_CONDATTR_T_SIZE],
}

impl Default for pthread_condattr_t {
    fn default() -> Self {
        Self {
            _opaque: [0u8; PTHREAD_CONDATTR_T_SIZE],
        }
    }
}


/// One-time initialization control (`pthread_once_t`, `<pthread.h>`).
///
/// Unlike the opaque, per-architecture-sized `pthread_mutex_t`/`pthread_attr_t`,
/// glibc defines this as a plain `int` (`bits/pthreadtypes.h`) on every
/// architecture this crate supports.
pub(super) type pthread_once_t = c_int;

/// Initial value for a [`pthread_once_t`] (`PTHREAD_ONCE_INIT`, `<pthread.h>`).
pub(super) const PTHREAD_ONCE_INIT: pthread_once_t = 0;

/// One-time initialization routine, as accepted by `pthread_once(3)`.
pub(super) type PthreadOnceRoutine = unsafe extern "C" fn();

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
#[cfg(feature = "real_time")]
pub(super) const SCHED_FIFO: c_int = 1;

/// Scheduling-inheritance attribute: use the policy/priority set on the
/// `pthread_attr_t` itself instead of inheriting the creating thread's.
#[cfg(feature = "real_time")]
pub(super) const PTHREAD_EXPLICIT_SCHED: c_int = 1;

/// Mirrors glibc's `struct sched_param` (`<bits/sched.h>`), which on Linux
/// has no fields beyond `sched_priority`.
#[cfg(feature = "real_time")]
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

/// Clock identifier for `clock_gettime(2)`: time since an unspecified
/// starting point that never jumps backward or with wall-clock adjustments.
/// Value is glibc's generic `<bits/time.h>` namespace, stable across every
/// architecture it supports.
pub(super) const CLOCK_MONOTONIC: c_int = 1;

/// `errno` value for a timed-out wait (`<asm-generic/errno.h>`). Stable
/// across every architecture this crate supports (all use the generic Linux
/// errno numbering; only a handful of non-supported archs like mips/sparc/alpha
/// diverge).
pub(super) const ETIMEDOUT: c_int = 110;

/// Mirrors `struct timespec` (`<time.h>`).
///
/// glibc declares both fields as `long`, which matches the architecture's
/// native word size on every target this crate supports (same reasoning as
/// [`ThreadHandle`](crate::os::types::ThreadHandle)'s `c_ulong`).
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(super) struct timespec {
    pub(super) tv_sec: c_long,
    pub(super) tv_nsec: c_long,
}

/// Mirrors `struct itimerspec` (`<time.h>`): the relative interval used to
/// arm, re-arm, or (with an all-zero `it_value`) disarm a POSIX timer via
/// [`timer_settime`].
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(super) struct itimerspec {
    pub(super) it_interval: timespec,
    pub(super) it_value: timespec,
}

/// Process/thread ID type (`__pid_t`, `bits/types.h`). glibc defines this as
/// a plain `int` on every architecture it supports.
pub(super) type pid_t = c_int;

/// Opaque per-process timer identifier (`timer_t`, `bits/types/timer_t.h`).
/// glibc defines `__timer_t` as `void *`; neither the kernel nor this crate
/// ever dereferences it, only passes the opaque value back to
/// [`timer_settime`]/[`timer_delete`].
pub(super) type timer_t = *mut c_void;

/// Padding word count for glibc's `sigevent_t` union (`bits/types/sigevent_t.h`,
/// `__SIGEV_PAD_SIZE`). Together with [`sigval_t`]'s pointer-sized member,
/// this keeps [`sigevent`] at glibc's fixed `__SIGEV_MAX_SIZE` (64 bytes) on
/// every architecture this crate supports, whether `sizeof(void *)` is 4 or 8.
#[cfg(target_pointer_width = "64")]
const SIGEV_PAD_SIZE: usize = 12;
#[cfg(target_pointer_width = "32")]
const SIGEV_PAD_SIZE: usize = 13;

/// Mirrors glibc's `sigval_t` (`bits/types/sigval_t.h`): a signal-associated
/// value that is either an `int` or a pointer.
#[repr(C)]
#[derive(Copy, Clone)]
pub(super) union sigval_t {
    pub(super) sival_int: c_int,
    pub(super) sival_ptr: *mut c_void,
}

impl Default for sigval_t {
    fn default() -> Self {
        Self { sival_ptr: core::ptr::null_mut() }
    }
}

/// Mirrors the anonymous union inside glibc's `sigevent_t`
/// (`_sigev_un` in `bits/types/sigevent_t.h`). Only the `tid` member (used
/// with [`SIGEV_THREAD_ID`]) is exposed; `pad` exists solely so this union
/// has the same size as glibc's, which also carries a `_sigev_thread` member
/// (function-pointer notification, `SIGEV_THREAD`) this crate never uses.
#[repr(C)]
#[derive(Copy, Clone)]
pub(super) union sigevent_un {
    pub(super) pad: [c_int; SIGEV_PAD_SIZE],
    /// Kernel thread ID to notify, when `sigev_notify == SIGEV_THREAD_ID`.
    pub(super) tid: pid_t,
}

impl Default for sigevent_un {
    fn default() -> Self {
        Self { pad: [0; SIGEV_PAD_SIZE] }
    }
}

/// Mirrors glibc's `sigevent_t` (`bits/types/sigevent_t.h`), used to tell
/// [`timer_create`] how to notify on timer expiry. This crate only uses
/// [`SIGEV_THREAD_ID`] notification (deliver `sigev_signo` directly to the
/// thread named by `sigev_un.tid`), which is a Linux/glibc extension.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(super) struct sigevent {
    pub(super) sigev_value: sigval_t,
    pub(super) sigev_signo: c_int,
    pub(super) sigev_notify: c_int,
    pub(super) sigev_un: sigevent_un,
}

/// Notify by delivering `sigev_signo` directly to the thread whose kernel TID
/// is given in `sigev_un.tid` (`SIGEV_THREAD_ID`, a Linux/glibc extension to
/// `bits/sigevent-consts.h`, gated there behind `__USE_GNU`). Not part of
/// POSIX; lets a timer target one specific thread instead of the whole
/// process.
pub(super) const SIGEV_THREAD_ID: c_int = 4;

/// Alarm clock signal (`SIGALRM`, `bits/signum-generic.h`). Value 14 is part
/// of Linux's base (non-real-time) signal numbering, stable across every
/// architecture this crate supports.
pub(super) const SIGALRM: c_int = 14;

/// `sigprocmask(2)`/`pthread_sigmask(3)` operation: add the signals in `set`
/// to the caller's current mask (`SIG_BLOCK`). Value is stable across every
/// architecture this crate supports (defined in the generic
/// `asm-generic/signal.h`, not an arch-specific ABI detail).
pub(super) const SIG_BLOCK: c_int = 0;

/// Mutex type: the owning thread may lock it again without deadlocking,
/// as long as it unlocks it the same number of times (`<pthread.h>`,
/// `pthread_mutexattr_settype(3)`). Value is glibc's generic namespace,
/// stable across every architecture it supports.
pub(super) const PTHREAD_MUTEX_RECURSIVE: c_int = 1;

/// Mutex protocol: a thread holding the mutex has its priority temporarily
/// raised to that of the highest-priority thread blocked on it, preventing
/// priority inversion (`<pthread.h>`, `pthread_mutexattr_setprotocol(3)`).
/// Value is glibc's generic namespace, stable across every architecture it
/// supports.
pub(super) const PTHREAD_PRIO_INHERIT: c_int = 1;

unsafe extern "C" {

    /// Initialize a thread attributes object with default values.
    pub(super) fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int;

    /// Set the stack size attribute, in bytes.
    pub(super) fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: usize) -> c_int;

    /// Set whether a thread inherits the creating thread's scheduling
    /// policy/priority (`PTHREAD_INHERIT_SCHED`) or uses `attr`'s own,
    /// explicitly-set policy/priority (`PTHREAD_EXPLICIT_SCHED`).
    #[cfg(feature = "real_time")]
    pub(super) fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: c_int) -> c_int;

    /// Set the scheduling policy attribute (e.g. [`SCHED_FIFO`]).
    #[cfg(feature = "real_time")]
    pub(super) fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: c_int) -> c_int;

    /// Set the scheduling parameters (priority) attribute.
    #[cfg(feature = "real_time")]
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

    /// Get the current time of `clock_id` (e.g. [`CLOCK_MONOTONIC`]) into
    /// `tp` (`clock_gettime(2)`).
    pub(super) fn clock_gettime(clock_id: c_int, tp: *mut timespec) -> c_int;

    /// Suspend the calling thread until `req` has elapsed. If interrupted by
    /// a signal, returns -1 and (when `rem` is non-null) writes the time
    /// left to sleep to `rem` (`nanosleep(2)`).
    pub(super) fn nanosleep(req: *const timespec, rem: *mut timespec) -> c_int;

    /// Like [`nanosleep`], but sleeps against a specific `clock_id` (e.g.
    /// [`CLOCK_MONOTONIC`]) and, when `flags` is `0`, treats `req` as
    /// relative (`TIMER_ABSTIME` would make it absolute) (`clock_nanosleep(2)`).
    pub(super) fn clock_nanosleep(clock_id: c_int, flags: c_int, req: *const timespec, rem: *mut timespec) -> c_int;

    /// Relinquish the processor to another thread ready to run, without
    /// blocking the caller (`sched_yield(2)`).
    pub(super) fn sched_yield() -> c_int;

    /// Initialize a mutex attributes object with default values.
    pub(super) fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int;

    /// Set the mutex type attribute (e.g. [`PTHREAD_MUTEX_RECURSIVE`]).
    pub(super) fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, kind: c_int) -> c_int;

    /// Set the mutex priority-inheritance protocol attribute (e.g.
    /// [`PTHREAD_PRIO_INHERIT`]).
    pub(super) fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: c_int) -> c_int;

    /// Initialize `mutex` with the attributes in `attr` (`NULL` for the
    /// implementation's defaults).
    pub(super) fn pthread_mutex_init(mutex: *mut pthread_mutex_t, attr: *const pthread_mutexattr_t) -> c_int;

    /// Destroy `mutex`, freeing any resources it holds.
    ///
    /// # Safety
    ///
    /// `mutex` must not be locked, and must not be used again afterwards.
    pub(super) fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int;

    /// Lock `mutex`, blocking the calling thread until it becomes available.
    pub(super) fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int;

    /// Attempt to lock `mutex` without blocking; fails with `EBUSY` if it is
    /// already held by another thread.
    pub(super) fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int;

    /// Unlock `mutex`. Must be called by the thread that currently holds it.
    pub(super) fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int;

    /// Run `init_routine` exactly once for the process, no matter how many
    /// threads call this concurrently with the same `once_control`
    /// (`pthread_once(3)`).
    pub(super) fn pthread_once(once_control: *mut pthread_once_t, init_routine: Option<PthreadOnceRoutine>) -> c_int;

    /// Initialize a condition-variable attributes object with default values.
    pub(super) fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> c_int;

    /// Set the clock (e.g. [`CLOCK_MONOTONIC`]) against which
    /// `pthread_cond_timedwait`'s absolute deadline is measured.
    pub(super) fn pthread_condattr_setclock(attr: *mut pthread_condattr_t, clock_id: c_int) -> c_int;

    /// Initialize `cond` with the attributes in `attr` (`NULL` for the
    /// implementation's defaults).
    pub(super) fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> c_int;

    /// Destroy `cond`, freeing any resources it holds.
    ///
    /// # Safety
    ///
    /// No thread may be blocked in [`pthread_cond_wait`]/[`pthread_cond_timedwait`]
    /// on `cond`, and it must not be used again afterwards.
    pub(super) fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int;

    /// Atomically unlock `mutex` and block on `cond` until signaled, then
    /// re-lock `mutex` before returning (`pthread_cond_wait(3)`).
    ///
    /// # Safety
    ///
    /// `mutex` must be locked by the calling thread, and must be the same
    /// mutex on every call for a given `cond`.
    pub(super) fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> c_int;

    /// As [`pthread_cond_wait`], but gives up and returns `ETIMEDOUT` once
    /// the absolute deadline `abstime` (in `cond`'s attribute clock) passes
    /// (`pthread_cond_timedwait(3)`).
    ///
    /// # Safety
    ///
    /// Same as [`pthread_cond_wait`].
    pub(super) fn pthread_cond_timedwait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t, abstime: *const timespec) -> c_int;

    /// Wake every thread currently blocked on `cond`
    /// (`pthread_cond_broadcast(3)`).
    pub(super) fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int;

    /// Initialize `set` to exclude every signal (`sigemptyset(3)`).
    pub(super) fn sigemptyset(set: *mut sigset_t) -> c_int;

    /// Add `signum` to `set` (`sigaddset(3)`).
    pub(super) fn sigaddset(set: *mut sigset_t, signum: c_int) -> c_int;

    /// Fetch and/or change the calling thread's blocked-signal mask
    /// (`sigprocmask(2)`); `oldset` may be null if the previous mask isn't
    /// needed.
    ///
    /// POSIX leaves the multi-threaded behavior of `sigprocmask` unspecified
    /// — `pthread_sigmask(3)` is the portable per-thread call — but on
    /// Linux/glibc `sigprocmask` is a thin, thread-local wrapper around it,
    /// which is what [`crate::posix::timer`] relies on to give a newly
    /// created thread a mask that already has `SIGALRM` blocked.
    pub(super) fn sigprocmask(how: c_int, set: *const sigset_t, oldset: *mut sigset_t) -> c_int;

    /// Synchronously accept one pending signal from `set`, storing its
    /// number in `sig` (`sigwait(3)`). The signals in `set` must be blocked
    /// in every thread that might receive them (see [`sigprocmask`]) —
    /// `sigwait` "consumes" a pending blocked signal instead of running a
    /// handler for it. Returns 0 on success, or a positive `errno`-style
    /// error code (unlike most of this module's functions, `sigwait` does
    /// not set `errno` itself).
    pub(super) fn sigwait(set: *const sigset_t, sig: *mut c_int) -> c_int;

    /// Return the calling thread's kernel thread ID (`gettid(2)`), as
    /// opposed to [`pthread_self`]'s process-wide `pthread_t`. Needed
    /// because [`SIGEV_THREAD_ID`] notification targets a kernel TID, not a
    /// `pthread_t`.
    ///
    /// A direct glibc wrapper since glibc 2.30.
    pub(super) fn gettid() -> pid_t;

    /// Create a per-process timer that notifies as described by `sevp`,
    /// writing its ID to `timerid` (`timer_create(2)`). This crate always
    /// uses [`CLOCK_MONOTONIC`] and [`SIGEV_THREAD_ID`] notification (see
    /// [`crate::posix::timer`]).
    pub(super) fn timer_create(clockid: c_int, sevp: *const sigevent, timerid: *mut timer_t) -> c_int;

    /// Arm, re-arm, or (with an all-zero `new_value.it_value`) disarm
    /// `timerid` (`timer_settime(2)`). `flags = 0` means `new_value.it_value`
    /// is relative to now, which is all this crate uses.
    pub(super) fn timer_settime(timerid: timer_t, flags: c_int, new_value: *const itimerspec, old_value: *mut itimerspec) -> c_int;

    /// Destroy `timerid`, releasing its resources (`timer_delete(2)`). Any
    /// pending expiration notification is discarded.
    pub(super) fn timer_delete(timerid: timer_t) -> c_int;
}