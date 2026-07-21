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
#[cfg(feature = "sched_fifo")]
use crate::posix::ffi::{PTHREAD_EXPLICIT_SCHED, SCHED_FIFO, pthread_attr_setinheritsched, pthread_attr_setschedparam, pthread_attr_setschedpolicy, sched_param};
use crate::posix::ffi::{
	__libc_current_sigrtmin, CLOCK_MONOTONIC, ETIMEDOUT, PTHREAD_ONCE_INIT, clock_gettime, get_pthread_stack_min, pthread_attr_init, pthread_attr_setstacksize, pthread_attr_t,
	pthread_cond_broadcast, pthread_cond_destroy, pthread_cond_init, pthread_cond_t, pthread_cond_timedwait, pthread_cond_wait, pthread_condattr_init, pthread_condattr_setclock,
	pthread_condattr_t, pthread_create, pthread_join, pthread_kill, pthread_once, pthread_once_t, pthread_self, pthread_setname_np, sigdelset, sigfillset, sigset_t, signal, sigsuspend,
	timespec,
};
use crate::posix::types::{BaseType, StackType, ThreadHandle, TickType, UBaseType};
use crate::traits::{ThreadFn, ThreadFnPtr, ThreadNotification, ThreadParam, ToPriority, ToTick};
use crate::utils::{Bytes, DoublePtr, Error, Result};

const MAX_TASK_NAME_LEN: usize = 16;

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

/// Represents the possible states of a FreeRTOS task/thread.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::{Thread, ThreadState};
/// 
/// let thread = Thread::current();
/// let metadata = thread.metadata().unwrap();
/// 
/// match metadata.state {
///     ThreadState::Running => println!("Thread is currently executing"),
///     ThreadState::Ready => println!("Thread is ready to run"),
///     ThreadState::Blocked => println!("Thread is waiting for an event"),
///     ThreadState::Suspended => println!("Thread is suspended"),
///     _ => println!("Unknown state"),
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ThreadState {
    /// Thread is currently executing on a CPU
    Running = 0, //TODO: to implement in ThreadMetadata
    /// Thread is ready to run but not currently executing
    Ready = 1, //TODO: to implement in ThreadMetadata
    /// Thread is blocked waiting for an event (e.g., semaphore, queue)
    Blocked = 2,
    /// Thread in ThreadMetadata has been explicitly suspended
    Suspended = 3, //TODO: to implement in ThreadMetadata
    /// Thread has been deleted
    Deleted = 4, //TODO: to implement in ThreadMetadata
    /// Invalid or unknown state
    Invalid,
}

/// Metadata and runtime information about a thread.
///
/// Contains detailed information about a thread's state, priorities, stack usage,
/// and runtime statistics.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::Thread;
/// 
/// let thread = Thread::current();
/// let metadata = thread.metadata().unwrap();
/// 
/// println!("Thread: {}", metadata.name);
/// println!("Priority: {}", metadata.priority);
/// println!("Stack high water mark: {}", metadata.stack_high_water_mark);
/// ```
#[derive(Clone, Debug)]
pub struct ThreadMetadata {
    /// FreeRTOS task handle
    pub thread: ThreadHandle,
    /// Thread name
    pub name: Bytes<MAX_TASK_NAME_LEN>,
    /// Original stack depth allocated for this thread
    pub stack_depth: StackType,
    /// Thread priority
    pub priority: UBaseType,
    /// Unique thread number assigned by OS
    pub thread_number: UBaseType,
    /// Current execution state
    pub state: ThreadState,
    /// Current priority (may differ from base priority due to priority inheritance)
    pub current_priority: UBaseType,
    /// Base priority without inheritance
    pub base_priority: UBaseType,
    /// Total runtime counter (requires configGENERATE_RUN_TIME_STATS)
    pub run_time_counter: UBaseType,
    /// Minimum remaining stack space ever recorded (lower values indicate higher stack usage)
    pub stack_high_water_mark: StackType,
}

unsafe impl Send for ThreadMetadata {}
unsafe impl Sync for ThreadMetadata {}

/// Provides default values for ThreadMetadata.
///
/// Creates a metadata instance with null/zero values, representing an
/// invalid or uninitialized thread.
impl Default for ThreadMetadata {
    fn default() -> Self {
        Self {
            thread: 0,
            name: Bytes::new(),
            stack_depth: 0,
            priority: 0,
            thread_number: 0,
            state: ThreadState::Invalid,
            current_priority: 0,
            base_priority: 0,
            run_time_counter: 0,
            stack_high_water_mark: 0,
        }
    }
}

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

    pub fn new_with_to_priority(name: &str, stack_depth: StackType, priority: impl ToPriority) -> Self {
        Self::new(name, stack_depth, priority.to_priority())
    }

    pub fn new_with_handle_and_to_priority(handle: ThreadHandle, name: &str, stack_depth: StackType, priority: impl ToPriority) -> Result<Self> {
        Self::new_with_handle(handle, name, stack_depth, priority.to_priority())
    }

    pub fn get_metadata_from_handle(handle: ThreadHandle) -> ThreadMetadata {
        if handle == 0 {
            return ThreadMetadata::default();
        }

        ThreadMetadata {
            thread: handle,
            name: Bytes::from_str("thread"),
            stack_depth: 0,
            priority: 0,
            thread_number: 0,
            state: ThreadState::Ready,
            current_priority: 0,
            base_priority: 0,
            run_time_counter: 0,
            stack_high_water_mark: 0,
        }
    }

    pub fn get_metadata(thread: &Thread) -> ThreadMetadata {
        // Name/stack/priority reflect what was passed to `new()` regardless of
        // whether the thread has been spawned yet; only `state`/`thread` depend
        // on there being a live `pthread_t` behind it.
        ThreadMetadata {
            thread: thread.handle,
            name: thread.name.clone(),
            stack_depth: thread.stack_depth,
            priority: thread.priority,
            thread_number: thread.handle,
            state: if thread.is_null() { ThreadState::Invalid } else { ThreadState::Ready },
            current_priority: thread.priority,
            base_priority: thread.priority,
            run_time_counter: 0,
            stack_high_water_mark: 0,
        }
    }

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

    Box::into_raw(Box::new(ret)) as *mut c_void
}

impl ThreadFn for Thread {

    fn is_null(&self) -> bool {
        self.handle == 0
    }

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
            get_pthread_stack_min()
        } + self.stack_depth as usize;

        let min_safe_stack_size = 1024usize * 1024usize;

        unsafe {
            pthread_attr_setstacksize (&mut attr, if requested_stack_size < min_safe_stack_size {  min_safe_stack_size } else { requested_stack_size });
        }

        #[cfg(feature = "sched_fifo")]
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
            get_pthread_stack_min()
        } + self.stack_depth as usize;

        let min_safe_stack_size = 1024usize * 1024usize;

        unsafe {
            pthread_attr_setstacksize (&mut attr, if requested_stack_size < min_safe_stack_size {  min_safe_stack_size } else { requested_stack_size });
        }

        #[cfg(feature = "sched_fifo")]
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

    fn delete(&self) {
        let _ = unsafe { pthread_join(self.handle, null_mut()) };
        forget_notify_slot(self.handle);
        forget_thread(self.handle);
    }

    fn suspend(&self) {
        if self.is_null() {
            return;
        }

        ensure_suspend_signal_handlers();

        unsafe {
            pthread_kill(self.handle, suspend_signal());
        }
    }

    fn resume(&self) {
        if self.is_null() {
            return;
        }

        ensure_suspend_signal_handlers();

        unsafe {
            pthread_kill(self.handle, resume_signal());
        }
    }

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

    fn get_metadata(&self) -> ThreadMetadata {
        self.metadata()
    }

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

    fn notify_from_isr(&self, notification: ThreadNotification, higher_priority_task_woken: &mut BaseType) -> Result<()> {
        // No real interrupt context on POSIX, and thus no scheduler decision
        // to report back — matches `System`'s other `_from_isr` stand-ins.
        *higher_priority_task_woken = 0;

        self.notify(notification)
    }

    fn wait_notification(&self, bits_to_clear_on_entry: u32, bits_to_clear_on_exit: u32, timeout_ticks: TickType) -> Result<u32> {
        if self.is_null() {
            return Err(Error::NullPtr);
        }

        let slot = notify_slot(self.handle);
        let mut state = slot.state.lock().unwrap();

        state.value &= !bits_to_clear_on_entry;

        if !state.pending {
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