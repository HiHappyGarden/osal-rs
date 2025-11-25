use crate::freertos::ffi::{vTaskEndScheduler, vTaskStartScheduler};
#[cfg(all(not(test), not(feature = "std")))]
use crate::freertos::free_rtos_allocator::FreeRTOSAllocator;

#[cfg(all(not(test), not(feature = "std")))]
#[global_allocator]
static GLOBAL: FreeRTOSAllocator = FreeRTOSAllocator;


pub fn os_version() -> &'static str {
    "FreeRTOS V11.2.0"
}


pub fn start_scheduler() {
    unsafe {
        vTaskStartScheduler();
    }
}

pub fn stop_scheduler() {
    unsafe {
        vTaskEndScheduler();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_version() {
        assert_eq!(os_version(), "FreeRTOS V11.2.0");
    }
}