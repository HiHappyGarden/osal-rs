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

use core::{alloc::{GlobalAlloc, Layout}, ffi::c_void};

use crate::posix::ffi::{free, malloc, realloc};

pub struct POSIXAllocator;

unsafe impl GlobalAlloc for POSIXAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { 
            malloc(layout.size() as u64) as *mut u8 
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            free(ptr as *mut c_void);
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.alloc(layout) };
        unsafe {
            if !ptr.is_null() {
                core::ptr::write_bytes( ptr as *mut c_void, 0, layout.size());
            }
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { realloc(ptr as *mut c_void, new_size as u64) as *mut u8 };

        unsafe {
            if !new_ptr.is_null() && !ptr.is_null() {
                core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
            }
        }
        new_ptr
    }
}