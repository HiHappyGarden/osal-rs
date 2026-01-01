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

#[cfg(not(feature = "std"))]
pub mod ffi {
    use core::ffi::{c_char, c_int};

    unsafe extern "C" {
        pub fn printf_on_uart(format: *const c_char, ...) -> c_int;

    }
}

use core::ffi::c_char;

use alloc::{ffi::CString, format};

use crate::log::ffi::printf_on_uart;
use crate::os::{System, SystemFn};


const COLOR_RED: &str = "\x1b[31m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_BLUE: &str = "\x1b[34m";
const COLOR_MAGENTA: &str = "\x1b[35m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_RESET: &str = "\x1b[0m";
pub const RETURN: &str = "\r\n";

pub mod log_levels {
    pub const FLAG_DEBUG: u8 = 1 << 0;
    pub const FLAG_INFO: u8 = 1 << 1;
    pub const FLAG_WARNING: u8 = 1 << 2;
    pub const FLAG_ERROR: u8 = 1 << 3;
    pub const FLAG_FATAL: u8 = 1 << 4;
    pub const FLAG_COLOR_ON: u8 = 1 << 6;
    pub const FLAG_STATE_ON: u8 = 1 << 7;

    pub const LEVEL_DEBUG: u8 = FLAG_DEBUG | FLAG_INFO | FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    pub const LEVEL_INFO: u8 = FLAG_INFO | FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    pub const LEVEL_WARNING: u8 = FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    pub const LEVEL_ERROR: u8 = FLAG_ERROR | FLAG_FATAL;

    pub const LEVEL_FATAL: u8 = FLAG_FATAL;
}

static mut MASK: u8 = log_levels::LEVEL_DEBUG | log_levels::FLAG_COLOR_ON | log_levels::FLAG_STATE_ON;
static mut BUSY: u8 = 0;

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        unsafe {
            use alloc::string::ToString;
            let formatted = alloc::format!($($arg)*);
            if let Ok(c_str) = alloc::ffi::CString::new(formatted) {
                $crate::log::ffi::printf_on_uart(b"%s\0".as_ptr() as *const core::ffi::c_char, c_str.as_ptr());
            }
        }
    }};
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\r\n")
    };
    ($fmt:expr) => {{
        unsafe {
            use alloc::string::ToString;
            let formatted = alloc::format!(concat!($fmt, "\r\n"));
            if let Ok(c_str) = alloc::ffi::CString::new(formatted) {
                $crate::log::ffi::printf_on_uart(b"%s\0".as_ptr() as *const core::ffi::c_char, c_str.as_ptr());
            }
        }
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        unsafe {
            use alloc::string::ToString;
            let formatted = alloc::format!(concat!($fmt, "\r\n"), $($arg)*);
            if let Ok(c_str) = alloc::ffi::CString::new(formatted) {
                $crate::log::ffi::printf_on_uart(b"%s\0".as_ptr() as *const core::ffi::c_char, c_str.as_ptr());
            }
        }
    }};
}

pub fn set_level_log(level: u8) {
    unsafe {
        MASK =
            (MASK & log_levels::FLAG_STATE_ON) | (level & !log_levels::FLAG_STATE_ON);
    }
}

pub fn set_enable_log(enabled: bool) {
    unsafe {
        if enabled {
            MASK |= log_levels::FLAG_STATE_ON;
        } else {
            MASK &= !log_levels::FLAG_STATE_ON;
        }
    }
}

pub fn get_enable_log() -> bool {
    unsafe { (MASK & log_levels::FLAG_STATE_ON) != 0 }
}

pub fn is_enabled_log(log_type: u8) -> bool {
    unsafe { (MASK & log_levels::FLAG_STATE_ON) != 0 && (MASK & log_type) != 0 }
}

pub fn get_level_log() -> u8 {
    unsafe { MASK & !log_levels::FLAG_STATE_ON & !log_levels::FLAG_COLOR_ON }
}

pub fn set_enable_color(enabled: bool) {
    unsafe {
        if enabled {
            MASK |= log_levels::FLAG_COLOR_ON;
        } else {
            MASK &= !log_levels::FLAG_COLOR_ON;
        }
    }
}



pub fn sys_log(tag: &str, log_type: u8, to_print: &str) {
    unsafe {
        while BUSY != 0 {}
        BUSY = 1;

        let mut color_reset = COLOR_RESET;
        let color = if MASK & log_levels::FLAG_COLOR_ON == log_levels::FLAG_COLOR_ON {

            match log_type {
                log_levels::FLAG_DEBUG => COLOR_CYAN,
                log_levels::FLAG_INFO => COLOR_GREEN,
                log_levels::FLAG_WARNING => COLOR_YELLOW,
                log_levels::FLAG_ERROR => COLOR_RED,
                log_levels::FLAG_FATAL => COLOR_MAGENTA,
                _ => COLOR_RESET,
            }
        } else {
            color_reset = "";
            ""
        };


        let now = System::get_current_time_us();


        #[cfg(not(feature = "std"))]
        {
            let formatted = format!("{color}({millis}ms)[{tag}] {to_print}{color_reset}{RETURN}", millis=now.as_millis());
            if let Ok(c_str) = CString::new(formatted) {
                printf_on_uart(b"%s\0".as_ptr() as *const c_char, c_str.as_ptr());
            }
        }

        #[cfg(feature = "std")]
        {
            print!("{}[{}] ", color, tag);
            core::fmt::write(&mut core::fmt::Formatter::new(), args).unwrap();
            print!("{}", COLOR_RESET);
            print!("\r\n");
        }

        BUSY = 0;
    }
}

#[macro_export]
macro_rules! log_debug {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_DEBUG) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_DEBUG, &msg);
        }
    }};
}

#[macro_export]
macro_rules! log_info {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_INFO) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_INFO, &msg);
        }
    }};
}

#[macro_export]
macro_rules! log_warning {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_WARNING) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_WARNING, &msg);
        }
    }};
}

#[macro_export]
macro_rules! log_error {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_ERROR) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_ERROR, &msg);
        }
    }};
}

#[macro_export]
macro_rules! log_fatal {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_FATAL) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_FATAL, &msg);
        }
    }};
}
