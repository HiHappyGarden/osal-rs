#![no_std]
#![no_main]


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
pub use traits::Event as EventTrait;
pub use osal::mutex::*;
pub use traits::Mutex as MutexTrait;
pub use osal::queue::*;
pub use traits::Queue as QueueTrait;
pub use osal::semaphore::*;
pub use traits::Semaphore as SemaphoreTrait;
pub use osal::stream_buffer::*;
pub use traits::StreamBuffer as StreamBufferTrait;
pub use osal::system::*;
pub use osal::thread::*;
pub use traits::Thread as ThreadTrait;
#[allow(unused_imports)]
pub use osal::time::*;
pub use osal::timer::*;
pub use traits::Timer as TimerTrait;
pub use types::*;

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

