#![cfg_attr(not(feature = "std"), no_std)]

// Suppress warnings from FreeRTOS FFI bindings being included in multiple modules
#![allow(clashing_extern_declarations)]
#![allow(dead_code)]
extern crate alloc;

pub mod traits;

#[cfg(feature = "freertos")]
mod freertos;
#[cfg(feature = "posix")]
mod posix;

#[cfg(feature = "freertos")]
#[allow(unused_imports)]
use crate::freertos as osal;
#[cfg(feature = "posix")]
#[allow(unused_imports)]
use crate::posix as osal;



// pub use osal::event::*;
// pub use traits::EventTrait;
// pub use osal::mutex::*;
// pub use traits::MutexTrait;
// pub use osal::queue::*;
// pub use traits::QueueTrait;
// pub use osal::semaphore::*;
// pub use traits::SemaphoreTrait;
// pub use osal::stream_buffer::*;
// pub use traits::StreamBufferTrait;
pub use osal::system::*;
pub use traits::system::System as SystemTrait;
pub use osal::thread::*;
// pub use traits::ThreadTrait;
// #[allow(unused_imports)]
// pub use osal::time::*;
// pub use osal::timer::*;
// pub use traits::TimerTrait;
pub use osal::config as config;
pub use osal::types as types;

// Panic handler for no_std library - only when building as final binary
// Examples with std will provide their own
#[cfg(not(feature = "disable_panic"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

