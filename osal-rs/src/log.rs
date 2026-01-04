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

//! Logging system for embedded environments.
//!
//! Provides a flexible logging system with multiple severity levels and color support.
//! Designed for no-std environments with UART output support.
//!
//! # Features
//!
//! - Multiple log levels (DEBUG, INFO, WARNING, ERROR, FATAL)
//! - Color-coded output support (ANSI colors)
//! - Timestamp logging with millisecond precision
//! - Thread-safe logging with busy-wait synchronization
//! - Configurable log level masking
//! - Zero-cost when logs are disabled
//!
//! # Examples
//!
//! ## Basic logging
//!
//! ```ignore
//! use osal_rs::{log_info, log_error, log_debug};
//! 
//! log_info!("APP", "Application started");
//! log_debug!("APP", "Counter value: {}", 42);
//! log_error!("APP", "Failed to initialize: {}", error_msg);
//! ```
//!
//! ## Configuring log levels
//!
//! ```ignore
//! use osal_rs::log::*;
//! 
//! // Set log level to WARNING and above
//! set_level_log(log_levels::LEVEL_WARNING);
//! 
//! // Enable/disable logging
//! set_enable_log(true);
//! 
//! // Enable/disable color output
//! set_enable_color(true);
//! ```
//!
//! ## Using print macros
//!
//! ```ignore
//! use osal_rs::{print, println};
//! 
//! print!("Hello");
//! println!(" World!");
//! println!("Value: {}", 123);
//! ```

#[cfg(not(feature = "std"))]
pub mod ffi {
    use core::ffi::{c_char, c_int};

    unsafe extern "C" {
        /// FFI function to print formatted strings to UART.
        ///
        /// This is the low-level C function that interfaces with the hardware UART.
        pub fn printf_on_uart(format: *const c_char, ...) -> c_int;

    }
}

use core::ffi::c_char;

use alloc::{ffi::CString, format};

use crate::log::ffi::printf_on_uart;
use crate::os::{System, SystemFn};


/// ANSI escape code for red text color
const COLOR_RED: &str = "\x1b[31m";
/// ANSI escape code for green text color
const COLOR_GREEN: &str = "\x1b[32m";
/// ANSI escape code for yellow text color
const COLOR_YELLOW: &str = "\x1b[33m";
/// ANSI escape code for blue text color
const COLOR_BLUE: &str = "\x1b[34m";
/// ANSI escape code for magenta text color
const COLOR_MAGENTA: &str = "\x1b[35m";
/// ANSI escape code for cyan text color
const COLOR_CYAN: &str = "\x1b[36m";
/// ANSI escape code to reset all text attributes
const COLOR_RESET: &str = "\x1b[0m";
/// Carriage return + line feed for proper terminal output
pub const RETURN: &str = "\r\n";

/// Log level flags and level configurations.
///
/// This module defines bit flags for different log levels and combined
/// level masks for filtering log messages.
pub mod log_levels {
    /// Flag for DEBUG level messages (most verbose)
    pub const FLAG_DEBUG: u8 = 1 << 0;
    /// Flag for INFO level messages
    pub const FLAG_INFO: u8 = 1 << 1;
    /// Flag for WARNING level messages
    pub const FLAG_WARNING: u8 = 1 << 2;
    /// Flag for ERROR level messages
    pub const FLAG_ERROR: u8 = 1 << 3;
    /// Flag for FATAL level messages (most severe)
    pub const FLAG_FATAL: u8 = 1 << 4;
    /// Flag to enable color output
    pub const FLAG_COLOR_ON: u8 = 1 << 6;
    /// Flag to enable/disable logging entirely
    pub const FLAG_STATE_ON: u8 = 1 << 7;

    /// DEBUG level: Shows all messages
    pub const LEVEL_DEBUG: u8 = FLAG_DEBUG | FLAG_INFO | FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    /// INFO level: Shows INFO and above
    pub const LEVEL_INFO: u8 = FLAG_INFO | FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    /// WARNING level: Shows WARNING and above
    pub const LEVEL_WARNING: u8 = FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    /// ERROR level: Shows ERROR and FATAL only
    pub const LEVEL_ERROR: u8 = FLAG_ERROR | FLAG_FATAL;

    /// FATAL level: Shows only FATAL messages
    pub const LEVEL_FATAL: u8 = FLAG_FATAL;
}

/// Global log level mask with color and state flags enabled by default
static mut MASK: u8 = log_levels::LEVEL_DEBUG | log_levels::FLAG_COLOR_ON | log_levels::FLAG_STATE_ON;
/// Simple busy flag for thread-safe logging (0 = free, non-zero = busy)
static mut BUSY: u8 = 0;

/// Prints formatted text to UART without a newline.
///
/// This macro is only available in no-std mode. In std mode, use the standard `print!` macro.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::print;
/// 
/// print!("Hello");
/// print!(" World: {}", 42);
/// ```
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

/// Prints formatted text to UART with a newline (\r\n).
///
/// This macro is only available in no-std mode. In std mode, use the standard `println!` macro.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::println;
/// 
/// println!("Hello World");
/// println!("Value: {}", 42);
/// println!();  // Just a newline
/// ```
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

/// Sets the log level threshold.
///
/// Only log messages at or above this level will be displayed.
///
/// # Parameters
///
/// * `level` - Log level (use constants from `log_levels` module)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::*;
/// 
/// // Show only warnings and errors
/// set_level_log(log_levels::LEVEL_WARNING);
/// 
/// // Show all messages
/// set_level_log(log_levels::LEVEL_DEBUG);
/// ```
pub fn set_level_log(level: u8) {
    unsafe {
        MASK =
            (MASK & log_levels::FLAG_STATE_ON) | (level & !log_levels::FLAG_STATE_ON);
    }
}

/// Enables or disables all logging.
///
/// When disabled, all log macros become no-ops for zero runtime cost.
///
/// # Parameters
///
/// * `enabled` - `true` to enable logging, `false` to disable
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::set_enable_log;
/// 
/// set_enable_log(false);  // Disable all logging
/// // ... logs will not be printed ...
/// set_enable_log(true);   // Re-enable logging
/// ```
pub fn set_enable_log(enabled: bool) {
    unsafe {
        if enabled {
            MASK |= log_levels::FLAG_STATE_ON;
        } else {
            MASK &= !log_levels::FLAG_STATE_ON;
        }
    }
}

/// Checks if logging is currently enabled.
///
/// # Returns
///
/// `true` if logging is enabled, `false` otherwise
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::get_enable_log;
/// 
/// if get_enable_log() {
///     println!("Logging is active");
/// }
/// ```
pub fn get_enable_log() -> bool {
    unsafe { (MASK & log_levels::FLAG_STATE_ON) != 0 }
}

/// Checks if a specific log level is enabled.
///
/// # Parameters
///
/// * `log_type` - Log level flag to check
///
/// # Returns
///
/// `true` if the log level is enabled, `false` otherwise
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::*;
/// 
/// if is_enabled_log(log_levels::FLAG_DEBUG) {
///     // Debug logging is active
/// }
/// ```
pub fn is_enabled_log(log_type: u8) -> bool {
    unsafe { (MASK & log_levels::FLAG_STATE_ON) != 0 && (MASK & log_type) != 0 }
}

/// Gets the current log level threshold.
///
/// # Returns
///
/// Current log level mask (without state and color flags)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::*;
/// 
/// let level = get_level_log();
/// ```
pub fn get_level_log() -> u8 {
    unsafe { MASK & !log_levels::FLAG_STATE_ON & !log_levels::FLAG_COLOR_ON }
}

/// Enables or disables color output.
///
/// When enabled, log messages are color-coded by severity:
/// - DEBUG: Cyan
/// - INFO: Green  
/// - WARNING: Yellow
/// - ERROR: Red
/// - FATAL: Magenta
///
/// # Parameters
///
/// * `enabled` - `true` to enable colors, `false` for plain text
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::set_enable_color;
/// 
/// set_enable_color(true);   // Enable colored output
/// set_enable_color(false);  // Disable colors
/// ```
pub fn set_enable_color(enabled: bool) {
    unsafe {
        if enabled {
            MASK |= log_levels::FLAG_COLOR_ON;
        } else {
            MASK &= !log_levels::FLAG_COLOR_ON;
        }
    }
}



/// Core logging function that outputs formatted log messages.
///
/// This is the low-level function called by all log macros. It handles:
/// - Thread-safe output using a busy-wait lock
/// - Color formatting based on log level
/// - Timestamp prefixing with millisecond precision
/// - Tag prefixing for message categorization
///
/// # Parameters
///
/// * `tag` - Category or module name for the log message
/// * `log_type` - Log level flag (DEBUG, INFO, WARNING, ERROR, FATAL)
/// * `to_print` - The message to log
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::*;
/// 
/// sys_log("APP", log_levels::FLAG_INFO, "Application started");
/// ```
///
/// # Note
///
/// Prefer using the log macros (`log_info!`, `log_error!`, etc.) instead of
/// calling this function directly.
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

/// Logs a DEBUG level message.
///
/// Debug messages are the most verbose and typically used during development.
/// Color: Cyan (if colors are enabled)
///
/// # Parameters
///
/// * `app_tag` - Category or module identifier
/// * `fmt` - Format string
/// * `arg` - Optional format arguments
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log_debug;
/// 
/// log_debug!("APP", "Initializing subsystem");
/// log_debug!("APP", "Counter: {}, Status: {}", counter, status);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_DEBUG) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_DEBUG, &msg);
        }
    }};
}

/// Logs an INFO level message.
///
/// Informational messages about normal application operation.
/// Color: Green (if colors are enabled)
///
/// # Parameters
///
/// * `app_tag` - Category or module identifier
/// * `fmt` - Format string  
/// * `arg` - Optional format arguments
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log_info;
/// 
/// log_info!("APP", "System initialized successfully");
/// log_info!("NET", "Connected to server at {}", ip_addr);
/// ```
#[macro_export]
macro_rules! log_info {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_INFO) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_INFO, &msg);
        }
    }};
}

/// Logs a WARNING level message.
///
/// Warning messages indicate potential issues that don't prevent operation.
/// Color: Yellow (if colors are enabled)
///
/// # Parameters
///
/// * `app_tag` - Category or module identifier
/// * `fmt` - Format string
/// * `arg` - Optional format arguments
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log_warning;
/// 
/// log_warning!("MEM", "Memory usage above 80%");
/// log_warning!("SENSOR", "Temperature high: {} C", temp);
/// ```
#[macro_export]
macro_rules! log_warning {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_WARNING) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_WARNING, &msg);
        }
    }};
}

/// Logs an ERROR level message.
///
/// Error messages indicate failures that affect functionality.
/// Color: Red (if colors are enabled)
///
/// # Parameters
///
/// * `app_tag` - Category or module identifier
/// * `fmt` - Format string
/// * `arg` - Optional format arguments
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log_error;
/// 
/// log_error!("FS", "Failed to open file");
/// log_error!("NET", "Connection timeout: {}", error);
/// ```
#[macro_export]
macro_rules! log_error {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_ERROR) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_ERROR, &msg);
        }
    }};
}

/// Logs a FATAL level message.
///
/// Fatal messages indicate critical errors that prevent continued operation.
/// Color: Magenta (if colors are enabled)
///
/// # Parameters
///
/// * `app_tag` - Category or module identifier
/// * `fmt` - Format string
/// * `arg` - Optional format arguments
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log_fatal;
/// 
/// log_fatal!("SYS", "Kernel panic!");
/// log_fatal!("HW", "Hardware fault detected: {}", fault_code);
/// ```
#[macro_export]
macro_rules! log_fatal {
    ($app_tag:expr, $fmt:expr $(, $($arg:tt)*)?) => {{
        if $crate::log::is_enabled_log($crate::log::log_levels::FLAG_FATAL) {
            let msg = alloc::format!($fmt $(, $($arg)*)?);
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_FATAL, &msg);
        }
    }};
}
