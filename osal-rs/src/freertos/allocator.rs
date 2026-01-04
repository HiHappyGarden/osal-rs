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

//! Global memory allocator using FreeRTOS heap implementation.
//!
//! This module provides a custom allocator that integrates Rust's allocation
//! system with FreeRTOS heap management functions (`pvPortMalloc` and `vPortFree`).
//!
//! # Features
//!
//! - Uses FreeRTOS heap for all Rust allocations
//! - Supports standard allocation operations (alloc, dealloc, realloc, alloc_zeroed)
//! - Thread-safe (FreeRTOS heap functions are thread-safe)
//! - Enables use of Rust's heap types (`Box`, `Vec`, `String`, etc.)
//!
//! # Usage
//!
//! This allocator is automatically set as the global allocator when using
//! the FreeRTOS backend. No manual configuration is required.
//!
//! # Examples
//!
//! ```ignore
//! use alloc::vec::Vec;
//! use alloc::string::String;
//! 
//! // All allocations use FreeRTOS heap
//! let v = Vec::new();
//! let s = String::from("Hello");
//! ```
//!
//! # Safety
//!
//! The allocator relies on FreeRTOS heap being properly configured in
//! FreeRTOSConfig.h with appropriate heap implementation (heap_4.c, heap_5.c, etc.)

use core::ffi::c_void;
use core::alloc::{GlobalAlloc, Layout};

use crate::freertos::ffi::{pvPortMalloc, vPortFree};

/// Global memory allocator using FreeRTOS heap.
///
/// This allocator implements Rust's `GlobalAlloc` trait by forwarding
/// all allocation requests to FreeRTOS heap management functions.
///
/// # Thread Safety
///
/// FreeRTOS heap functions are thread-safe, making this allocator safe
/// to use from multiple tasks simultaneously.
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    /// Allocates memory from the FreeRTOS heap.
    ///
    /// # Parameters
    ///
    /// * `layout` - Memory layout specifying size and alignment requirements
    ///
    /// # Returns
    ///
    /// Pointer to allocated memory, or null if allocation fails
    ///
    /// # Safety
    ///
    /// Returns uninitialized memory. Caller must initialize before use.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            pvPortMalloc(layout.size()) as *mut u8
        }
        
    }

    /// Frees memory previously allocated by `alloc`.
    ///
    /// # Parameters
    ///
    /// * `ptr` - Pointer to memory to free (must have been returned by `alloc`)
    /// * `_layout` - Original layout (not used by FreeRTOS heap)
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer previously returned by `alloc`.
    /// Double-free will cause undefined behavior.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            vPortFree(ptr as *mut c_void);
        }
    }

    /// Allocates zero-initialized memory.
    ///
    /// Allocates memory and initializes all bytes to zero.
    ///
    /// # Parameters
    ///
    /// * `layout` - Memory layout specifying size and alignment
    ///
    /// # Returns
    ///
    /// Pointer to zero-filled memory, or null if allocation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use core::alloc::{Layout, GlobalAlloc};
    /// 
    /// let layout = Layout::from_size_align(1024, 4).unwrap();
    /// let ptr = allocator.alloc_zeroed(layout);
    /// // All 1024 bytes are guaranteed to be zero
    /// ```
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        
        let ptr = unsafe { self.alloc(layout) };
        unsafe {
            if !ptr.is_null() {
                core::ptr::write_bytes( ptr as *mut c_void, 0, layout.size());
            }
        }
        ptr
    }

    /// Reallocates memory to a new size.
    ///
    /// Creates a new allocation of `new_size`, copies data from the old
    /// allocation, and frees the old memory.
    ///
    /// # Parameters
    ///
    /// * `ptr` - Pointer to existing allocation
    /// * `_layout` - Original layout
    /// * `new_size` - New size in bytes
    ///
    /// # Returns
    ///
    /// Pointer to reallocated memory, or null if allocation fails.
    /// If allocation fails, the original memory remains valid.
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid pointer from previous `alloc`
    /// - Copying preserves min(old_size, new_size) bytes
    /// - Old pointer becomes invalid after successful reallocation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Grow allocation from 100 to 200 bytes
    /// let new_ptr = allocator.realloc(old_ptr, old_layout, 200);
    /// // old_ptr is now invalid, use new_ptr
    /// ```
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { pvPortMalloc(new_size) as *mut u8 };

        unsafe {
            if !new_ptr.is_null() && !ptr.is_null() {
                core::ptr::copy_nonoverlapping(ptr, new_ptr, _layout.size().min(new_size));
                vPortFree(ptr as *mut c_void);
            }
        }
        new_ptr
    }
}