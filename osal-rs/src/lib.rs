/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! # OSAL-RS - Operating System Abstraction Layer for Rust
//!
//! A cross-platform abstraction layer for embedded and real-time operating systems.
//!
//! ## Overview
//!
//! OSAL-RS provides a unified, safe Rust API for working with different real-time
//! operating systems. Currently supports FreeRTOS with planned support for POSIX
//! and other RTOSes.
//!
//! ## Features
//!
//! - **Thread Management**: Create and control threads with priorities
//! - **Synchronization**: Mutexes, semaphores, and event groups
//! - **Communication**: Queues for inter-thread message passing
//! - **Timers**: Software timers for periodic and one-shot operations
//! - **Time Management**: Duration-based timing with tick conversion
//! - **No-std Support**: Works in bare-metal embedded environments
//! - **Type Safety**: Leverages Rust's type system for correctness
//!
//! ## Quick Start
//!
//! ### Basic Thread Example
//!
//! ```ignore
//! use osal_rs::os::*;
//! use core::time::Duration;
//!
//! fn main() {
//!     // Create a thread
//!     let thread = Thread::new(
//!         "worker",
//!         4096,  // stack size
//!         5,     // priority
//!         || {
//!             loop {
//!                 println!("Working...");
//!                 Duration::from_secs(1).sleep();
//!             }
//!         }
//!     ).unwrap();
//!
//!     thread.start().unwrap();
//!     
//!     // Start the scheduler
//!     System::start();
//! }
//! ```
//!
//! ### Mutex Example
//!
//! ```ignore
//! use osal_rs::os::*;
//! use alloc::sync::Arc;
//!
//! let counter = Arc::new(Mutex::new(0));
//! let counter_clone = counter.clone();
//!
//! let thread = Thread::new("incrementer", 2048, 5, move || {
//!     let mut guard = counter_clone.lock().unwrap();
//!     *guard += 1;
//! }).unwrap();
//! ```
//!
//! ### Queue Example
//!
//! ```ignore
//! use osal_rs::os::*;
//! use core::time::Duration;
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
//! ```ignore
//! use osal_rs::os::*;
//! use core::time::Duration;
//!
//! let sem = Semaphore::new(1, 1).unwrap();
//!
//! if sem.wait(Duration::from_millis(100)).into() {
//!     // Critical section
//!     sem.signal();
//! }
//! ```
//!
//! ### Timer Example
//!
//! ```ignore
//! use osal_rs::os::*;
//! use core::time::Duration;
//!
//! let timer = Timer::new_with_to_tick(
//!     "periodic",
//!     Duration::from_millis(500),
//!     true,  // auto-reload
//!     None,
//!     |_, _| {
//!         println!("Timer tick");
//!         Ok(None)
//!     }
//! ).unwrap();
//!
//! timer.start_with_to_tick(Duration::from_millis(10));
//! ```
//!
//! ## Module Organization
//!
//! - [`os`] - Main module containing all OS abstractions
//!   - Threads, mutexes, semaphores, queues, event groups, timers
//!   - System-level functions
//!   - Type definitions
//! - [`utils`] - Utility types and error definitions
//! - [`log`] - Logging macros
//! - `traits` - Private module defining the trait abstractions
//! - `freertos` - Private FreeRTOS implementation (enabled with `freertos` feature)
//! - `posix` - Private POSIX implementation (enabled with `posix` feature, planned)
//!
//! ## Features
//!
//! - `freertos` - Enable FreeRTOS support (default)
//! - `posix` - Enable POSIX support (planned)
//! - `std` - Enable standard library support for testing
//! - `disable_panic` - Disable the default panic handler and allocator
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
//! ## Platform Support
//!
//! Currently tested on:
//! - ARM Cortex-M (Raspberry Pi Pico/RP2040, RP2350)
//! - ARM Cortex-M4F (STM32F4 series)
//! - ARM Cortex-M7 (STM32H7 series)
//! - RISC-V (RP2350 RISC-V cores)
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
//! Operations in ISR context have restrictions:
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
//! - Allocations happen on the FreeRTOS heap, not the system heap
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
//!
//! ## License
//!
//! GPL-3.0 - See LICENSE file for details

#![cfg_attr(not(feature = "std"), no_std)]

// Suppress warnings from FreeRTOS FFI bindings being included in multiple modules
#![allow(clashing_extern_declarations)]
#![allow(dead_code)]

extern crate alloc;

/// FreeRTOS implementation of OSAL traits.
///
/// This module contains the concrete implementation of all OSAL abstractions
/// for FreeRTOS, including threads, mutexes, queues, timers, etc.
///
/// Enabled with the `freertos` feature flag (on by default).
#[cfg(feature = "freertos")]
mod freertos;

/// POSIX implementation of OSAL traits (planned).
///
/// This module will contain the implementation for POSIX-compliant systems.
/// Currently under development.
///
/// Enabled with the `posix` feature flag.
#[cfg(feature = "posix")]
mod posix;

pub mod log;

/// Trait definitions for OSAL abstractions.
///
/// This private module defines all the trait interfaces that concrete
/// implementations must satisfy. Traits are re-exported through the `os` module.
mod traits;

pub mod utils;

/// Select FreeRTOS as the active OSAL backend.
#[cfg(feature = "freertos")]
use crate::freertos as osal;

/// Select POSIX as the active OSAL backend.
#[cfg(feature = "posix")]
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

    #[cfg(not(feature = "disable_panic"))]
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
    /// - Active by default
    /// - Disabled with `disable_panic` feature if you want to provide your own allocator
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
    #[cfg(not(feature = "disable_panic"))]
    #[global_allocator]
    pub static ALLOCATOR: Allocator = Allocator;

    /// Duration type and time conversion utilities.
    #[allow(unused_imports)]
    pub use crate::osal::duration::*;
    
    /// Event group synchronization primitives.
    pub use crate::osal::event_group::*;
    
    /// Mutex types and guards for mutual exclusion.
    pub use crate::osal::mutex::*;
    
    /// Queue types for inter-task communication.
    pub use crate::osal::queue::*;
    
    /// Semaphore types for signaling and resource management.
    pub use crate::osal::semaphore::*;
    
    /// System-level functions (scheduler, timing, critical sections).
    pub use crate::osal::system::*;
    
    /// Thread/task management and notification types.
    pub use crate::osal::thread::*;
    
    /// Software timer types for periodic and one-shot callbacks.
    pub use crate::osal::timer::*;
    
    /// All OSAL trait definitions for advanced usage.
    pub use crate::traits::*;
    
    /// RTOS configuration constants and types.
    pub use crate::osal::config as config;
    
    /// Type aliases and common types used throughout OSAL.
    pub use crate::osal::types as types;
    
}

/// Default panic handler for `no_std` environments.
///
/// This panic handler is active when the `disable_panic` feature is **not** enabled.
/// It prints panic information and enters an infinite loop to halt execution.
///
/// # Behavior
///
/// 1. Attempts to print panic information using the `println!` macro
/// 2. Enters an infinite empty loop, halting the program
///
/// # Feature Flag
///
/// - Enabled by default in library mode
/// - Disabled with `disable_panic` feature when users want to provide their own handler
/// - Automatically disabled in examples that use `std`
///
/// # Safety
///
/// This handler is intentionally simple and does not attempt cleanup or recovery.
/// In production embedded systems, consider:
/// - Logging panic info to persistent storage
/// - Performing safe shutdown procedures
/// - Resetting the system via watchdog
///
/// # Custom Panic Handler
///
/// To provide your own panic handler, enable the `disable_panic` feature:
///
/// ```toml
/// [dependencies]
/// osal-rs = { version = "*", features = ["disable_panic"] }
/// ```
///
/// Then define your own `#[panic_handler]` in your application.
#[cfg(not(feature = "disable_panic"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic occurred: {}", info);
    #[allow(clippy::empty_loop)]
    loop {}
}

