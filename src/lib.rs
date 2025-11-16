

#![cfg_attr(not(feature = "posix"), no_std)]
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
pub use osal::mutex::*;
pub use osal::queue::*;
pub use osal::semaphore::*;
pub use osal::stream_buffer::*;
pub use osal::system::*;
pub use osal::thread::*;
#[allow(unused_imports)]
pub use osal::time::*;
pub use osal::timer::*;
pub use traits::Thread as ThreadTrait;
pub use types::*;


#[cfg(all(not(test), feature = "freertos"))]
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

