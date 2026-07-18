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