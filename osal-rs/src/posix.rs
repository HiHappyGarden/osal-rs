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

//! POSIX implementation of OSAL-RS traits.
//!
//! This module provides concrete implementations of all OSAL-RS traits on top
//! of the POSIX/pthreads API, so OSAL-RS applications can run — and their
//! tests and doc examples can execute for real — on any POSIX host (Linux,
//! macOS, ...) without embedded hardware or a cross toolchain.
//!
//! # Overview
//!
//! POSIX threads (pthreads) provide the primitives this module wraps:
//! - Preemptive multitasking backed by native OS threads
//! - Mutexes with priority inheritance (`PTHREAD_PRIO_INHERIT`)
//! - Semaphores, event groups and queues built on a `pthread_mutex_t` +
//!   `pthread_cond_t` pair rather than plain POSIX unnamed semaphores, so
//!   `signal()`/`post()` can enforce a maximum count
//! - Software timers, each backed by a dedicated background thread plus a
//!   real kernel timer (`timer_create(2)`)
//! - Monotonic timing via `CLOCK_MONOTONIC`
//!
//! This module wraps the POSIX C API with safe Rust abstractions that
//! implement the same OSAL-RS trait interfaces as the FreeRTOS backend
//! (see [`crate::freertos`]), so application code written against
//! `osal_rs::os::*` is portable between the two by switching feature flags.
//!
//! # Modules
//!
//! - [`config`] - POSIX configuration constants (tick period)
//! - [`duration`] - Duration/tick conversions for the POSIX clock
//! - [`event_group`] - Event group synchronization primitives
//! - [`mutex`] - Mutex implementations with priority inheritance
//! - [`queue`] - Message queues for inter-task communication
//! - [`semaphore`] - Binary and counting semaphores
//! - [`system`] - System-level control and timing
//! - [`thread`] - Task/thread creation and management
//! - [`timer`] - Software timers for delayed/periodic callbacks
//! - [`types`] - POSIX-specific type definitions
//!
//! # Requirements
//!
//! - A POSIX-compliant host (glibc/Linux tested)
//! - No special build steps: unlike `freertos`, this backend links only
//!   against the host's libc/libpthread — no cross toolchain or RTOS kernel
//!   sources required
//!
//! # Usage
//!
//! Types from this module implement the traits defined in [`crate::traits`],
//! the same interfaces the `freertos` backend implements, allowing you to
//! write portable code through the unified [`crate::os`] module.
//!
//! ## Example: Creating a Thread
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
//!
//! ## Example: Using a Mutex
//!
//! ```
//! use osal_rs::os::*;
//!
//! let mutex = Mutex::new(0);
//! {
//!     let mut guard = mutex.lock().unwrap();
//!     *guard += 1;
//! } // Lock released here
//!
//! assert_eq!(*mutex.lock().unwrap(), 1);
//! ```
//!
//! ## Example: Using a Queue
//!
//! ```
//! use osal_rs::os::*;
//!
//! let queue = Queue::new(4, 4).unwrap();
//!
//! // Producer
//! queue.post(&[1u8, 2, 3, 4], 100).unwrap();
//!
//! // Consumer
//! let mut buffer = [0u8; 4];
//! queue.fetch(&mut buffer, 100).unwrap();
//! assert_eq!(buffer, [1, 2, 3, 4]);
//! ```
//!
//! ## Example: Using a Semaphore
//!
//! ```
//! use osal_rs::os::*;
//! use osal_rs::utils::OsalRsBool;
//! use core::time::Duration;
//!
//! let sem = Semaphore::new(1, 0).unwrap();
//! sem.signal();
//! assert_eq!(sem.wait(Duration::from_millis(100)), OsalRsBool::True);
//! ```
//!
//! ## Example: Using an Event Group
//!
//! ```
//! use osal_rs::os::*;
//! use osal_rs::os::types::EventBits;
//!
//! const EVENT_A: EventBits = 1 << 0;
//! const EVENT_B: EventBits = 1 << 1;
//!
//! let events = EventGroup::new().unwrap();
//! events.set(EVENT_A | EVENT_B);
//!
//! let bits = events.wait(EVENT_A | EVENT_B, true, 100);
//! assert_eq!(bits & (EVENT_A | EVENT_B), EVENT_A | EVENT_B);
//! ```
//!
//! ## Example: Using a Timer
//!
//! ```
//! use osal_rs::os::*;
//! use std::sync::Arc;
//! use core::time::Duration;
//!
//! let timer = Timer::new_with_to_tick(
//!     "heartbeat",
//!     Duration::from_millis(10),
//!     false, // one-shot
//!     None,
//!     |_timer, _param| {
//!         println!("Tick!");
//!         Ok(Arc::new(()))
//!     }
//! ).unwrap();
//!
//! timer.start(0);
//! ```
//!
//! # Platform Support
//!
//! Tested against glibc on Linux. Any POSIX-compliant host implementing
//! pthreads, `timer_create(2)`/`sigwait(3)` and `CLOCK_MONOTONIC` should
//! work, but only Linux/glibc is part of this crate's test suite.
//!
//! # Safety
//!
//! This module uses FFI to call POSIX/pthread functions. The Rust wrappers
//! ensure memory safety through:
//! - Proper lifetime management
//! - Type safety
//! - Resource cleanup (RAII)
//! - Checked conversions
//!
//! # Caveats
//!
//! - `System::start()` on POSIX simply spins until [`crate::os::System::stop`]
//!   is called from another thread — there is no scheduler to hand control
//!   to, unlike FreeRTOS where it never returns.
//! - Timers each spawn their own background thread and permanently block
//!   `SIGALRM` on the thread that creates them (see [`timer`] for why);
//!   create a new [`crate::os::Timer`] rather than reusing one that already
//!   fired as a one-shot.

/// POSIX FFI (Foreign Function Interface) bindings.
///
/// This module is private and contains unsafe C bindings to the POSIX/pthread API.
#[macro_use]
mod ffi;

/// POSIX configuration constants and utilities.
pub mod config;

/// Duration type implementations for POSIX clock/time conversion.
pub(crate) mod duration;

/// Event group synchronization primitives.
pub(crate) mod event_group;

/// Mutex implementations with optional priority inheritance.
pub(crate) mod mutex;

/// Message queue implementations for inter-task communication.
pub(crate) mod queue;

/// Binary and counting semaphore implementations.
pub(crate) mod semaphore;

/// System-level control, timing, and scheduler management.
pub(crate) mod system;

/// Task/thread creation, management, and notifications.
pub(crate) mod thread;

/// Software timer implementations for delayed and periodic callbacks.
pub(crate) mod timer;

/// POSIX-specific type definitions and aliases.
pub mod types;