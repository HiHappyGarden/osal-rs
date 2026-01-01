/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

#![cfg_attr(not(feature = "std"), no_std)]

// Suppress warnings from FreeRTOS FFI bindings being included in multiple modules
#![allow(clashing_extern_declarations)]
#![allow(dead_code)]
#![allow(unused_imports)]
extern crate alloc;

#[cfg(feature = "freertos")]
mod freertos;

#[cfg(feature = "posix")]
mod posix;

pub mod log;

mod traits;

pub mod utils;

#[cfg(feature = "freertos")]
use crate::freertos as osal;

#[cfg(feature = "posix")]
use crate::posix as osal;

pub mod os {

    #[cfg(not(feature = "disable_panic"))]
    use crate::osal::allocator::Allocator;


    #[cfg(not(feature = "disable_panic"))]
    #[global_allocator]
    pub static ALLOCATOR: Allocator = Allocator;

    
    pub use crate::osal::duration::*;
    pub use crate::osal::event_group::*;
    pub use crate::osal::mutex::*;
    pub use crate::osal::queue::*;
    pub use crate::osal::semaphore::*;
    pub use crate::osal::system::*;
    pub use crate::osal::thread::*;
    pub use crate::osal::timer::*;
    pub use crate::traits::*;
    pub use crate::osal::config as config;
    pub use crate::osal::types as types;
    
}


// Panic handler for no_std library - only when building as final binary
// Examples with std will provide their own
#[cfg(not(feature = "disable_panic"))]
#[panic_handler]

fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic occurred: {}", info);
    #[allow(clippy::empty_loop)]
    loop {}
}

