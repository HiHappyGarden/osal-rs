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

//! FreeRTOS type definitions and handle wrappers.
//!
//! This module provides type aliases and handle types that interface with FreeRTOS.
//! All types are generated from the FreeRTOS configuration at build time.

// Include build-time generated types from FreeRTOS configuration
include!(concat!(env!("OUT_DIR"), "/types_generated.rs"));    

/// FreeRTOS opaque handle types for OS primitives.
/// These handles are used to reference FreeRTOS objects.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::types::*;
/// 
/// // ThreadHandle is used internally by Thread struct
/// let handle: ThreadHandle = std::ptr::null_mut();
/// ```
pub use super::ffi::{ThreadHandle, QueueHandle, SemaphoreHandle, EventGroupHandle, TimerHandle, MutexHandle};

/// Type alias for event group bits.
///
/// Represents a set of event flags where each bit can be set or cleared independently.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::os::types::EventBits;
/// use osal_rs::os::EventGroup;
/// 
/// let event_group = EventGroup::new().unwrap();
/// let bits: EventBits = 0b0101; // Set bits 0 and 2
/// event_group.set(bits);
/// ```
pub type EventBits = TickType;
