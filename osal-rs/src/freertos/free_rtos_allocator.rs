use core::ffi::c_void;
//use crate::freertos::ffi::{pvPortMalloc, vPortFree};
use core::{alloc::{GlobalAlloc, Layout}};


pub struct FreeRTOSAllocator;

unsafe impl GlobalAlloc for FreeRTOSAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
  //          pvPortMalloc(layout.size()) as *mut u8
        }
        
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
    //        vPortFree(ptr as *mut c_void);
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