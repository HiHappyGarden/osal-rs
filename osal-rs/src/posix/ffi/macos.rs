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

//! macOS/libSystem-specific FFI items: opaque type sizes, constant values
//! and functions Linux does not provide. Glob re-exported by the parent
//! [`ffi`](super) module, which holds everything shared with Linux.
//!
//! Only Apple Silicon (`aarch64-apple-darwin`) is supported. Values come
//! from the macOS SDK headers (`<sys/_pthread/_pthread_types.h>`,
//! `<sys/signal.h>`, `<time.h>`, `<sys/errno.h>`, `<unistd.h>`).

use core::ffi::{c_char, c_int, c_long, c_uint};

use super::{clock_gettime, pthread_cond_t, pthread_condattr_t, pthread_mutex_t, timespec};

#[cfg(not(target_arch = "aarch64"))]
compile_error!("osal-rs: on macOS the `posix` backend supports only Apple Silicon (aarch64)");

/// Size (in bytes) of `pthread_attr_t`: an 8-byte `__sig` signature
/// followed by `__PTHREAD_ATTR_SIZE__` (56) opaque bytes.
pub(in crate::posix) const PTHREAD_ATTR_T_SIZE: usize = 64;

/// Size (in bytes) of `pthread_mutex_t`: `__sig` + `__PTHREAD_MUTEX_SIZE__`
/// (56) opaque bytes. An initialized mutex always has a non-zero `__sig`.
pub(in crate::posix) const PTHREAD_MUTEX_T_SIZE: usize = 64;

/// Size (in bytes) of `pthread_mutexattr_t`: `__sig` +
/// `__PTHREAD_MUTEXATTR_SIZE__` (8) opaque bytes.
pub(in crate::posix) const PTHREAD_MUTEXATTR_T_SIZE: usize = 16;

/// Size (in bytes) of `pthread_cond_t`: `__sig` + `__PTHREAD_COND_SIZE__`
/// (40) opaque bytes.
pub(in crate::posix) const PTHREAD_COND_T_SIZE: usize = 48;

/// Size (in bytes) of `pthread_condattr_t`: `__sig` +
/// `__PTHREAD_CONDATTR_SIZE__` (8) opaque bytes.
pub(in crate::posix) const PTHREAD_CONDATTR_T_SIZE: usize = 16;

/// Signal set (`sigset_t`): a plain `__uint32_t` on macOS. Only its
/// address is handed to `sig*set`/`sigsuspend`.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(in crate::posix) struct sigset_t {
    _opaque: u32,
}

/// Mirrors macOS's user-space `struct sigaction` (`<sys/signal.h>`).
#[repr(C)]
#[derive(Copy, Clone)]
pub(in crate::posix) struct sigaction {
    sa_handler: super::sighandler_t,
    sa_mask: sigset_t,
    sa_flags: c_int,
}

impl sigaction {
    /// Action running `handler` with the signals in `mask` blocked (on top
    /// of the one being handled), restarting interrupted system calls
    /// (`SA_RESTART`, the same semantics as `signal(2)`).
    pub(in crate::posix) fn new(handler: super::sighandler_t, mask: sigset_t) -> Self {
        Self { sa_handler: handler, sa_mask: mask, sa_flags: SA_RESTART }
    }
}

/// `sigaction` flag: restart system calls interrupted by the handler
/// (`<sys/signal.h>`).
pub(in crate::posix) const SA_RESTART: c_int = 0x0002;

/// Minimum stack size (in bytes) for a thread (`PTHREAD_STACK_MIN`,
/// `<limits.h>`). macOS also requires the stack size to be a multiple of
/// the page size (16 KiB on Apple Silicon).
pub(in crate::posix) const PTHREAD_STACK_MIN: usize = 16384;

/// One-time initialization control (`pthread_once_t`): unlike glibc's plain
/// `int`, macOS uses a `__sig` signature followed by
/// `__PTHREAD_ONCE_SIZE__` (8) opaque bytes.
#[repr(C)]
pub(in crate::posix) struct pthread_once_t {
    __sig: c_long,
    __opaque: [c_char; 8],
}

/// Initial value for a [`pthread_once_t`] (`PTHREAD_ONCE_INIT`):
/// `{_PTHREAD_ONCE_SIG_init, {0}}`.
pub(in crate::posix) const PTHREAD_ONCE_INIT: pthread_once_t = pthread_once_t {
    __sig: 0x30B1BCBA,
    __opaque: [0; 8],
};

/// Scheduling policy: real-time first-in-first-out (`<sched.h>`).
#[cfg(feature = "real_time")]
pub(in crate::posix) const SCHED_FIFO: c_int = 4;

/// Scheduling-inheritance attribute: use the policy/priority set on the
/// `pthread_attr_t` itself instead of inheriting the creating thread's.
#[cfg(feature = "real_time")]
pub(in crate::posix) const PTHREAD_EXPLICIT_SCHED: c_int = 2;

/// Mirrors macOS's `struct sched_param` (`<sys/_pthread/_pthread_types.h>`),
/// which pads `sched_priority` with 4 opaque bytes (8 bytes total, versus
/// glibc's 4).
///
/// `sched_get_priority_min`/`max(SCHED_FIFO)` report 15..=47, but macOS
/// accepts other values too (verified: 1 and 99 both create the thread),
/// and grants `SCHED_FIFO` to unprivileged processes.
#[cfg(feature = "real_time")]
#[repr(C)]
#[derive(Copy, Clone)]
pub(in crate::posix) struct sched_param {
    sched_priority: c_int,
    __opaque: [c_char; 4],
}

#[cfg(feature = "real_time")]
impl sched_param {
    /// Scheduling parameters requesting `priority`.
    pub(in crate::posix) fn new(priority: c_int) -> Self {
        Self { sched_priority: priority, __opaque: [0; 4] }
    }
}

/// `sysconf(3)` parameter name for the page size (`<unistd.h>`).
pub(in crate::posix) const _SC_PAGESIZE: c_int = 29;

/// Clock identifier for `clock_gettime(2)`: time since an unspecified
/// starting point that never jumps backward or with wall-clock adjustments
/// (`<time.h>`, `_CLOCK_MONOTONIC`).
pub(in crate::posix) const CLOCK_MONOTONIC: c_int = 6;

/// `errno` value for a timed-out wait (`<sys/errno.h>`).
pub(in crate::posix) const ETIMEDOUT: c_int = 60;

/// `pthread_sigmask(3)` operation: add the signals in `set` to the
/// caller's blocked mask.
pub(in crate::posix) const SIG_BLOCK: c_int = 1;

/// `pthread_sigmask(3)` operation: replace the caller's blocked mask with
/// `set`.
pub(in crate::posix) const SIG_SETMASK: c_int = 3;

/// Mutex type: the owning thread may lock it again without deadlocking,
/// as long as it unlocks it the same number of times (`<pthread.h>`).
pub(in crate::posix) const PTHREAD_MUTEX_RECURSIVE: c_int = 2;

/// User-defined signal 1 (`<sys/signal.h>`). macOS has no real-time
/// signals (`SIGRTMIN`), so thread suspend uses this instead.
pub(in crate::posix) const SIGUSR1: c_int = 30;

/// User-defined signal 2 (`<sys/signal.h>`). macOS has no real-time
/// signals (`SIGRTMIN`), so thread resume uses this instead.
pub(in crate::posix) const SIGUSR2: c_int = 31;

/// Mach port name (`mach_port_t`, `<mach/port.h>`): a 32-bit integer.
type mach_port_t = c_uint;

/// Mach call status (`kern_return_t`); [`KERN_SUCCESS`] is 0.
type kern_return_t = c_int;

/// Successful [`kern_return_t`] (`<mach/kern_return.h>`).
const KERN_SUCCESS: kern_return_t = 0;

/// `host_statistics64` flavor returning a `vm_statistics64` (`<mach/host_info.h>`).
const HOST_VM_INFO64: c_int = 4;

/// Size of `vm_statistics64_data_t` in 32-bit words (`HOST_VM_INFO64_COUNT`).
pub(in crate::posix) const HOST_VM_INFO64_COUNT: c_uint = 104;

/// Opaque storage for `vm_statistics64_data_t` (`<mach/vm_statistics.h>`).
/// Only its first word, `free_count` (free pages), is read.
#[repr(C, align(8))]
struct vm_statistics64 {
    words: [c_uint; HOST_VM_INFO64_COUNT as usize],
}

unsafe extern "C" {

    /// The calling task's own port (what the `mach_task_self()` macro reads).
    static mach_task_self_: mach_port_t;

    /// Return a send right to the host port (`mach_host_self(3)`); the
    /// caller must release it with [`mach_port_deallocate`].
    fn mach_host_self() -> mach_port_t;

    /// Release a user reference on `name` in `task`'s port space.
    fn mach_port_deallocate(task: mach_port_t, name: mach_port_t) -> kern_return_t;

    /// Fill `info` with host statistics of the given `flavor`; `count` is
    /// the buffer size in 32-bit words on input and the size written on
    /// output.
    fn host_statistics64(host: mach_port_t, flavor: c_int, info: *mut vm_statistics64, count: *mut c_uint) -> kern_return_t;

    /// Set the name (`<= 63` chars + NUL) of the *calling* thread. Unlike
    /// glibc's variant, macOS's cannot name another thread.
    fn pthread_setname_np(name: *const c_char) -> c_int;

    /// Atomically unlock `mutex` and block on `cond` until signaled or until
    /// `reltime` (relative to now) elapses, then re-lock `mutex`. Unlike
    /// `pthread_cond_timedwait`, whose absolute deadline is always
    /// `CLOCK_REALTIME` on macOS, it is unaffected by wall-clock changes.
    fn pthread_cond_timedwait_relative_np(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t, reltime: *const timespec) -> c_int;
}

/// No-op on macOS: it has no `pthread_condattr_setclock`, so the monotonic
/// clock is honoured at wait time instead, by
/// [`cond_timedwait_monotonic`] turning the deadline into a relative timeout.
///
/// # Safety
///
/// Trivially safe; `unsafe` only to match the Linux signature.
pub(in crate::posix) unsafe fn condattr_set_monotonic(_attr: *mut pthread_condattr_t) {}

/// Atomically unlocks `mutex` and blocks on `cond` until it is signaled or
/// the monotonic `deadline` (from [`monotonic_deadline`](super::monotonic_deadline))
/// passes, then re-locks `mutex`. Returns [`ETIMEDOUT`] once the deadline
/// has passed; may also return spuriously, so callers re-check in a loop.
///
/// The time left is recomputed from `CLOCK_MONOTONIC` on every call, so a
/// caller looping over spurious wake-ups never waits past `deadline`.
///
/// # Safety
///
/// `cond` must have been initialized by
/// [`cond_init_monotonic`](super::cond_init_monotonic) and `mutex` must be
/// locked by the calling thread.
pub(in crate::posix) unsafe fn cond_timedwait_monotonic(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t, deadline: &timespec) -> c_int {
    let mut now = timespec::default();
    unsafe {
        clock_gettime(CLOCK_MONOTONIC, &mut now);
    }

    let remaining_ns = (deadline.tv_sec - now.tv_sec) as i64 * 1_000_000_000 + (deadline.tv_nsec - now.tv_nsec) as i64;
    if remaining_ns <= 0 {
        return ETIMEDOUT;
    }

    let reltime = timespec {
        tv_sec: (remaining_ns / 1_000_000_000) as c_long,
        tv_nsec: (remaining_ns % 1_000_000_000) as c_long,
    };

    unsafe { pthread_cond_timedwait_relative_np(cond, mutex, &reltime) }
}

/// Signal sent to a thread to ask it to suspend itself. macOS has no
/// real-time signals, so the backend reserves [`SIGUSR1`] for this.
pub(in crate::posix) fn suspend_signal() -> c_int {
    SIGUSR1
}

/// Signal sent to a suspended thread to wake it back up. macOS has no
/// real-time signals, so the backend reserves [`SIGUSR2`] for this.
pub(in crate::posix) fn resume_signal() -> c_int {
    SIGUSR2
}

/// Names the calling thread (`<= 63` chars + NUL).
///
/// # Safety
///
/// `name` must point to a NUL-terminated string.
pub(in crate::posix) unsafe fn set_current_thread_name(name: *const c_char) -> c_int {
    unsafe { pthread_setname_np(name) }
}

/// Number of physical memory pages currently free (`free_count` of
/// `host_statistics64(HOST_VM_INFO64)`, what `vm_stat` reports as
/// "Pages free"), or a value `<= 0` if it cannot be determined. macOS has
/// no `sysconf(_SC_AVPHYS_PAGES)`.
pub(in crate::posix) fn available_physical_pages() -> c_long {
    let mut stats = vm_statistics64 { words: [0; HOST_VM_INFO64_COUNT as usize] };
    let mut count = HOST_VM_INFO64_COUNT;

    unsafe {
        let host = mach_host_self();
        let ret = host_statistics64(host, HOST_VM_INFO64, &mut stats, &mut count);
        mach_port_deallocate(mach_task_self_, host);

        if ret != KERN_SUCCESS {
            return 0;
        }
    }

    stats.words[0] as c_long
}
