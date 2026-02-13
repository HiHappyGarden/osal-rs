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
//!
//! ## Features
//!
//! - `freertos` - Enable FreeRTOS support (default)
//! - `posix` - Enable POSIX support (planned)
//! - `std` - Enable standard library support for testing
//!
//! ## Requirements
//!
//! When using with FreeRTOS:
//! - FreeRTOS must be properly configured
//! - Link the C porting layer from `osal-rs-porting/freertos/`
//! - Set appropriate `FreeRTOSConfig.h` options
//!
//! ## Platform Support
//!
//! Currently tested on:
//! - ARM Cortex-M (Raspberry Pi Pico/RP2040, RP2350)
//! - ARM Cortex-M4F
//! - ARM Cortex-M7
//!
//! ## Safety
//!
//! This library uses `unsafe` internally to interface with C APIs but provides
//! safe Rust abstractions. All public APIs are designed to be memory-safe when
//! used correctly.
//!
//! ## License
//!
//! GPL-3.0 - See LICENSE file for details

#![cfg_attr(not(feature = "std"), no_std)]

// Suppress warnings from FreeRTOS FFI bindings being included in multiple modules
#![allow(clashing_extern_declarations)]
#![allow(dead_code)]
extern crate alloc;

#[cfg(feature = "freertos")]
mod freertos;

#[cfg(feature = "posix")]
mod posix;

pub mod log;

mod traits;

pub mod utils;

#[cfg(feature = "freertos")]
use crate::freertos as osal;

#[cfg(feature = "posix")]
use crate::posix as osal;

pub mod os {

    #[cfg(not(feature = "disable_panic"))]
    use crate::osal::allocator::Allocator;


    #[cfg(not(feature = "disable_panic"))]
    #[global_allocator]
    pub static ALLOCATOR: Allocator = Allocator;

    #[allow(unused_imports)]
    pub use crate::osal::duration::*;
    pub use crate::osal::event_group::*;
    pub use crate::osal::mutex::*;
    pub use crate::osal::queue::*;
    pub use crate::osal::semaphore::*;
    pub use crate::osal::system::*;
    pub use crate::osal::thread::*;
    pub use crate::osal::timer::*;
    pub use crate::traits::*;
    pub use crate::osal::config as config;
    pub use crate::osal::types as types;
    
}


// Panic handler for no_std library - only when building as final binary
// Examples with std will provide their own
#[cfg(not(feature = "disable_panic"))]
#[panic_handler]

fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic occurred: {}", info);
    #[allow(clippy::empty_loop)]
    loop {}
}

