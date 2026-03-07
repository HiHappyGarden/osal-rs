/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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

//! Trait definitions for OSAL abstractions.
//!
//! This module defines the trait interfaces that concrete RTOS implementations
//! must satisfy. These traits provide a portable API across different operating systems.
//!
//! # Architecture
//!
//! The OSAL-RS library uses a trait-based design pattern:
//! 1. **Traits** (this module) - Define the interface contracts
//! 2. **Implementations** (freertos, posix) - Provide concrete implementations
//! 3. **Re-exports** (`os` module) - Expose implementations through a unified API
//!
//! This design allows:
//! - Platform independence: Switch RTOS by changing feature flags
//! - Type safety: Compile-time verification of correct usage
//! - Zero-cost abstraction: Traits compile to direct function calls
//! - Extensibility: Add new RTOS backends by implementing the traits
//!
//! # Available Traits
//!
//! ## Synchronization Primitives
//!
//! - [`MutexFn`] - Mutual exclusion with RAII guards
//! - [`RawMutexFn`] - Low-level mutex without guards
//! - [`SemaphoreFn`] - Counting and binary semaphores
//! - [`EventGroupFn`] - Multi-bit synchronization flags
//!
//! ## Communication
//!
//! - [`QueueFn`] - FIFO queue for raw byte messages
//! - [`QueueStreamedFn`] - Type-safe queue using serialization
//! - [`Serialize`] / [`Deserialize`] - Serialization traits for queues
//!
//! ## Threading
//!
//! - [`ThreadFn`] - Thread/task creation and management
//! - [`ThreadNotification`] - Thread notification mechanisms
//! - [`ToPriority`] - Priority conversion trait
//!
//! ## Timers
//!
//! - [`TimerFn`] - Software timer callbacks
//!
//! ## System
//!
//! - [`SystemFn`] - System-level operations (scheduler, timing, critical sections)
//! - [`ToTick`] / [`FromTick`] - Time conversion to/from RTOS ticks
//!
//! ## Utilities
//!
//! - [`BytesHasLen`] - Length queries for serializable types
//!
//! # Naming Convention
//!
//! Traits are re-exported with a `Fn` suffix to avoid naming conflicts with
//! concrete implementation types:
//!
//! ```ignore
//! // Trait definition (in this module)
//! pub trait Thread { ... }
//!
//! // Re-exported as ThreadFn to avoid conflict
//! pub use Thread as ThreadFn;
//!
//! // Concrete type in freertos module
//! pub struct Thread { ... }
//! impl ThreadFn for Thread { ... }
//! ```
//!
//! This allows both the trait and the concrete type to coexist in the same namespace.
//!
//! # Usage
//!
//! Most users should use the `os` module instead of importing traits directly:
//!
//! ```ignore
//! use osal_rs::os::*;  // Gets concrete types
//!
//! let mutex = Mutex::new(0);  // Uses concrete freertos::Mutex
//! ```
//!
//! Advanced users can import traits for generic programming:
//!
//! ```ignore
//! use osal_rs::traits::MutexFn;
//!
//! fn use_mutex<M: MutexFn<i32>>(mutex: &M) {
//!     let guard = mutex.lock();
//!     // ...
//! }
//! ```
//!
//! # Implementation Requirements
//!
//! When implementing these traits for a new RTOS backend:
//! 1. Implement all trait methods faithfully to the documented behavior
//! 2. Ensure thread safety as documented
//! 3. Handle ISR context appropriately (provide `_from_isr` variants where needed)
//! 4. Use appropriate error types from `utils::Error`
//! 5. Follow RAII patterns where applicable (e.g., mutex guards)
//!
//! # See Also
//!
//! - Individual trait modules for detailed documentation
//! - `freertos` module for the FreeRTOS implementation
//! - `os` module for the unified public API

/// Byte serialization and deserialization traits.
mod byte;

/// Event group trait for multi-bit synchronization.
mod event_group;

/// Mutex traits for mutual exclusion with RAII.
mod mutex;

/// Queue traits for inter-task communication.
mod queue;

/// Semaphore trait for counting and binary semaphores.
mod semaphore;

/// System-level RTOS control trait.
mod system;

/// Thread/task management trait.
mod thread;

/// Tick conversion traits for time handling.
mod tick;

/// Software timer trait for callbacks.
mod timer;

// Re-export serialization traits for queue usage
pub use crate::traits::byte::*;

// Re-export event group trait with Fn suffix to avoid naming conflicts
pub use crate::traits::event_group::EventGroup as EventGroupFn;

// Re-export mutex traits with Fn suffix (RawMutex, Mutex, MutexGuard)
pub use crate::traits::mutex::{Mutex as MutexFn, MutexGuard as MutexGuardFn, RawMutex as RawMutexFn};

// Re-export queue traits with Fn suffix (Queue for raw bytes, QueueStreamed for typed messages)
pub use crate::traits::queue::{Queue as QueueFn, QueueStreamed as QueueStreamedFn};

// Re-export semaphore trait with Fn suffix
pub use crate::traits::semaphore::Semaphore as SemaphoreFn;

// Re-export system trait with Fn suffix for scheduler, timing, and critical section control
pub use crate::traits::system::System as SystemFn;

// Re-export thread trait and related types (ThreadParam, function pointers, notifications, priority conversion)
pub use crate::traits::thread::{Thread as ThreadFn, ThreadParam, ThreadFnPtr, ThreadSimpleFnPtr, ThreadNotification, ToPriority};

// Re-export tick conversion traits (ToTick, FromTick)
pub use crate::traits::tick::*;

// Re-export timer trait and related types (TimerParam, function pointer)
pub use crate::traits::timer::{Timer as TimerFn, TimerParam, TimerFnPtr};
