#![cfg_attr(not(feature = "std"), no_std)]

// Suppress warnings from FreeRTOS FFI bindings being included in multiple modules
#![allow(clashing_extern_declarations)]
#![allow(dead_code)]
extern crate alloc;

pub mod types;
#[cfg(feature = "freertos")]
mod freertos;
#[cfg(feature = "posix")]
mod posix;
pub mod traits;

#[cfg(feature = "freertos")]
use crate::freertos as osal;
#[cfg(feature = "posix")]
use crate::posix as osal;

pub use osal::event::*;
pub use traits::EventTrait;
pub use osal::mutex::*;
pub use traits::MutexTrait;
pub use osal::queue::*;
pub use traits::QueueTrait;
pub use osal::semaphore::*;
pub use traits::SemaphoreTrait;
pub use osal::stream_buffer::*;
pub use traits::StreamBufferTrait;
pub use osal::system::*;
pub use osal::thread::*;
pub use traits::ThreadTrait;
#[allow(unused_imports)]
pub use osal::time::*;
pub use osal::timer::*;
pub use traits::TimerTrait;
pub use types::*;

// Panic handler for no_std library - only when building as final binary
// Examples with std will provide their own
#[cfg(all(not(test), not(feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        #[cfg(feature = "freertos")]
        assert_eq!(os_version(), "FreeRTOS V11.2.0");
        
        #[cfg(feature = "posix")]
        assert_eq!(os_version(), "POSIX");
    }
}

