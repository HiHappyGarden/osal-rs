
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