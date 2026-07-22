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

//! Task/thread creation, management, and notifications for POSIX.
//!
//! [`Thread`] wraps a pthread, adding what pthreads lacks natively but
//! FreeRTOS tasks provide directly: suspend/resume (emulated with a pair of
//! real-time signals), a single-slot notification value
//! (`notify`/`wait_notification`, backed by a process-wide table keyed by
//! thread handle), and metadata queries (`get_metadata`, backed by a similar
//! registry so [`crate::os::System`] can enumerate every thread spawned
//! through this API).
//!
//! # Examples
//!
//! ```
//! use osal_rs::os::*;
//! use std::sync::Arc;
//!
//! let mut thread = Thread::new("worker", 1024, 5);
//! let spawned = thread.spawn_simple(|| {
//!     println!("Working...");
//!     Ok(Arc::new(()))
//! }).unwrap();
//!
//! spawned.join(core::ptr::null_mut()).unwrap();
//! ```

use core::cell::UnsafeCell;
use core::ffi::{c_int, c_long, c_void};
use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null_mut;
use core::time::Duration;
use std::collections::HashMap;

use alloc::sync::Arc;

use crate::os::{Mutex, MutexFn, MutexGuard, ThreadSimpleFnPtr};
use crate::posix::config::TICK_PERIOD_MS;
#[cfg(feature = "real_time")]
use crate::posix::ffi::{PTHREAD_EXPLICIT_SCHED, SCHED_FIFO, pthread_attr_setinheritsched, pthread_attr_setschedparam, pthread_attr_setschedpolicy, sched_param};
use crate::posix::ffi::{
	__libc_current_sigrtmin, CLOCK_MONOTONIC, ETIMEDOUT, PTHREAD_ONCE_INIT, clock_gettime, osal_rs_get_pthread_stack_min, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t,
	pthread_cond_broadcast, pthread_cond_destroy, pthread_cond_init, pthread_cond_t, pthread_cond_timedwait, pthread_cond_wait, pthread_condattr_init, pthread_condattr_setclock,
	pthread_condattr_t, pthread_create, pthread_join, pthread_kill, pthread_once, pthread_once_t, pthread_self, pthread_setname_np, sigdelset, sigfillset, sigset_t, signal, sigsuspend,
	timespec,
};
use crate::posix::types::{BaseType, StackType, ThreadHandle, TickType, UBaseType};
use crate::traits::{ThreadFn, ThreadFnPtr, ThreadMetadata, ThreadNotification, ThreadParam, ThreadState, ToPriority, ToTick};
use crate::traits::MAX_TASK_NAME_LEN;
use crate::utils::{Bytes, DoublePtr, Error, Result};

/// Real-time signal sent to a thread to ask it to suspend itself; see
/// [`suspend_signal_handler`].
fn suspend_signal() -> c_int {
    unsafe { __libc_current_sigrtmin() }
}

/// Real-time signal sent to a thread parked in [`suspend_signal_handler`] to
/// wake it back up. Always `suspend_signal() + 1`, so it lands on the next
/// glibc-usable real-time signal.
fn resume_signal() -> c_int {
    suspend_signal() + 1
}

/// Handler for [`suspend_signal`]: parks the calling thread until [`resume_signal`] arrives.
///
/// pthreads has no native suspend/resume, so this crate emulates it with a
/// pair of real-time signals. `sigsuspend()` atomically swaps in a mask that
/// blocks every signal except the resume one and blocks the thread until a
/// signal is delivered; since nothing else can get through, that signal can
/// only be the resume one. When `sigsuspend()` returns, this handler returns
/// too, and the thread it interrupted simply continues from wherever it was
/// — that's what makes the suspension transparent to the thread's own code.
///
/// # Caveat
///
/// If `resume()` runs before the target thread has actually reached
/// `sigsuspend()` below (the suspend signal was sent but not yet delivered),
/// the resume signal is delivered with nothing waiting for it and is lost,
/// leaving the thread suspended until a further `resume()` call. Callers
/// needing a hard guarantee should pair `suspend()`/`resume()` with their
/// own synchronization.
extern "C" fn suspend_signal_handler(_sig: c_int) {
    let mut mask: sigset_t = Default::default();

    unsafe {
        sigfillset(&mut mask);
        sigdelset(&mut mask, resume_signal());
        sigsuspend(&mask);
    }
}

/// No-op handler for [`resume_signal`].
///
/// Its only purpose is to exist: installing a handler is what lets this
/// signal interrupt `sigsuspend()` in [`suspend_signal_handler`] instead of
/// being blocked, and — since this is a real-time signal — it avoids the
/// default action of terminating the process.
extern "C" fn resume_signal_handler(_sig: c_int) {}

/// Installs [`suspend_signal_handler`]/[`resume_signal_handler`], once per process.
fn ensure_suspend_signal_handlers() {
    static mut ONCE: pthread_once_t = PTHREAD_ONCE_INIT;

    extern "C" fn init() {
        unsafe {
            signal(suspend_signal(), suspend_signal_handler as *const () as usize);
            signal(resume_signal(), resume_signal_handler as *const () as usize);
        }
    }

    unsafe {
        pthread_once(&raw mut ONCE, Some(init));
    }
}

/// Condition variable backing [`NotifySlot`]'s wait/wake, and [`ensure_suspend_signal_handlers`]'s
/// [`pthread_once_t`]-based sibling for one-time initialization.
///
/// Backed directly by `pthread_cond_t` rather than `std::sync::Condvar`:
/// the latter's `wait`/`wait_timeout` only accept `std::sync::MutexGuard`,
/// which can't pair with [`crate::os::Mutex`]'s own guard type.
struct RawCondvar(UnsafeCell<pthread_cond_t>);

unsafe impl Send for RawCondvar {}
unsafe impl Sync for RawCondvar {}

impl RawCondvar {
    fn new() -> Self {
        let mut attr: pthread_condattr_t = Default::default();
        let mut cond: pthread_cond_t = Default::default();

        unsafe {
            pthread_condattr_init(&mut attr);
            pthread_condattr_setclock(&mut attr, CLOCK_MONOTONIC);
            pthread_cond_init(&mut cond, &attr);
        }

        Self(UnsafeCell::new(cond))
    }

    /// Atomically unlocks `guard`'s mutex and blocks until [`notify_all`](Self::notify_all)
    /// wakes it, re-locking the mutex before returning. May return spuriously;
    /// callers must re-check their predicate in a loop, same as with any condvar.
    fn wait<T: ?Sized>(&self, guard: &MutexGuard<'_, T>) {
        unsafe {
            pthread_cond_wait(self.0.get(), guard.raw_handle());
        }
    }

    /// As [`wait`](Self::wait), but gives up once the monotonic-clock `deadline`
    /// passes. Returns `true` if it gave up because of the deadline, `false` if
    /// woken normally (which, same as [`wait`](Self::wait), may be spurious).
    fn wait_until<T: ?Sized>(&self, guard: &MutexGuard<'_, T>, deadline: timespec) -> bool {
        unsafe { pthread_cond_timedwait(self.0.get(), guard.raw_handle(), &deadline) == ETIMEDOUT }
    }

    fn notify_all(&self) {
        unsafe {
            pthread_cond_broadcast(self.0.get());
        }
    }
}

impl Default for RawCondvar {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RawCondvar {
    fn drop(&mut self) {
        unsafe {
            pthread_cond_destroy(self.0.get());
        }
    }
}

/// Computes an absolute deadline `timeout` from now on the monotonic clock,
/// for [`RawCondvar::wait_until`] (its `pthread_condattr_setclock(CLOCK_MONOTONIC)`
/// counterpart to `pthread_cond_timedwait`'s absolute `abstime`).
fn monotonic_deadline(timeout: Duration) -> timespec {
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

/// A thread's pending task-notification value, plus whether one is pending.
///
/// Mirrors the single-slot notification FreeRTOS keeps directly on its task
/// control block: `pending` is what `wait_notification()` blocks on, and
/// `value` is what it hands back once woken.
#[derive(Default)]
struct NotifyState {
    value: u32,
    pending: bool,
}

/// The synchronization primitives backing one thread's notification slot.
struct NotifySlot {
    state: Mutex<NotifyState>,
    cv: RawCondvar,
}

impl Default for NotifySlot {
    fn default() -> Self {
        Self {
            state: Mutex::new(NotifyState::default()),
            cv: RawCondvar::default(),
        }
    }
}

/// Process-wide table of notification slots, keyed by `pthread_t`.
///
/// pthreads has nothing resembling FreeRTOS's per-task notification value,
/// so this crate keeps its own, addressed by thread handle rather than
/// stored on `Thread` itself — `Thread` is freely cloned, but a notification
/// belongs to the underlying OS thread, not to any one Rust handle to it.
fn notify_registry() -> &'static Mutex<HashMap<ThreadHandle, Arc<NotifySlot>>> {
    static mut ONCE: pthread_once_t = PTHREAD_ONCE_INIT;
    static mut REGISTRY: *mut Mutex<HashMap<ThreadHandle, Arc<NotifySlot>>> = null_mut();

    extern "C" fn init() {
        unsafe {
            REGISTRY = Box::into_raw(Box::new(Mutex::new(HashMap::new())));
        }
    }

    unsafe {
        pthread_once(&raw mut ONCE, Some(init));
        &*REGISTRY
    }
}

/// Returns `handle`'s notification slot, creating it on first use.
fn notify_slot(handle: ThreadHandle) -> Arc<NotifySlot> {
    notify_registry()
        .lock()
        .unwrap()
        .entry(handle)
        .or_insert_with(|| Arc::new(NotifySlot::default()))
        .clone()
}

/// Drops `handle`'s notification slot, if any.
///
/// glibc recycles `pthread_t` values once a thread has been joined, so a
/// slot left behind after that point could be silently inherited by an
/// unrelated future thread. Called once a thread is known to be gone
/// (`delete()`/`join()` returning successfully).
fn forget_notify_slot(handle: ThreadHandle) {
    if let Ok(mut registry) = notify_registry().lock() {
        registry.remove(&handle);
    }
}

/// Process-wide table of threads spawned through this crate's `Thread` API,
/// keyed by `pthread_t`.
///
/// pthreads exposes no enumeration API of its own, so `System::get_all_thread()`
/// and `System::count_threads()` are backed by this registry instead: every
/// successful `spawn()`/`spawn_simple()` adds an entry, and `join()`/`delete()`
/// remove it once the thread is known to be gone.
fn thread_registry() -> &'static Mutex<HashMap<ThreadHandle, ThreadMetadata>> {
    static mut ONCE: pthread_once_t = PTHREAD_ONCE_INIT;
    static mut REGISTRY: *mut Mutex<HashMap<ThreadHandle, ThreadMetadata>> = null_mut();

    extern "C" fn init() {
        unsafe {
            REGISTRY = Box::into_raw(Box::new(Mutex::new(HashMap::new())));
        }
    }

    unsafe {
        pthread_once(&raw mut ONCE, Some(init));
        &*REGISTRY
    }
}

/// Records `metadata` under `metadata.thread` for [`System::get_all_thread()`].
fn register_thread(metadata: ThreadMetadata) {
    if let Ok(mut registry) = thread_registry().lock() {
        registry.insert(metadata.thread, metadata);
    }
}

/// Drops `handle`'s registry entry, if any (see [`forget_notify_slot`] for why).
fn forget_thread(handle: ThreadHandle) {
    if let Ok(mut registry) = thread_registry().lock() {
        registry.remove(&handle);
    }
}

/// Updates `handle`'s tracked [`ThreadState`] in the registry, if it has an entry.
///
/// Threads not spawned through this crate's API (e.g. a foreign thread only
/// ever wrapped via [`Thread::new_with_handle`]) have no registry entry and
/// are silently ignored, same as [`forget_thread`].
fn set_thread_state(handle: ThreadHandle, state: ThreadState) {
    if let Ok(mut registry) = thread_registry().lock() {
        if let Some(metadata) = registry.get_mut(&handle) {
            metadata.state = state;
        }
    }
}

/// Returns `handle`'s registry entry, if any.
fn registered_thread_metadata(handle: ThreadHandle) -> Option<ThreadMetadata> {
    thread_registry().lock().ok().and_then(|registry| registry.get(&handle).cloned())
}

/// Resolves `handle`'s effective [`ThreadState`] for metadata queries.
///
/// A thread can only ask about its own metadata while it's actually
/// executing: `suspend()` parks the target thread inside `sigsuspend()`
/// (see [`suspend_signal_handler`]), so a genuinely suspended thread can
/// never itself reach this call. `pthread_self()` therefore always
/// overrides `tracked` with [`ThreadState::Running`]; for every other
/// handle, the last state recorded by [`set_thread_state`] is the best
/// information available.
fn effective_thread_state(handle: ThreadHandle, tracked: ThreadState) -> ThreadState {
    if handle == unsafe { pthread_self() } { ThreadState::Running } else { tracked }
}

/// Snapshot of every thread currently registered via `spawn()`/`spawn_simple()`.
///
/// Used by [`crate::posix::system::System::get_all_thread`].
pub(crate) fn all_registered_threads() -> Vec<ThreadMetadata> {
    thread_registry()
        .lock()
        .map(|registry| registry.values().cloned().collect())
        .unwrap_or_default()
}

/// Number of threads currently registered via `spawn()`/`spawn_simple()`.
///
/// Used by [`crate::posix::system::System::count_threads`].
pub(crate) fn registered_thread_count() -> usize {
    thread_registry().lock().map(|registry| registry.len()).unwrap_or(0)
}

/// Applies a [`ThreadNotification`] action to `state`, FreeRTOS-`xTaskNotify`-style.
///
/// Every action except [`ThreadNotification::SetValueWithoutOverwrite`]
/// always succeeds. That one only updates `value` if no notification is
/// currently pending (i.e. the previous one was consumed by
/// `wait_notification()`); if one is already pending, it fails without
/// touching `value`, matching FreeRTOS's `xTaskNotify(eSetValueWithoutOverwrite)`
/// returning `pdFAIL`.
fn apply_notification(state: &mut NotifyState, notification: ThreadNotification) -> Result<()> {
    use ThreadNotification::*;
    match notification {
        NoAction => {}
        SetBits(bits) => state.value |= bits,
        Increment => state.value = state.value.wrapping_add(1),
        SetValueWithOverwrite(value) => state.value = value,
        SetValueWithoutOverwrite(value) => {
            if state.pending {
                return Err(Error::QueueFull);
            }
            state.value = value;
        }
    }

    state.pending = true;
    Ok(())
}

/// A schedulable unit of execution backed by a POSIX thread (`pthread_t`).
///
/// Created in a "not yet spawned" state via [`Thread::new`], and only backed
/// by a real OS thread once [`ThreadFn::spawn`]/[`ThreadFn::spawn_simple`] is
/// called on it. See [`Thread::new`] for a complete, testable example.
#[derive(Clone)]
pub struct Thread {
    handle: ThreadHandle,
    name: Bytes<MAX_TASK_NAME_LEN>,
    stack_depth: StackType,
    priority: UBaseType,
    callback: Option<Arc<ThreadFnPtr>>,
    param: Option<ThreadParam>,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    /// Describes a not-yet-spawned thread: `name`/`stack_depth`/`priority`
    /// are recorded now and used when [`ThreadFn::spawn`]/`spawn_simple` is
    /// called on it. [`ThreadFn::is_null`] is `true` until then.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let thread = Thread::new("worker", 1024, 5);
    /// assert!(thread.is_null());
    /// ```
    pub fn new(name: &str, stack_depth: StackType, priority: UBaseType) -> Self {
        Self {
            handle: 0,
            name: Bytes::from_str(name),
            stack_depth,
            priority,
            callback: None,
            param: None,
        }
    }

    /// Wraps an already-running thread's handle (e.g. one obtained from
    /// [`ThreadFn::get_current`]), rather than spawning a new one. Fails
    /// with [`Error::NullPtr`] if `handle` is the null sentinel (`0`).
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    /// let wrapped = Thread::new_with_handle(*current, "current", 0, 0).unwrap();
    /// assert!(!wrapped.is_null());
    /// ```
    pub fn new_with_handle(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: UBaseType) -> Result<Self> {
        if handle == 0 {
            return Err(Error::NullPtr);
        }

        Ok(Self {
            handle,
            name: Bytes::from_str(name),
            stack_depth,
            priority,
            callback: None,
            param: None,
        })
    }

    /// Same as [`Thread::new`], but accepts any [`ToPriority`] value instead
    /// of a raw [`UBaseType`] priority.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::os::types::UBaseType;
    ///
    /// enum Priority { High }
    ///
    /// impl ToPriority for Priority {
    ///     fn to_priority(&self) -> UBaseType { 5 }
    /// }
    ///
    /// let thread = Thread::new_with_to_priority("worker", 1024, Priority::High);
    /// assert!(thread.is_null());
    /// ```
    #[inline]
    pub fn new_with_to_priority(name: &str, stack_depth: StackType, priority: impl ToPriority) -> Self {
        Self::new(name, stack_depth, priority.to_priority())
    }

    /// Same as [`Thread::new_with_handle`], but accepts any [`ToPriority`]
    /// value instead of a raw [`UBaseType`] priority.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use osal_rs::os::types::UBaseType;
    ///
    /// enum Priority { Normal }
    ///
    /// impl ToPriority for Priority {
    ///     fn to_priority(&self) -> UBaseType { 0 }
    /// }
    ///
    /// let current = Thread::get_current();
    /// let wrapped = Thread::new_with_handle_and_to_priority(*current, "current", 0, Priority::Normal).unwrap();
    /// assert!(!wrapped.is_null());
    /// ```
    #[inline]
    pub fn new_with_handle_and_to_priority(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: impl ToPriority) -> Result<Self> {
        Self::new_with_handle(handle, name, stack_depth, priority.to_priority())
    }

    /// Looks up a [`ThreadMetadata`] snapshot for a raw [`ThreadHandle`],
    /// without needing a [`Thread`] value. Used by
    /// [`crate::os::SystemFn::get_all_thread`] to report threads it only knows
    /// by handle.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    /// let metadata = Thread::get_metadata_from_handle(*current);
    /// assert_eq!(metadata.thread, *current);
    /// ```
    pub fn get_metadata_from_handle(handle: ThreadHandle) -> ThreadMetadata {
        if handle == 0 {
            return ThreadMetadata::default();
        }

        match registered_thread_metadata(handle) {
            Some(metadata) => ThreadMetadata {
                state: effective_thread_state(handle, metadata.state),
                ..metadata
            },
            None => ThreadMetadata {
                thread: handle,
                name: Bytes::from_str("thread"),
                stack_depth: 0,
                priority: 0,
                thread_number: 0,
                state: effective_thread_state(handle, ThreadState::Ready),
                current_priority: 0,
                base_priority: 0,
                run_time_counter: 0,
                stack_high_water_mark: 0,
            },
        }
    }

    /// Builds a [`ThreadMetadata`] snapshot from a [`Thread`] value
    /// directly - same information as [`Thread::get_metadata_from_handle`],
    /// but also works for a not-yet-spawned thread (reported as
    /// [`ThreadState::Invalid`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let thread = Thread::new("worker", 1024, 3);
    /// let metadata = Thread::get_metadata(&thread);
    /// assert_eq!(metadata.state, ThreadState::Invalid);
    /// assert_eq!(metadata.priority, 3);
    /// ```
    pub fn get_metadata(thread: &Thread) -> ThreadMetadata {
        // Name/stack/priority reflect what was passed to `new()` regardless of
        // whether the thread has been spawned yet; only `state`/`thread` depend
        // on there being a live `pthread_t` behind it.
        let state = if thread.is_null() {
            ThreadState::Invalid
        } else {
            let tracked = registered_thread_metadata(thread.handle).map(|metadata| metadata.state).unwrap_or(ThreadState::Ready);
            effective_thread_state(thread.handle, tracked)
        };

        ThreadMetadata {
            thread: thread.handle,
            name: thread.name.clone(),
            stack_depth: thread.stack_depth,
            priority: thread.priority,
            thread_number: thread.handle,
            state,
            current_priority: thread.priority,
            base_priority: thread.priority,
            run_time_counter: 0,
            stack_high_water_mark: 0,
        }
    }

    /// Blocks like [`ThreadFn::wait_notification`], but accepts any
    /// [`ToTick`] timeout (e.g. a [`core::time::Duration`]) instead of a raw
    /// tick count.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use core::time::Duration;
    ///
    /// let current = Thread::get_current();
    /// current.notify(ThreadNotification::SetValueWithOverwrite(7)).unwrap();
    ///
    /// let value = current.wait_notification_with_to_tick(0, 0, Duration::from_millis(50)).unwrap();
    /// assert_eq!(value, 7);
    /// ```
    #[inline]
    pub fn wait_notification_with_to_tick(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32, timeout_ticks: impl ToTick) -> Result<u32> {
        self.wait_notification(bits_to_clear_on_entry, bits_to_clear_on_exit, timeout_ticks.to_ticks())
    }

    fn metadata(&self) -> ThreadMetadata {
        Self::get_metadata(self)
    }
}

/// Internal C-compatible wrapper for thread callbacks.
///
/// Bridges between the pthreads C API and Rust closures. It unpacks the
/// boxed thread instance, resolves the thread's own handle via
/// `pthread_self()` (avoiding any race with `pthread_create()`'s caller
/// writing `*thread`, which may not have happened yet once this routine
/// starts running), and invokes the user-provided callback.
///
/// The callback's `Result<ThreadParam>` is boxed and returned as the raw
/// `void *` the pthreads API uses for a thread's exit value: whoever calls
/// `Thread::join()` on this thread receives this same pointer back and can
/// reconstruct it with `Box::from_raw(ptr as *mut Result<ThreadParam>)`.
///
/// # Safety
///
/// - `param_ptr` must be a valid pointer produced by `Box::into_raw` on a `Thread`
/// - Called only by `pthread_create()` as the thread's start routine
unsafe extern "C" fn callback_c_wrapper(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut();
    }

    let mut thread_instance: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut _) };

    thread_instance.as_mut().handle = unsafe { pthread_self() };
    let handle = thread_instance.handle;

    let param_arc: Option<ThreadParam> = thread_instance.param.clone();

    // Note: intentionally does *not* call `Thread::delete()`/`join()` here —
    // that would have this thread call `pthread_join()` on its own ID, which
    // is undefined behavior (self-join). Reaping/cleanup is left to whichever
    // other thread eventually calls `join()`/`delete()` on this handle.
    let ret = if let Some(callback) = &thread_instance.callback.clone() {
        callback(thread_instance, param_arc)
    } else {
        Err(Error::NullPtr)
    };

    // The callback has returned: the thread is finished even though nobody
    // has joined it yet, so reflect that in the registry rather than leaving
    // whatever state (e.g. `Ready`) was last tracked while it was running.
    set_thread_state(handle, ThreadState::Deleted);

    Box::into_raw(Box::new(ret)) as *mut c_void
}

/// Internal C-compatible wrapper for simple (parameter-less) thread callbacks.
///
/// Unpacks the boxed `Arc<ThreadSimpleFnPtr>` and invokes it directly; unlike
/// [`callback_c_wrapper`] there is no `Thread` instance to reconstruct here.
/// The callback's `Result<ThreadParam>` is boxed and returned as the raw
/// `void *` exit value, the same way `callback_c_wrapper` does, so `Thread::join()`
/// works identically for threads spawned with `spawn_simple()`.
///
/// # Safety
///
/// - `param_ptr` must be a valid pointer produced by `Box::into_raw` on an `Arc<ThreadSimpleFnPtr>`
/// - Called only by `pthread_create()` as the thread's start routine
unsafe extern "C" fn simple_callback_c_wrapper(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut();
    }

    let func: Box<Arc<ThreadSimpleFnPtr>> = unsafe { Box::from_raw(param_ptr as *mut _) };
    let ret = func();

    // See the equivalent comment in `callback_c_wrapper`.
    set_thread_state(unsafe { pthread_self() }, ThreadState::Deleted);

    Box::into_raw(Box::new(ret)) as *mut c_void
}

impl ThreadFn for Thread {

    /// Returns `true` if this handle refers to no thread - either
    /// [`Thread::new`] was never followed by `spawn`/`spawn_simple`, or the
    /// pthread ID happens to be the reserved `0` sentinel.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let thread = Thread::new("worker", 1024, 5);
    /// assert!(thread.is_null());
    /// ```
    fn is_null(&self) -> bool {
        self.handle == 0
    }

    /// Spawns a new pthread running `callback(self_handle, param)`, passing
    /// through an arbitrary [`ThreadParam`] (an `Arc<dyn Any + Send + Sync>`)
    /// that the callback can downcast back to its concrete type. Prefer
    /// [`ThreadFn::spawn_simple`] when no parameter is needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicI32, Ordering};
    ///
    /// static RECEIVED: AtomicI32 = AtomicI32::new(0);
    ///
    /// let mut thread = Thread::new("worker", 1024, 5);
    /// let param: ThreadParam = Arc::new(42i32);
    ///
    /// let spawned = thread.spawn(Some(param), |_handle, param| {
    ///     if let Some(value) = param.and_then(|p| p.downcast_ref::<i32>().copied()) {
    ///         RECEIVED.store(value, Ordering::SeqCst);
    ///     }
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// spawned.join(core::ptr::null_mut()).unwrap();
    /// assert_eq!(RECEIVED.load(Ordering::SeqCst), 42);
    /// ```
    fn spawn<F>(&mut self, param: Option<ThreadParam>, callback: F) -> Result<Self>
    where
        F: Fn(Box<dyn ThreadFn>, Option<ThreadParam>) -> Result<ThreadParam>,
        F: Send + Sync + 'static,
        Self: Sized,
    {
        let func: Arc<ThreadFnPtr> = Arc::new(callback);
        self.callback = Some(func);
        self.param = param.clone();

        let mut attr: pthread_attr_t = Default::default();

        unsafe {
            pthread_attr_init (&mut attr);
        }

        let requested_stack_size = unsafe {
            osal_rs_get_pthread_stack_min()
        } + self.stack_depth as usize;

        let min_safe_stack_size = 1024usize * 1024usize;

        unsafe {
            pthread_attr_setstacksize (&mut attr, if requested_stack_size < min_safe_stack_size {  min_safe_stack_size } else { requested_stack_size });
        }

        #[cfg(feature = "real_time")]
        unsafe {
            let fifo_param = sched_param {
                sched_priority: self.priority as core::ffi::c_int,
            };
            pthread_attr_setinheritsched(&mut attr, PTHREAD_EXPLICIT_SCHED);
            pthread_attr_setschedpolicy(&mut attr, SCHED_FIFO);
            pthread_attr_setschedparam(&mut attr, &fifo_param);
        }

        let boxed_thread = Box::new(self.clone());

        let ret = unsafe {
            pthread_create(&mut self.handle, &attr, Some(callback_c_wrapper), Box::into_raw(boxed_thread) as *mut c_void)
        };

        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        unsafe {
            pthread_setname_np(self.handle, self.name.as_cstr().as_ptr());
        }

        register_thread(ThreadMetadata {
            thread: self.handle,
            name: self.name.clone(),
            stack_depth: self.stack_depth,
            priority: self.priority,
            thread_number: 0,
            state: ThreadState::Ready,
            current_priority: self.priority,
            base_priority: self.priority,
            run_time_counter: 0,
            stack_high_water_mark: 0,
        });

        Ok(Self {
            handle: self.handle,
            name: self.name.clone(),
            stack_depth: self.stack_depth,
            priority: self.priority,
            callback: self.callback.clone(),
            param,
        })
    }

    /// Spawns a new pthread running `callback()`. Simpler than
    /// [`ThreadFn::spawn`] when no parameter needs to be passed in.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut thread = Thread::new("worker", 1024, 5);
    /// let spawned = thread.spawn_simple(|| {
    ///     println!("Working...");
    ///     Ok(Arc::new(()))
    /// }).unwrap();
    ///
    /// spawned.join(core::ptr::null_mut()).unwrap();
    /// ```
    fn spawn_simple<F>(&mut self, callback: F) -> Result<Self>
    where
        F: Fn() -> Result<ThreadParam> + Send + Sync + 'static,
        Self: Sized,
    {
        let func: Arc<ThreadSimpleFnPtr> = Arc::new(callback);
        let boxed_func = Box::new(func);


        let mut attr: pthread_attr_t = Default::default();

        unsafe {
            pthread_attr_init (&mut attr);
        }

        let requested_stack_size = unsafe {
            osal_rs_get_pthread_stack_min()
        } + self.stack_depth as usize;

        let min_safe_stack_size = 1024usize * 1024usize;

        unsafe {
            pthread_attr_setstacksize (&mut attr, if requested_stack_size < min_safe_stack_size {  min_safe_stack_size } else { requested_stack_size });
        }

        #[cfg(feature = "real_time")]
        unsafe {
            let fifo_param = sched_param {
                sched_priority: self.priority as core::ffi::c_int,
            };
            pthread_attr_setinheritsched(&mut attr, PTHREAD_EXPLICIT_SCHED);
            pthread_attr_setschedpolicy(&mut attr, SCHED_FIFO);
            pthread_attr_setschedparam(&mut attr, &fifo_param);
        }

        let ret = unsafe {
            pthread_create(&mut self.handle, &attr, Some(simple_callback_c_wrapper), Box::into_raw(boxed_func) as *mut c_void)
        };

        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        unsafe {
            pthread_setname_np(self.handle, self.name.as_cstr().as_ptr());
        }

        register_thread(ThreadMetadata {
            thread: self.handle,
            name: self.name.clone(),
            stack_depth: self.stack_depth,
            priority: self.priority,
            thread_number: 0,
            state: ThreadState::Ready,
            current_priority: self.priority,
            base_priority: self.priority,
            run_time_counter: 0,
            stack_high_water_mark: 0,
        });

        Ok(Self {
            handle: self.handle,
            name: self.name.clone(),
            stack_depth: self.stack_depth,
            priority: self.priority,
            callback: self.callback.clone(),
            param: self.param.clone(),
        })
    }

    /// Joins the thread (blocking until it finishes) and forgets its
    /// registry/notification-slot entries, discarding any error from the
    /// underlying `pthread_join`. Prefer [`ThreadFn::join`] when the exit
    /// status matters.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut thread = Thread::new("worker", 1024, 5);
    /// let spawned = thread.spawn_simple(|| Ok(Arc::new(()))).unwrap();
    /// spawned.delete();
    /// ```
    fn delete(&self) {
        let _ = unsafe { pthread_join(self.handle, null_mut()) };
        forget_notify_slot(self.handle);
        forget_thread(self.handle);
    }

    /// Suspends the thread by sending it a dedicated real-time signal that
    /// parks it until [`ThreadFn::resume`] sends the matching wake-up signal
    /// (pthreads has no native suspend/resume of its own). A no-op if this
    /// handle [`ThreadFn::is_null`].
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicU32, Ordering};
    ///
    /// static COUNTER: AtomicU32 = AtomicU32::new(0);
    ///
    /// let mut thread = Thread::new("counter", 1024, 1);
    /// let worker = thread.spawn_simple(|| {
    ///     loop {
    ///         COUNTER.fetch_add(1, Ordering::SeqCst);
    ///         System::delay(1);
    ///     }
    /// }).unwrap();
    ///
    /// System::delay(30);
    /// worker.suspend();
    ///
    /// let paused_at = COUNTER.load(Ordering::SeqCst);
    /// System::delay(50);
    /// // No progress while suspended.
    /// assert_eq!(COUNTER.load(Ordering::SeqCst), paused_at);
    ///
    /// worker.resume();
    /// System::delay(30);
    /// // Progress resumes.
    /// assert!(COUNTER.load(Ordering::SeqCst) > paused_at);
    /// ```
    fn suspend(&self) {
        if self.is_null() {
            return;
        }

        ensure_suspend_signal_handlers();

        unsafe {
            pthread_kill(self.handle, suspend_signal());
        }

        set_thread_state(self.handle, ThreadState::Suspended);
    }

    /// Resumes a thread previously suspended with [`ThreadFn::suspend`]. See
    /// [`ThreadFn::suspend`] for a complete example. A no-op if this handle
    /// [`ThreadFn::is_null`].
    fn resume(&self) {
        if self.is_null() {
            return;
        }

        ensure_suspend_signal_handlers();

        unsafe {
            pthread_kill(self.handle, resume_signal());
        }

        set_thread_state(self.handle, ThreadState::Ready);
    }

    /// Blocks until the thread finishes, writing its exit value (boxed by
    /// [`ThreadFn::spawn`]/`spawn_simple`) to `*ret_val` if non-null. Fails
    /// with [`Error::NullPtr`] if this handle [`ThreadFn::is_null`].
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    /// use std::sync::Arc;
    ///
    /// let mut thread = Thread::new("worker", 1024, 5);
    /// let spawned = thread.spawn_simple(|| Ok(Arc::new(()))).unwrap();
    /// assert!(spawned.join(core::ptr::null_mut()).is_ok());
    /// ```
    fn join(&self, ret_val: DoublePtr) -> Result<i32> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        let ret = unsafe { pthread_join(self.handle, ret_val) };

        if ret != 0 {
            Err(Error::ReturnWithCode(ret))
        } else {
            forget_notify_slot(self.handle);
            forget_thread(self.handle);
            Ok(0)
        }
    }

    /// Returns a [`ThreadMetadata`] snapshot for this thread. See
    /// [`Thread::get_metadata`] (the inherent, static-style helper this
    /// delegates to) for a complete example.
    fn get_metadata(&self) -> ThreadMetadata {
        self.metadata()
    }

    /// Returns a [`Thread`] handle for the calling thread itself - works
    /// whether called from the "main" thread or from inside a callback
    /// running on a thread this crate spawned.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    /// assert!(!current.is_null());
    /// ```
    fn get_current() -> Self
    where
        Self: Sized,
    {
        // `pthread_self()` returns whichever thread calls it, so this is
        // correct whether `get_current()` runs on the "main" thread or from
        // inside a callback running on a thread this crate spawned (see
        // `callback_c_wrapper`, which relies on the same call for the same
        // reason).
        Self {
            handle: unsafe { pthread_self() },
            name: Bytes::from_str("current"),
            stack_depth: 0,
            priority: 0,
            callback: None,
            param: None,
        }
    }

    /// Sets or updates this thread's single-slot notification value (see
    /// [`ThreadNotification`] for the available update strategies) and wakes
    /// it if it's blocked in [`ThreadFn::wait_notification`].
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    /// current.notify(ThreadNotification::SetValueWithOverwrite(5)).unwrap();
    ///
    /// let value = current.wait_notification(0, 0, 0).unwrap();
    /// assert_eq!(value, 5);
    /// ```
    fn notify(&self, notification: ThreadNotification) -> Result<()> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        let slot = notify_slot(self.handle);

        let result = {
            let mut state = slot.state.lock().unwrap();
            apply_notification(&mut state, notification)
        };

        if result.is_ok() {
            // Wake a thread blocked in wait_notification() below; a no-op if none is.
            slot.cv.notify_all();
        }

        result
    }

    /// ISR-safe variant of [`ThreadFn::notify`]; identical on POSIX (there
    /// is no real interrupt context, and thus no scheduler decision to
    /// report back through `higher_priority_task_woken`, which is always set
    /// to `0`).
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    /// let mut woken = 0;
    /// current.notify_from_isr(ThreadNotification::Increment, &mut woken).unwrap();
    /// assert_eq!(woken, 0);
    ///
    /// let value = current.wait_notification(0, 0, 0).unwrap();
    /// assert_eq!(value, 1);
    /// ```
    fn notify_from_isr(&self, notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()> {
        // No real interrupt context on POSIX, and thus no scheduler decision
        // to report back — matches `System`'s other `_from_isr` stand-ins.
        *higher_priority_task_woken = 0;

        self.notify(notification)
    }

    /// Blocks until a notification is pending or `timeout_ticks` elapses
    /// (pass [`TickType::MAX`] to wait forever), returning the notification
    /// value. `bits_to_clear_on_entry`/`bits_to_clear_on_exit` clear the
    /// matching bits from the value before waiting/before returning,
    /// respectively. Fails with [`Error::Timeout`] on timeout.
    ///
    /// # Examples
    ///
    /// ```
    /// use osal_rs::os::*;
    ///
    /// let current = Thread::get_current();
    ///
    /// // Nothing notified yet: times out instead of blocking forever.
    /// assert!(current.wait_notification(0, 0, 10).is_err());
    ///
    /// current.notify(ThreadNotification::SetValueWithOverwrite(9)).unwrap();
    /// assert_eq!(current.wait_notification(0, 0, 10).unwrap(), 9);
    /// ```
    fn wait_notification(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32, timeout_ticks: TickType) -> Result<u32> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        let slot = notify_slot(self.handle);
        let mut state = slot.state.lock().unwrap();

        state.value &= !bits_to_clear_on_entry;

        if !state.pending {
            set_thread_state(self.handle, ThreadState::Blocked);

            if timeout_ticks == TickType::MAX {
                // Not a `while !state.pending` loop: `pending` is flipped by
                // `notify()` through a *different* `MutexGuard` (its own
                // `slot.state.lock()`) while this thread is parked inside
                // `wait()` — invisible to clippy's `while_immutable_condition`,
                // which only looks for reassignment in the loop body.
                loop {
                    if state.pending {
                        break;
                    }
                    slot.cv.wait(&state);
                }
            } else {
                let deadline = monotonic_deadline(Duration::from_millis((timeout_ticks as u64).saturating_mul(TICK_PERIOD_MS)));

                loop {
                    if state.pending {
                        break;
                    }
                    if slot.cv.wait_until(&state, deadline) {
                        break;
                    }
                }
            }

            set_thread_state(self.handle, ThreadState::Ready);
        }

        if !state.pending {
            return Err(Error::Timeout);
        }

        state.pending = false;
        let value = state.value;
        state.value &= !bits_to_clear_on_exit;

        Ok(value)
    }
}

impl Deref for Thread {
    type Target = ThreadHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Debug for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thread")
            .field("handle", &self.handle)
            .field("name", &self.name)
            .field("stack_depth", &self.stack_depth)
            .field("priority", &self.priority)
            .field("callback", &self.callback.as_ref().map(|_| "Some(...)"))
            .field("param", &self.param)
            .finish()
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Thread {{ handle: {:?}, name: {}, priority: {}, stack_depth: {} }}",
            self.handle,
            self.name,
            self.priority,
            self.stack_depth
        )
    }
}