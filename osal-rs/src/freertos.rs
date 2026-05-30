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

//! FreeRTOS implementation of OSAL-RS traits.
//!
//! This module provides concrete implementations of all OSAL-RS traits for
//! the FreeRTOS real-time operating system, enabling portable real-time
//! applications across different RTOS implementations.
//!
//! # Overview
//!
//! FreeRTOS is a popular open-source RTOS for embedded systems, providing:
//! - Preemptive multitasking with priority-based scheduling
//! - Multiple synchronization primitives (mutexes, semaphores, queues)
//! - Software timers
//! - Memory management
//! - Low memory footprint
//!
//! This module wraps the FreeRTOS C API with safe Rust abstractions that
//! implement the OSAL-RS trait interfaces.
//!
//! # Modules
//!
//! - [`allocator`] - Memory allocation using FreeRTOS heap
//! - [`config`] - FreeRTOS configuration constants
//! - [`duration`] - Duration type conversions for FreeRTOS ticks
//! - [`event_group`] - Event group synchronization primitives
//! - [`mutex`] - Mutex implementations with priority inheritance
//! - [`queue`] - Message queues for inter-task communication
//! - [`semaphore`] - Binary and counting semaphores
//! - [`system`] - System-level RTOS control and timing
//! - [`thread`] - Task/thread creation and management
//! - [`timer`] - Software timers for delayed/periodic callbacks
//! - [`types`] - FreeRTOS-specific type definitions
//!
//! # Requirements
//!
//! - FreeRTOS kernel source code (linked at build time)
//! - Proper FreeRTOSConfig.h configuration for your platform
//! - C compiler for building FreeRTOS kernel
//!
//! # Configuration
//!
//! FreeRTOS behavior is configured at compile time via `FreeRTOSConfig.h`.
//! Key settings include:
//! - `configTICK_RATE_HZ` - System tick frequency (typically 100-1000 Hz)
//! - `configMAX_PRIORITIES` - Number of priority levels
//! - `configTOTAL_HEAP_SIZE` - Heap size for dynamic allocation
//! - `configUSE_PREEMPTION` - Enable preemptive scheduling
//! - `configUSE_MUTEXES` - Enable mutex support
//! - `configUSE_TIMERS` - Enable software timer support
//!
//! # Usage
//!
//! Types from this module implement the traits defined in [`crate::traits`],
//! allowing you to write portable code that works across different RTOS
//! implementations.
//!
//! ## Example: Creating a Thread
//!
//! ```ignore
//! use osal_rs::freertos::thread::Thread;
//! use osal_rs::freertos::system::System;
//! use osal_rs::traits::Thread as ThreadTrait;
//! use osal_rs::traits::System as SystemTrait;
//!
//! let mut thread = Thread::new("worker", 1024, 5);
//! thread.spawn_simple(|| {
//!     loop {
//!         println!("Working...");
//!         System::delay(1000);
//!     }
//! }).unwrap();
//!
//! System::start();  // Start FreeRTOS scheduler
//! ```
//!
//! ## Example: Using a Queue
//!
//! ```ignore
//! use osal_rs::freertos::queue::Queue;
//! use osal_rs::traits::Queue as QueueTrait;
//!
//! let queue = Queue::new(10, 16).unwrap();
//!
//! // Producer
//! let data = [1, 2, 3, 4];
//! queue.post(&data, 100).unwrap();
//!
//! // Consumer
//! let mut buffer = [0u8; 16];
//! queue.fetch(&mut buffer, 100).unwrap();
//! ```
//!
//! # Platform Support
//!
//! FreeRTOS supports many architectures including:
//! - ARM Cortex-M (M0, M0+, M3, M4, M7, M33, M55)
//! - ARM Cortex-A
//! - RISC-V
//! - x86
//! - And many others
//!
//! Each platform requires a port-specific configuration and startup code.
//!
//! # Safety
//!
//! This module uses FFI to call FreeRTOS C functions. The Rust wrappers
//! ensure memory safety through:
//! - Proper lifetime management
//! - Type safety
//! - Resource cleanup (RAII)
//! - Checked conversions
//!
//! # Performance
//!
//! FreeRTOS is designed for resource-constrained embedded systems with:
//! - Fast context switches (typically < 1µs on modern MCUs)
//! - Small memory footprint (< 10KB with typical configuration)
//! - Low interrupt latency
//! - Efficient priority-based scheduler

/// Memory allocator using FreeRTOS heap.
/// FreeRTOS FFI (Foreign Function Interface) bindings.
///
/// This module is private and contains unsafe C bindings to the FreeRTOS kernel.
#[macro_use]
mod ffi;

pub(crate) mod allocator;

/// FreeRTOS configuration constants and utilities.
pub mod config;

/// Duration type implementations for FreeRTOS tick conversion.
pub(crate) mod duration;

/// Event group synchronization primitives.
pub(crate) mod event_group;

/// Mutex implementations with optional priority inheritance.
pub(crate) mod mutex;

/// Message queue implementations for inter-task communication.
pub(crate) mod queue;

/// Binary and counting semaphore implementations.
pub(crate) mod semaphore;

/// System-level RTOS control, timing, and scheduler management.
pub(crate) mod system;

/// Task/thread creation, management, and notifications.
pub(crate) mod thread;

/// Software timer implementations for delayed and periodic callbacks.
pub(crate) mod timer;

/// FreeRTOS-specific type definitions and aliases.
pub mod types;
