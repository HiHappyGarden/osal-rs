

#![no_std]
#![allow(dead_code)]
extern crate alloc;

pub mod commons;
#[cfg(feature = "freertos")]
mod freertos;
#[cfg(feature = "posix")]
mod posix;
pub mod traits;
pub mod error;

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
pub use osal::system::*;
pub use osal::thread::*;
#[allow(unused_imports)]
pub use osal::time::*;
pub use osal::timer::*;
pub use traits::Thread as ThreadTrait;

#[macro_export]
macro_rules! ms_to_us {
    ($ms:expr) => {
        { ($ms as u64) * 1_000 }
    };
}

#[macro_export]
macro_rules! sec_to_us {
    ($sec:expr) => {
        { ($sec as u64) * 1_000_000 }
    };
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
