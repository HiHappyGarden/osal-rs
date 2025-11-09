

#![no_std]
#![no_main]
#![allow(dead_code)]
extern crate alloc;

mod commons;
#[cfg(feature = "freertos")]
mod freertos;
#[cfg(feature = "posix")]
mod posix;
mod traits;

#[cfg(feature = "freertos")]
use crate::freertos as osal;
#[cfg(feature = "posix")]
use crate::posix as osal;

pub use osal::event::*;
#[allow(unused_imports)]
pub use osal::memory::*;
pub use osal::mutex::*;
pub use osal::queue::*;
pub use osal::semaphore::*;
pub use osal::stream_buffer::*;
pub use osal::thread::*;
#[allow(unused_imports)]
pub use osal::time::*;
pub use osal::timer::*;





pub fn init() {


    
}

#[cfg(feature = "freertos")]
pub fn os_version() -> &'static str {
    "FreeRTOS V11.2.0"
}

#[cfg(feature = "posix")]
pub fn os_version() -> &'static str {
    "POSIX"
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
