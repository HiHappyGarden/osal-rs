#[cfg(all(not(test), not(feature = "std")))]
use crate::posix::posix_allocator::POSIXAllocator;

#[cfg(all(not(test), not(feature = "std")))]
#[global_allocator]
static GLOBAL: POSIXAllocator = POSIXAllocator;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_version() {
        assert_eq!(os_version(), "POSIX");
    }
}