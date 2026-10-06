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

//! Linux/glibc-specific FFI items: opaque type sizes, constant values and
//! functions macOS does not provide. Glob re-exported by the parent
//! [`ffi`](super) module, which holds everything shared with macOS.

use core::ffi::{c_char, c_int, c_long, c_ulong};

use crate::os::types::ThreadHandle;

use super::{pthread_cond_t, pthread_condattr_t, pthread_mutex_t, timespec};

// Size (in bytes) of glibc's opaque `pthread_attr_t`, taken from
// `bits/pthreadtypes-arch.h` (`__SIZEOF_PTHREAD_ATTR_T`) for each
// architecture this crate supports:
// - 64-bit: x86_64/amd64, aarch64/arm64, riscv64
// - 32-bit: i586/i686, armv7l/armv6l/arm, riscv32
#[cfg(target_arch = "x86_64")]
pub(in crate::posix) const PTHREAD_ATTR_T_SIZE: usize = 56;
#[cfg(target_arch = "x86")]
pub(in crate::posix) const PTHREAD_ATTR_T_SIZE: usize = 36;
#[cfg(target_arch = "aarch64")]
pub(in crate::posix) const PTHREAD_ATTR_T_SIZE: usize = 64;
#[cfg(target_arch = "arm")]
pub(in crate::posix) const PTHREAD_ATTR_T_SIZE: usize = 36;
#[cfg(target_arch = "riscv64")]
pub(in crate::posix) const PTHREAD_ATTR_T_SIZE: usize = 56;
#[cfg(target_arch = "riscv32")]
pub(in crate::posix) const PTHREAD_ATTR_T_SIZE: usize = 32;

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
     bits/pthreadtypes-arch.h __SIZEOF_PTHREAD_ATTR_T value to posix/ffi/linux.rs"
);

/// Minimum stack size (in bytes) glibc allows for a thread
/// (`PTHREAD_STACK_MIN`, `bits/pthread_stack_min.h`), for each architecture
/// this crate supports (same architecture set as [`PTHREAD_ATTR_T_SIZE`]).
///
/// Every supported architecture uses glibc's generic Linux value (16384)
/// except `aarch64`, which needs a larger minimum (131072) to fit its wider
/// signal frames.
#[cfg(any(target_arch = "x86_64", target_arch = "x86", target_arch = "arm", target_arch = "riscv64", target_arch = "riscv32"))]
pub(in crate::posix) const PTHREAD_STACK_MIN: usize = 16384;
#[cfg(target_arch = "aarch64")]
pub(in crate::posix) const PTHREAD_STACK_MIN: usize = 131072;

/// Size (in bytes) of glibc's opaque `pthread_mutex_t`, taken from
/// `bits/pthreadtypes-arch.h` (`__SIZEOF_PTHREAD_MUTEX_T`) for each
/// architecture this crate supports (same architecture set as
/// [`PTHREAD_ATTR_T_SIZE`]).
#[cfg(target_arch = "x86_64")]
pub(in crate::posix) const PTHREAD_MUTEX_T_SIZE: usize = 40;
#[cfg(target_arch = "x86")]
pub(in crate::posix) const PTHREAD_MUTEX_T_SIZE: usize = 24;
#[cfg(target_arch = "aarch64")]
pub(in crate::posix) const PTHREAD_MUTEX_T_SIZE: usize = 48;
#[cfg(target_arch = "arm")]
pub(in crate::posix) const PTHREAD_MUTEX_T_SIZE: usize = 24;
#[cfg(target_arch = "riscv64")]
pub(in crate::posix) const PTHREAD_MUTEX_T_SIZE: usize = 40;
#[cfg(target_arch = "riscv32")]
pub(in crate::posix) const PTHREAD_MUTEX_T_SIZE: usize = 32;

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
pub(in crate::posix) const PTHREAD_MUTEXATTR_T_SIZE: usize = 4;
#[cfg(target_arch = "aarch64")]
pub(in crate::posix) const PTHREAD_MUTEXATTR_T_SIZE: usize = 8;

/// Size (in bytes) of glibc's opaque `pthread_cond_t` (`__SIZEOF_PTHREAD_COND_T`).
///
/// Unlike [`PTHREAD_MUTEX_T_SIZE`], glibc keeps this the same 48 bytes on
/// every architecture this crate supports (`bits/pthreadtypes-arch.h`).
pub(in crate::posix) const PTHREAD_COND_T_SIZE: usize = 48;

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
pub(in crate::posix) const PTHREAD_CONDATTR_T_SIZE: usize = 4;
#[cfg(target_arch = "aarch64")]
pub(in crate::posix) const PTHREAD_CONDATTR_T_SIZE: usize = 8;

/// Signal set (`sigset_t`, `bits/types/__sigset_t.h`): glibc defines it as
/// `1024 / (8 * sizeof(unsigned long))` `unsigned long` words, i.e. 128
/// bytes aligned like `unsigned long` on both 32- and 64-bit targets. Only
/// its address is handed to `sig*set`/`sigsuspend`.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(in crate::posix) struct sigset_t {
    _opaque: [c_ulong; 1024 / (8 * size_of::<c_ulong>())],
}

/// Mirrors glibc's `struct sigaction` (`bits/sigaction.h`, the generic
/// layout every architecture this crate supports uses).
#[repr(C)]
#[derive(Copy, Clone)]
pub(in crate::posix) struct sigaction {
    sa_handler: super::sighandler_t,
    sa_mask: sigset_t,
    sa_flags: c_int,
    sa_restorer: usize,
}

impl sigaction {
    /// Action running `handler` with the signals in `mask` blocked (on top
    /// of the one being handled), restarting interrupted system calls
    /// (`SA_RESTART`, the same semantics as `signal(2)`).
    pub(in crate::posix) fn new(handler: super::sighandler_t, mask: sigset_t) -> Self {
        Self { sa_handler: handler, sa_mask: mask, sa_flags: SA_RESTART, sa_restorer: 0 }
    }
}

/// `sigaction` flag: restart system calls interrupted by the handler
/// (`bits/sigaction.h`).
pub(in crate::posix) const SA_RESTART: c_int = 0x10000000;

/// One-time initialization control (`pthread_once_t`, `<pthread.h>`).
///
/// glibc defines this as a plain `int` (`bits/pthreadtypes.h`) on every
/// architecture this crate supports.
pub(in crate::posix) type pthread_once_t = c_int;

/// Initial value for a [`pthread_once_t`] (`PTHREAD_ONCE_INIT`, `<pthread.h>`).
pub(in crate::posix) const PTHREAD_ONCE_INIT: pthread_once_t = 0;

/// Scheduling policy: real-time first-in-first-out (`<sched.h>`).
///
/// Value is stable across every glibc-supported architecture (defined in
/// the generic `bits/sched.h`, not per-arch).
#[cfg(feature = "real_time")]
pub(in crate::posix) const SCHED_FIFO: c_int = 1;

/// Scheduling-inheritance attribute: use the policy/priority set on the
/// `pthread_attr_t` itself instead of inheriting the creating thread's.
#[cfg(feature = "real_time")]
pub(in crate::posix) const PTHREAD_EXPLICIT_SCHED: c_int = 1;

/// Mirrors glibc's `struct sched_param` (`<bits/sched.h>`), which on Linux
/// has no fields beyond `sched_priority`.
#[cfg(feature = "real_time")]
#[repr(C)]
#[derive(Copy, Clone)]
pub(in crate::posix) struct sched_param {
    sched_priority: c_int,
}

#[cfg(feature = "real_time")]
impl sched_param {
    /// Scheduling parameters requesting `priority`.
    pub(in crate::posix) fn new(priority: c_int) -> Self {
        Self { sched_priority: priority }
    }
}

/// `sysconf(3)` parameter names (`<bits/confname.h>`). Like [`SIGRTMIN`'s
/// accessor](__libc_current_sigrtmin), these are glibc's own generic
/// namespace, stable across every architecture it supports — not a
/// kernel/arch ABI detail.
pub(in crate::posix) const _SC_PAGESIZE: c_int = 30;
pub(in crate::posix) const _SC_AVPHYS_PAGES: c_int = 86;

/// Clock identifier for `clock_gettime(2)`: time since an unspecified
/// starting point that never jumps backward or with wall-clock adjustments.
/// Value is glibc's generic `<bits/time.h>` namespace, stable across every
/// architecture it supports.
pub(in crate::posix) const CLOCK_MONOTONIC: c_int = 1;

/// `errno` value for a timed-out wait (`<asm-generic/errno.h>`). Stable
/// across every architecture this crate supports (all use the generic Linux
/// errno numbering; only a handful of non-supported archs like mips/sparc/alpha
/// diverge).
pub(in crate::posix) const ETIMEDOUT: c_int = 110;

/// `pthread_sigmask(3)` operation: add the signals in `set` to the
/// caller's blocked mask.
pub(in crate::posix) const SIG_BLOCK: c_int = 0;

/// `pthread_sigmask(3)` operation: replace the caller's blocked mask with
/// `set`.
pub(in crate::posix) const SIG_SETMASK: c_int = 2;

/// Mutex type: the owning thread may lock it again without deadlocking,
/// as long as it unlocks it the same number of times (`<pthread.h>`,
/// `pthread_mutexattr_settype(3)`). Value is glibc's generic namespace,
/// stable across every architecture it supports.
pub(in crate::posix) const PTHREAD_MUTEX_RECURSIVE: c_int = 1;

unsafe extern "C" {

    /// Set the name (glibc extension, `<= 15` chars + NUL) of an existing thread.
    fn pthread_setname_np(thread: ThreadHandle, name: *const c_char) -> c_int;

    /// Return the first real-time signal number glibc has not reserved for
    /// its own internal use (`SIGRTMIN(3)`).
    ///
    /// The kernel's raw `SIGRTMIN` is reserved by NPTL for thread
    /// cancellation/setuid bookkeeping; glibc exposes the first
    /// application-usable one through this function rather than a fixed
    /// constant, since the number of signals it reserves is an
    /// implementation detail that could change.
    fn __libc_current_sigrtmin() -> c_int;

    /// Set the clock (e.g. [`CLOCK_MONOTONIC`]) against which
    /// `pthread_cond_timedwait`'s absolute deadline is measured.
    fn pthread_condattr_setclock(attr: *mut pthread_condattr_t, clock_id: c_int) -> c_int;

    /// Atomically unlock `mutex` and block on `cond` until signaled or until
    /// the absolute deadline `abstime` (in `cond`'s attribute clock) passes,
    /// then re-lock `mutex` (`pthread_cond_timedwait(3)`).
    fn pthread_cond_timedwait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t, abstime: *const timespec) -> c_int;
}

/// Binds a condvar attribute to [`CLOCK_MONOTONIC`], so absolute deadlines
/// built by [`monotonic_deadline`](super::monotonic_deadline) are measured
/// on the same clock. Called by [`cond_init_monotonic`](super::cond_init_monotonic).
///
/// # Safety
///
/// `attr` must point to an initialized [`pthread_condattr_t`].
pub(in crate::posix) unsafe fn condattr_set_monotonic(attr: *mut pthread_condattr_t) {
    unsafe {
        pthread_condattr_setclock(attr, CLOCK_MONOTONIC);
    }
}

/// Atomically unlocks `mutex` and blocks on `cond` until it is signaled or
/// the monotonic `deadline` (from [`monotonic_deadline`](super::monotonic_deadline))
/// passes, then re-locks `mutex`. Returns [`ETIMEDOUT`] once the deadline
/// has passed; may also return spuriously, so callers re-check in a loop.
///
/// # Safety
///
/// `cond` must have been initialized by
/// [`cond_init_monotonic`](super::cond_init_monotonic) and `mutex` must be
/// locked by the calling thread.
pub(in crate::posix) unsafe fn cond_timedwait_monotonic(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t, deadline: &timespec) -> c_int {
    unsafe { pthread_cond_timedwait(cond, mutex, deadline) }
}

/// Signal sent to a thread to ask it to suspend itself: the first real-time
/// signal glibc leaves to applications (see [`__libc_current_sigrtmin`]).
pub(in crate::posix) fn suspend_signal() -> c_int {
    unsafe { __libc_current_sigrtmin() }
}

/// Signal sent to a suspended thread to wake it back up. Always
/// `suspend_signal() + 1`, so it lands on the next glibc-usable real-time
/// signal.
pub(in crate::posix) fn resume_signal() -> c_int {
    suspend_signal() + 1
}

/// Names the calling thread (`<= 15` chars + NUL, longer names are rejected
/// with `ERANGE`).
///
/// # Safety
///
/// `name` must point to a NUL-terminated string.
pub(in crate::posix) unsafe fn set_current_thread_name(name: *const c_char) -> c_int {
    unsafe { pthread_setname_np(super::pthread_self(), name) }
}

/// Number of physical memory pages currently free (`MemFree`), or a value
/// `<= 0` if it cannot be determined.
pub(in crate::posix) fn available_physical_pages() -> c_long {
    unsafe { super::sysconf(_SC_AVPHYS_PAGES) }
}
