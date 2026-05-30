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

use core::ffi::c_void;

pub type TickType = u64;
pub type BaseType = i64;
pub type UBaseType = u64;
pub type StackType = usize;

pub type ThreadHandle = *const c_void;
pub type QueueHandle = *const c_void;
pub type SemaphoreHandle = *const c_void;
pub type EventGroupHandle = *const c_void;
pub type TimerHandle = *const c_void;
pub type MutexHandle = *const c_void;

pub type EventBits = TickType;