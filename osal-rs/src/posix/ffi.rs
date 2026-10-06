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
//! directly to the platform's C library with hand-written declarations —
//! no `libc` crate, no `bindgen`, nothing external.
//!
//! # Layout
//!
//! Everything both supported platforms share (the opaque pthread object
//! wrappers, `struct timespec`, the portable POSIX functions and the
//! constants whose values coincide) lives here. What differs — opaque type
//! sizes, constant values, and functions only one platform provides — lives
//! in a submodule selected by `target_os` and glob re-exported from here:
//!
//! - `ffi/linux.rs` - Linux/glibc
//! - `ffi/macos.rs` - macOS/libSystem (Apple Silicon only)
//!
//! Any other `target_os` fails to build.
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

use core::ffi::{c_int, c_long, c_void};
use core::time::Duration;

use crate::os::types::ThreadHandle;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(super) use linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(super) use macos::*;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("osal-rs: the `posix` backend supports only Linux and macOS");

/// Opaque storage for `pthread_attr_t`.
///
/// Rust never reads/writes its fields directly; only its address is handed
/// to `pthread_attr_*`/`pthread_create`, so a correctly sized-and-aligned
/// byte buffer is a valid stand-in for the real C struct. The size comes
/// from the platform submodule (`PTHREAD_ATTR_T_SIZE`).
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

/// Opaque storage for `pthread_mutex_t`.
///
/// As with [`pthread_attr_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_mutex_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real C struct.
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
    /// `true` while the buffer is still all zeroes, i.e. before
    /// [`pthread_mutex_init`] ran on it. Valid on both platforms: glibc's
    /// initialized mutexes are not all-zero once a type/protocol attribute
    /// is set, and macOS's always carry a non-zero `__sig` signature.
    pub(super) fn is_empty(&self) -> bool {
        self._opaque.iter().all(|&b| b == 0)
    }
}

/// Opaque storage for `pthread_mutexattr_t`.
///
/// As with [`pthread_attr_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_mutexattr_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real C struct.
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

/// Opaque storage for `pthread_cond_t`.
///
/// As with [`pthread_mutex_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_cond_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real C struct.
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

/// Opaque storage for `pthread_condattr_t`.
///
/// As with [`pthread_mutex_t`], Rust never reads/writes its fields directly;
/// only its address is handed to `pthread_condattr_*`, so a correctly
/// sized-and-aligned byte buffer is a valid stand-in for the real C struct.
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

/// One-time initialization routine, as accepted by `pthread_once(3)`.
pub(super) type PthreadOnceRoutine = unsafe extern "C" fn();

/// Thread entry point matching the C signature `void *(*)(void *)`.
pub(super) type ThreadStartRoutine = unsafe extern "C" fn(arg: *mut c_void) -> *mut c_void;

/// Signal handler function pointer, as accepted/returned by `signal(2)`.
///
/// Represented as `usize` rather than a Rust fn-pointer type (mirroring the
/// `libc` crate's `sighandler_t`) so `SIG_DFL`/`SIG_IGN`/`SIG_ERR` — which
/// are sentinel integer values, not real code addresses — round-trip
/// without needing an `Option<fn>` niche that only fits a null handler.
pub(super) type sighandler_t = usize;

/// Mirrors `struct timespec` (`<time.h>`).
///
/// Both glibc and macOS declare both fields as `long` (`time_t` is `long`
/// on every target this crate supports), which matches the architecture's
/// native word size.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(super) struct timespec {
    pub(super) tv_sec: c_long,
    pub(super) tv_nsec: c_long,
}

/// `errno` value for an operation the caller lacks the privilege for
/// (`EPERM`), e.g. `pthread_create` asking for `SCHED_FIFO` without
/// `CAP_SYS_NICE`/`RLIMIT_RTPRIO` on Linux. Value 1 on Linux and macOS alike.
#[cfg(feature = "real_time")]
pub(super) const EPERM: c_int = 1;

/// Interrupt signal (`SIGINT`). Value 2 on Linux and macOS alike.
pub(super) const SIGINT: c_int = 2;

/// Termination signal (`SIGTERM`). Value 15 on Linux and macOS alike.
pub(super) const SIGTERM: c_int = 15;

/// Mutex protocol: a thread holding the mutex has its priority temporarily
/// raised to that of the highest-priority thread blocked on it, preventing
/// priority inversion (`<pthread.h>`, `pthread_mutexattr_setprotocol(3)`).
/// Value 1 on Linux and macOS alike.
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

    /// Lowest priority valid for scheduling `policy` (`sched_get_priority_min(2)`):
    /// 1 for `SCHED_FIFO` on Linux, 15 on macOS.
    #[cfg(feature = "real_time")]
    pub(super) fn sched_get_priority_min(policy: c_int) -> c_int;

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

    pub(super) fn pthread_detach(thread: ThreadHandle) -> c_int;

    /// Send signal `sig` to `thread` (`pthread_kill(3)`).
    ///
    /// Used to implement [`suspend`](crate::posix::thread::Thread)/`resume`,
    /// since pthreads has no native suspend/resume API of its own.
    pub(super) fn pthread_kill(thread: ThreadHandle, sig: c_int) -> c_int;

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

    /// Install `act` as the action for `signum`, optionally storing the
    /// previous one in `oldact` (`sigaction(2)`). Unlike [`signal`], lets
    /// the handler run with extra signals blocked (`sa_mask`).
    pub(super) fn sigaction(signum: c_int, act: *const sigaction, oldact: *mut sigaction) -> c_int;

    /// Fetch and/or change the calling thread's blocked-signal mask
    /// (`pthread_sigmask(3)`): `how` is [`SIG_BLOCK`] (add `set`) or
    /// [`SIG_SETMASK`] (replace with `set`); `oldset` may be null.
    pub(super) fn pthread_sigmask(how: c_int, set: *const sigset_t, oldset: *mut sigset_t) -> c_int;

    /// Initialize `set` to exclude every signal (`sigemptyset(3)`).
    pub(super) fn sigemptyset(set: *mut sigset_t) -> c_int;

    /// Add `signum` to `set` (`sigaddset(3)`).
    pub(super) fn sigaddset(set: *mut sigset_t, signum: c_int) -> c_int;

    /// Query a system configuration value (`sysconf(3)`), e.g. [`_SC_PAGESIZE`].
    pub(super) fn sysconf(name: c_int) -> c_long;

    /// Get the current time of `clock_id` (e.g. [`CLOCK_MONOTONIC`]) into
    /// `tp` (`clock_gettime(2)`).
    pub(super) fn clock_gettime(clock_id: c_int, tp: *mut timespec) -> c_int;

    /// Suspend the calling thread until `req` has elapsed. If interrupted by
    /// a signal, returns -1 and (when `rem` is non-null) writes the time
    /// left to sleep to `rem` (`nanosleep(2)`).
    pub(super) fn nanosleep(req: *const timespec, rem: *mut timespec) -> c_int;

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

    /// Initialize `cond` with the attributes in `attr` (`NULL` for the
    /// implementation's defaults).
    pub(super) fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> c_int;

    /// Destroy `cond`, freeing any resources it holds.
    ///
    /// # Safety
    ///
    /// No thread may be blocked in [`pthread_cond_wait`]/[`cond_timedwait_monotonic`]
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

    /// Wake every thread currently blocked on `cond`
    /// (`pthread_cond_broadcast(3)`).
    pub(super) fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int;

}

/// Initializes `cond` for timed waits measured on the monotonic clock, as
/// [`cond_timedwait_monotonic`] and [`monotonic_deadline`] expect. The only
/// platform-specific step, binding the condvar's attribute clock, is
/// delegated to the platform submodule's `condattr_set_monotonic`.
///
/// # Safety
///
/// `cond` must be valid for writes of a [`pthread_cond_t`] and not already
/// initialized.
pub(super) unsafe fn cond_init_monotonic(cond: *mut pthread_cond_t) -> c_int {
    let mut attr = pthread_condattr_t::default();

    unsafe {
        pthread_condattr_init(&mut attr);
        condattr_set_monotonic(&mut attr);
        pthread_cond_init(cond, &attr)
    }
}

/// Computes an absolute deadline `timeout` from now on the monotonic clock,
/// for [`cond_timedwait_monotonic`].
pub(super) fn monotonic_deadline(timeout: Duration) -> timespec {
    let mut now = timespec::default();
    unsafe {
        clock_gettime(CLOCK_MONOTONIC, &mut now);
    }

    let mut tv_sec = now.tv_sec + timeout.as_secs() as c_long;
    let mut tv_nsec = now.tv_nsec + timeout.subsec_nanos() as c_long;

    if tv_nsec >= 1_000_000_000 {
        tv_sec += 1;
        tv_nsec -= 1_000_000_000;
    }

    timespec { tv_sec, tv_nsec }
}

#[cfg(test)]
mod tests {
    //! Checks the hand-written sizes and constants above (and in the
    //! platform submodule) against the real C headers, by compiling and
    //! running a small C program with `$CC` (default `cc`). A mismatch here
    //! means undefined behavior at runtime, so it must fail loudly.

    use super::*;

    use std::collections::HashMap;
    use std::process::Command;

    const PROBE: &str = r#"
#define _GNU_SOURCE
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <time.h>
#include <unistd.h>
#if defined(__APPLE__)
#include <mach/mach.h>
#endif

#define S(t) printf("sizeof %s=%ld\n", #t, (long)sizeof(t))
#define V(c) printf("%s=%ld\n", #c, (long)(c))

int main(void) {
    S(pthread_attr_t); S(pthread_mutex_t); S(pthread_mutexattr_t);
    S(pthread_cond_t); S(pthread_condattr_t); S(pthread_once_t);
    S(sigset_t); S(pthread_t); S(struct timespec); S(struct sched_param);
    S(struct sigaction); V(SA_RESTART);
    printf("alignof sigset_t=%ld\n", (long)_Alignof(sigset_t));
    V(CLOCK_MONOTONIC); V(ETIMEDOUT); V(_SC_PAGESIZE); V(SIG_BLOCK); V(SIG_SETMASK);
    V(PTHREAD_MUTEX_RECURSIVE); V(PTHREAD_PRIO_INHERIT);
    V(SCHED_FIFO); V(PTHREAD_EXPLICIT_SCHED); V(EPERM);
    V(SIGINT); V(SIGTERM);
#if defined(__linux__)
    V(_SC_AVPHYS_PAGES);
#endif
#if defined(__APPLE__)
    V(SIGUSR1); V(SIGUSR2); V(HOST_VM_INFO64_COUNT);
#endif
    return 0;
}
"#;

    /// Compiles and runs [`PROBE`], returning its `name=value` lines.
    fn c_values() -> HashMap<String, i64> {
        let dir = std::env::temp_dir().join(format!("osal_rs_ffi_layout_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create probe dir");

        let src = dir.join("probe.c");
        let exe = dir.join("probe");
        std::fs::write(&src, PROBE).expect("write probe source");

        let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
        let status = Command::new(&cc).arg(&src).arg("-o").arg(&exe).status().expect("run C compiler");
        assert!(status.success(), "{cc} failed to compile the layout probe");

        let output = Command::new(&exe).output().expect("run layout probe");
        let _ = std::fs::remove_dir_all(&dir);

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(name, value)| (name.to_string(), value.parse().expect("numeric probe value")))
            .collect()
    }

    #[test]
    fn layout_matches_c_headers() {
        let c = c_values();
        let check = |name: &str, rust: i64| assert_eq!(c[name], rust, "`{name}` differs from the C headers");

        check("sizeof pthread_attr_t", size_of::<pthread_attr_t>() as i64);
        check("sizeof pthread_mutex_t", size_of::<pthread_mutex_t>() as i64);
        check("sizeof pthread_mutexattr_t", size_of::<pthread_mutexattr_t>() as i64);
        check("sizeof pthread_cond_t", size_of::<pthread_cond_t>() as i64);
        check("sizeof pthread_condattr_t", size_of::<pthread_condattr_t>() as i64);
        check("sizeof pthread_once_t", size_of::<pthread_once_t>() as i64);
        check("sizeof sigset_t", size_of::<sigset_t>() as i64);
        check("alignof sigset_t", align_of::<sigset_t>() as i64);
        check("sizeof struct sigaction", size_of::<sigaction>() as i64);
        check("SA_RESTART", SA_RESTART as i64);
        check("sizeof pthread_t", size_of::<ThreadHandle>() as i64);
        check("sizeof struct timespec", size_of::<timespec>() as i64);
        #[cfg(feature = "real_time")]
        check("sizeof struct sched_param", size_of::<sched_param>() as i64);

        check("CLOCK_MONOTONIC", CLOCK_MONOTONIC as i64);
        check("ETIMEDOUT", ETIMEDOUT as i64);
        check("_SC_PAGESIZE", _SC_PAGESIZE as i64);
        check("SIG_BLOCK", SIG_BLOCK as i64);
        check("SIG_SETMASK", SIG_SETMASK as i64);
        check("PTHREAD_MUTEX_RECURSIVE", PTHREAD_MUTEX_RECURSIVE as i64);
        check("PTHREAD_PRIO_INHERIT", PTHREAD_PRIO_INHERIT as i64);
        #[cfg(feature = "real_time")]
        {
            check("SCHED_FIFO", SCHED_FIFO as i64);
            check("PTHREAD_EXPLICIT_SCHED", PTHREAD_EXPLICIT_SCHED as i64);
            check("EPERM", EPERM as i64);
        }
        check("SIGINT", SIGINT as i64);
        check("SIGTERM", SIGTERM as i64);

        #[cfg(target_os = "linux")]
        {
            check("_SC_AVPHYS_PAGES", _SC_AVPHYS_PAGES as i64);
        }

        #[cfg(target_os = "macos")]
        {
            check("SIGUSR1", SIGUSR1 as i64);
            check("SIGUSR2", SIGUSR2 as i64);
            check("HOST_VM_INFO64_COUNT", HOST_VM_INFO64_COUNT as i64);
        }
    }
}
