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

//! # OSAL-RS - Operating System Abstraction Layer for Rust
//!
//! A cross-platform abstraction layer for embedded and real-time operating systems.
//!
//! ## Overview
//!
//! OSAL-RS provides a unified, safe Rust API for working with different real-time
//! operating systems. It currently ships two backends, selected at compile time
//! via feature flags - there is no default, exactly one must be enabled:
//! - **FreeRTOS** (feature `freertos`) - for bare-metal embedded targets
//! - **POSIX** (feature `posix`) - runs on any pthreads-capable host (Linux, macOS)
//!   so applications, tests and doc examples can execute natively without
//!   embedded hardware or a cross toolchain
//!
//! Application code written against `osal_rs::os::*` is portable between the
//! two backends by switching feature flags.
//!
//! ## Features
//!
//! - **Thread Management**: Create and control threads with priorities
//! - **Synchronization**: Mutexes, semaphores, and event groups
//! - **Communication**: Queues for inter-thread message passing
//! - **Timers**: Software timers for periodic and one-shot operations
//! - **Time Management**: Duration-based timing with tick conversion
//! - **No-std Support**: Works in bare-metal embedded environments (`freertos` backend)
//! - **Host Testing**: Runs under `std` on POSIX hosts for native testing (`posix` backend)
//! - **Type Safety**: Leverages Rust's type system for correctness
//! - **Async/Await**: Backend-agnostic `async`/`await` without Tokio (feature `async`)
//!
//! ## Quick Start
//!
//! ### Basic Thread Example
//!
//! ```no_run
//! use osal_rs::os::*;
//!
//! fn main() {
//!     // Create a thread
//!     let mut thread = Thread::new(
//!         "worker",
//!         4096,  // stack size
//!         5,     // priority
//!     );
//!
//!     thread.spawn_simple(|| {
//!         loop {
//!             println!("Working...");
//!             System::delay(1000);
//!         }
//!     }).unwrap();
//!
//!     // Start the scheduler (never returns)
//!     System::start();
//! }
//! ```
//!
//! ### Mutex Example
//!
//! ```
//! use osal_rs::os::*;
//! use std::sync::Arc;
//!
//! let counter = Arc::new(Mutex::new(0));
//! let counter_clone = counter.clone();
//!
//! let mut thread = Thread::new("incrementer", 2048, 5);
//! thread.spawn_simple(move || {
//!     let mut guard = counter_clone.lock().unwrap();
//!     *guard += 1;
//!     Ok(Arc::new(()))
//! }).unwrap();
//! ```
//!
//! ### Queue Example
//!
//! ```
//! use osal_rs::os::*;
//!
//! let queue = Queue::new(10, 4).unwrap();
//!
//! // Send data
//! let data = [1u8, 2, 3, 4];
//! queue.post(&data, 100).unwrap();
//!
//! // Receive data
//! let mut buffer = [0u8; 4];
//! queue.fetch(&mut buffer, 100).unwrap();
//! ```
//!
//! ### Semaphore Example
//!
//! ```
//! use osal_rs::os::*;
//! use osal_rs::utils::OsalRsBool;
//! use core::time::Duration;
//!
//! let sem = Semaphore::new(1, 1).unwrap();
//!
//! if sem.wait(Duration::from_millis(100)) == OsalRsBool::True {
//!     // Critical section
//!     sem.signal();
//! }
//! ```
//!
//! ### Timer Example
//!
//! ```
//! extern crate alloc;
//! use osal_rs::os::*;
//! use alloc::sync::Arc;
//! use core::time::Duration;
//!
//! let timer = Timer::new_with_to_tick(
//!     "periodic",
//!     Duration::from_millis(500),
//!     true,  // auto-reload
//!     None,
//!     |_timer, _param| {
//!         println!("Timer tick");
//!         Ok(Arc::new(()))
//!     }
//! ).unwrap();
//!
//! timer.start_with_to_tick(Duration::from_millis(10));
//! ```
//!
//! ### Async/Await Example (feature `async`)
//!
#![cfg_attr(feature = "async", doc = "```")]
#![cfg_attr(not(feature = "async"), doc = "```ignore")]
//! use osal_rs::os::{block_on, AsyncMutex, AsyncQueue, AsyncSemaphore};
//!
//! // Drive a future to completion on the calling RTOS task — no Tokio needed.
//! block_on(async {
//!     let mutex = AsyncMutex::new(0u32);
//!     {
//!         let mut guard = mutex.lock().await;
//!         *guard += 1;
//!     }
//!
//!     let sem = AsyncSemaphore::new(1, 0).unwrap();
//!     sem.signal();
//!     sem.wait_async().await;
//!
//!     let queue = AsyncQueue::new(8, 4).unwrap();
//!     queue.post_async(&[1u8, 2, 3, 4]).await.unwrap();
//!     let mut buf = [0u8; 4];
//!     queue.fetch_async(&mut buf).await.unwrap();
//! });
//! ```
//!
//! ## Module Organization
//!
//! - [`os`] - Main module containing all OS abstractions
//!   - Threads, mutexes, semaphores, queues, event groups, timers
//!   - System-level functions
//!   - Type definitions
//!   - `block_on`, `AsyncQueue`, `AsyncSemaphore`, `AsyncMutex` (feature `async`)
//! - [`async_primitives`] - Async wrappers for OSAL primitives (feature `async`)
//! - [`utils`] - Utility types and error definitions
//! - [`log`] - Logging macros
//! - `traits` - Private module defining the trait abstractions
//! - `freertos` - Private FreeRTOS implementation (enabled with `freertos` feature)
//! - `posix` - Private POSIX implementation (enabled with `posix` feature)
//!
//! ## Features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `freertos` | ❌ | FreeRTOS backend |
//! | `posix` | ❌ | POSIX/host backend |
//! | `async` | ❌ | Async/await without Tokio |
//! | `serde` | ❌ | Serialization via `osal-rs-serde` |
//! | `real_time` | ❌ | POSIX only: schedules spawned threads with `SCHED_FIFO` instead of inheriting the creating thread's policy/priority. Not meant to be requested by hand - the build script auto-enables it when the host OS/kernel supports real-time scheduling |
//!
//! There is no default backend: exactly one of `freertos`/`posix` must be
//! enabled explicitly. Enabling neither trips the `compile_error!` below;
//! enabling both is equally unsupported (the two are mutually exclusive by
//! `cfg`, so the crate fails to build either way).
//!
//! ## Requirements
//!
//! When using with FreeRTOS:
//! - FreeRTOS kernel must be properly configured
//! - Link the C porting layer from `osal-rs-porting/freertos/`
//! - Set appropriate `FreeRTOSConfig.h` options:
//!   - `configTICK_RATE_HZ` - Defines the tick frequency
//!   - `configUSE_MUTEXES` - Must be 1 for mutex support
//!   - `configUSE_COUNTING_SEMAPHORES` - Must be 1 for semaphore support
//!   - `configUSE_TIMERS` - Must be 1 for timer support
//!   - `configSUPPORT_DYNAMIC_ALLOCATION` - Must be 1 for dynamic allocation
//!
//! When using with POSIX:
//! - A POSIX-compliant host implementing pthreads, `timer_create(2)`/`sigwait(3)`
//!   and `CLOCK_MONOTONIC` (glibc/Linux is part of this crate's test suite)
//! - No special build steps: unlike `freertos`, this backend links only
//!   against the host's libc/libpthread - no cross toolchain or RTOS kernel
//!   sources required
//! - Disables `no_std`: the `posix` feature builds the crate against `std`
//!
//! ## Platform Support
//!
//! Currently tested on:
//! - ARM Cortex-M (Raspberry Pi Pico/RP2040, RP2350) - `freertos` backend
//! - ARM Cortex-M4F (STM32F4 series) - `freertos` backend
//! - ARM Cortex-M7 (STM32H7 series) - `freertos` backend
//! - RISC-V (RP2350 RISC-V cores) - `freertos` backend
//! - glibc/Linux - `posix` backend
//!
//! ## Thread Safety
//!
//! All types are designed with thread safety in mind:
//! - Most operations are thread-safe and can be called from multiple threads
//! - Methods with `_from_isr` suffix are ISR-safe (callable from interrupt context)
//! - Regular methods (without `_from_isr`) must not be called from ISR context
//! - Mutexes use priority inheritance to prevent priority inversion
//!
//! ## ISR Context
//!
//! Operations in ISR context have restrictions (applies to the `freertos`
//! backend; POSIX has no interrupt context - `_from_isr` variants are
//! provided there for API compatibility but behave like their regular
//! counterparts):
//! - Cannot block or use timeouts (must use zero timeout or `_from_isr` variants)
//! - Must be extremely fast to avoid blocking other interrupts
//! - Use semaphores or queues to defer work to task context
//! - Event groups and notifications are ISR-safe for signaling
//!
//! ## Safety
//!
//! This library uses `unsafe` internally to interface with C APIs but provides
//! safe Rust abstractions. All public APIs are designed to be memory-safe when
//! used correctly:
//! - Type safety through generic parameters
//! - RAII patterns for automatic resource management
//! - Rust's ownership system prevents data races
//! - FFI boundaries are carefully validated
//!
//! ## Performance Considerations
//!
//! - `freertos` backend: allocations happen on the FreeRTOS heap (via the
//!   `#[global_allocator]` in [`os`]), not the system heap
//! - `posix` backend: uses the system allocator and native OS threads; each
//!   [`os::Timer`] spawns a dedicated background thread
//! - Stack sizes must be carefully tuned for each thread
//! - Priority inversion is mitigated through priority inheritance
//! - Context switches are triggered by blocking operations
//!
//! ## Best Practices
//!
//! 1. **Thread Creation**: Always specify appropriate stack sizes based on usage
//! 2. **Mutexes**: Prefer scoped locking with guards to prevent deadlocks
//! 3. **Queues**: Use type-safe `QueueStreamed` when possible
//! 4. **Semaphores**: Use binary semaphores for signaling, counting for resources
//! 5. **ISR Handlers**: Keep ISR code minimal, defer work to tasks
//! 6. **Error Handling**: Always check `Result` return values
//! 7. **Async Tasks**: Call `block_on` once per RTOS task; do not nest executors
//!
//! ## License
//!
//! LGPL-2.1-or-later - See LICENSE file for details

#![cfg_attr(not(feature = "posix"), no_std)]

extern crate alloc;

#[cfg(not(any(feature = "freertos", feature = "posix")))]
compile_error!("Enable either the `freertos` backend or the `posix` host backend.");

/// FreeRTOS implementation of OSAL traits.
///
/// This module contains the concrete implementation of all OSAL abstractions
/// for FreeRTOS, including threads, mutexes, queues, timers, etc.
///
/// Enabled with the `freertos` feature flag (no default backend - must be
/// requested explicitly).
#[cfg(all(not(feature = "posix"), feature = "freertos"))]
mod freertos;

/// POSIX implementation of OSAL traits.
///
/// This module contains the concrete implementation of all OSAL abstractions
/// on top of the POSIX/pthreads API (threads, mutexes, semaphores, queues,
/// event groups, timers), so applications - and their tests and doc examples -
/// can run natively on any POSIX host (Linux, macOS) without embedded hardware
/// or a cross toolchain. See [`posix`] for details.
///
/// Enabled with the `posix` feature flag.
#[cfg(all(feature = "posix", not(feature = "freertos")))]
mod posix;

pub mod log;

/// Trait definitions for OSAL abstractions.
///
/// This private module defines all the trait interfaces that concrete
/// implementations must satisfy. Traits are re-exported through the `os` module.
mod traits;

pub mod utils;

/// Async executor (block_on).
#[cfg(feature = "async")]
mod async_executor;

/// Async primitives (AsyncQueue, AsyncSemaphore, AsyncMutex).
#[cfg(feature = "async")]
pub mod async_primitives;

/// Select FreeRTOS as the active OSAL backend.
#[cfg(all(not(feature = "posix"), feature = "freertos"))]
use crate::freertos as osal;

/// Select POSIX as the active OSAL backend.
#[cfg(all(feature = "posix", not(feature = "freertos")))]
use crate::posix as osal;

/// Main OSAL module re-exporting all OS abstractions and traits.
///
/// This module provides a unified interface to all OSAL functionality through `osal_rs::os::*`.
/// It re-exports:
/// - Thread management types (`Thread`, `ThreadNotification`)
/// - Synchronization primitives (`Mutex`, `Semaphore`, `EventGroup`)
/// - Communication types (`Queue`, `QueueStreamed`)
/// - Timer types (`Timer`)
/// - System functions (`System`)
/// - All trait definitions from the `traits` module
/// - Type definitions and configuration from the active backend
///
/// The actual implementation (FreeRTOS or POSIX) is selected at compile time via features.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::*;
///
/// fn main() {
///     // Create and start a thread
///     let thread = Thread::new("worker", 4096, 5, || {
///         println!("Worker thread running");
///     }).unwrap();
///     
///     thread.start().unwrap();
///     System::start();
/// }
/// ```
pub mod os {

    #[cfg(all(not(feature = "posix"), feature = "freertos"))]
    use crate::osal::allocator::Allocator;

    /// Global allocator using the underlying RTOS heap.
    ///
    /// This static variable configures Rust's global allocator to use the
    /// RTOS heap (e.g., FreeRTOS heap) instead of the system heap.
    ///
    /// # Behavior
    ///
    /// - All allocations via `alloc::vec::Vec`, `alloc::boxed::Box`, `alloc::string::String`, etc.
    ///   will use the RTOS heap
    /// - Memory is managed by the underlying RTOS (e.g., `pvPortMalloc`/`vPortFree` in FreeRTOS)
    /// - Thread-safe: can be used from multiple tasks safely
    ///
    /// # Feature Flag
    ///
    /// Active when using the `freertos` backend.
    ///
    /// # FreeRTOS Configuration
    ///
    /// Ensure your `FreeRTOSConfig.h` has:
    /// - `configSUPPORT_DYNAMIC_ALLOCATION` set to 1
    /// - `configTOTAL_HEAP_SIZE` configured appropriately for your application
    ///
    /// # Example
    ///
    /// ```ignore
    /// use alloc::vec::Vec;
    ///
    /// // This allocation uses the FreeRTOS heap via ALLOCATOR
    /// let mut v = Vec::new();
    /// v.push(42);
    /// ```
    #[cfg(all(not(feature = "posix"), feature = "freertos"))]
    #[global_allocator]
    pub static ALLOCATOR: Allocator = Allocator;

    /// Event group synchronization primitives.
    #[allow(unused_imports)]
    pub use crate::osal::event_group::*;
    
    /// Mutex types and guards for mutual exclusion.
    #[allow(unused_imports)]
    pub use crate::osal::mutex::*;
    
    /// Queue types for inter-task communication.
    #[allow(unused_imports)]
    pub use crate::osal::queue::*;
    
    /// Semaphore types for signaling and resource management.
    #[allow(unused_imports)]
    pub use crate::osal::semaphore::*;
    
    /// System-level functions (scheduler, timing, critical sections).
    pub use crate::osal::system::*;
    
    /// Thread/task management and notification types.
    pub use crate::osal::thread::*;
    
    /// Software timer types for periodic and one-shot callbacks.
    #[allow(unused_imports)]
    pub use crate::osal::timer::*;
    
    /// All OSAL trait definitions for advanced usage.
    pub use crate::traits::*;
    
    /// RTOS configuration constants and types.
    pub use crate::osal::config as config;
    
    /// Type aliases and common types used throughout OSAL.
    pub use crate::osal::types as types;

    /// Single-future async executor (backend-agnostic, no Tokio).
    #[cfg(feature = "async")]
    pub use crate::async_executor::block_on;

    /// Async-capable wrappers for OSAL primitives.
    #[cfg(feature = "async")]
    pub use crate::async_primitives::*;
    
}

/// Default panic handler for `no_std` environments.
///
/// This panic handler is active for the `freertos` backend.
/// It prints panic information and enters an infinite loop to halt execution.
///
/// # Behavior
///
/// 1. Attempts to print panic information using the `println!` macro
/// 2. Enters an infinite empty loop, halting the program
///
/// # Feature Flag
///
/// - Enabled for `freertos`
/// - Automatically disabled when using `posix`
///
/// # Safety
///
/// This handler is intentionally simple and does not attempt cleanup or recovery.
/// In production embedded systems, consider:
/// - Logging panic info to persistent storage
/// - Performing safe shutdown procedures
/// - Resetting the system via watchdog
///
#[cfg(all(not(feature = "posix"), feature = "freertos"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic occurred: {}", info);
    #[allow(clippy::empty_loop)]
    loop {}
}

