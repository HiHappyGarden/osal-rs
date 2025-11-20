static mut START_MAIN_LOOP : bool = false;

pub fn os_version() -> &'static str {
    "POSIX"
}


pub fn start_scheduler() {
    unsafe { START_MAIN_LOOP = true; }
    loop {
        unsafe {
            if !START_MAIN_LOOP {
                break;
            }
        }
    }
}

pub fn stop_scheduler() {
    unsafe { START_MAIN_LOOP = false; }
}

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

const ARENA_SIZE: usize = 128 * 1024;
const MAX_SUPPORTED_ALIGN: usize = 4096;
#[repr(C, align(4096))] // 4096 == MAX_SUPPORTED_ALIGN
struct SimpleAllocator {
    arena: UnsafeCell<[u8; ARENA_SIZE]>,
    remaining: AtomicUsize, // we allocate from the top, counting down
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    arena: UnsafeCell::new([0x55; ARENA_SIZE]),
    remaining: AtomicUsize::new(ARENA_SIZE),
};

unsafe impl Sync for SimpleAllocator {}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // `Layout` contract forbids making a `Layout` with align=0, or align not power of 2.
        // So we can safely use a mask to ensure alignment without worrying about UB.
        let align_mask_to_round_down = !(align - 1);

        if align > MAX_SUPPORTED_ALIGN {
            return null_mut();
        }

        let mut allocated = 0;
        if self
            .remaining
            .fetch_update(Relaxed, Relaxed, |mut remaining| {
                if size > remaining {
                    return None;
                }
                remaining -= size;
                remaining &= align_mask_to_round_down;
                allocated = remaining;
                Some(remaining)
            })
            .is_err()
        {
            return null_mut();
        };
        unsafe { self.arena.get().cast::<u8>().add(allocated) }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_version() {
        assert_eq!(os_version(), "POSIX");
    }
}