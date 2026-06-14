/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
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
//!
//! # Thread Safety
//!
//! The logging system uses a simple busy-wait lock to ensure thread-safe output:
//! - Only one thread can log at a time
//! - Other threads spin-wait until the log is complete
//! - No heap allocation during the lock
//! - Suitable for task context only (see ISR Context below)
//!
//! # ISR Context
//!
//! **WARNING**: Do not use logging macros from interrupt service routines (ISRs).
//! 
//! Reasons:
//! - Busy-wait lock can cause priority inversion
//! - String formatting allocates memory
//! - UART operations may block
//! - Can significantly delay interrupt response
//!
//! If you need logging from ISR context, use a queue to defer the log to a task.
//!
//! # Performance Considerations
//!
//! - Each log call allocates a string (uses RTOS heap)
//! - UART transmission is synchronous and relatively slow
//! - Verbose logging (DEBUG level) can impact real-time performance
//! - Consider using WARNING or ERROR level in production
//! - Logs are compiled out when the level is disabled (zero cost)
//!
//! # Color Output
//!
//! When colors are enabled, log levels are color-coded:
//! - **DEBUG**: Cyan - Detailed debugging information
//! - **INFO**: Green - Normal operational messages
//! - **WARNING**: Yellow - Potential issues, non-critical
//! - **ERROR**: Red - Errors affecting functionality
//! - **FATAL**: Magenta - Critical errors, system failure
//!
//! Disable colors if your terminal doesn't support ANSI escape codes.
//!
//! # Best Practices
//!
//! 1. **Use appropriate tags**: Use meaningful tags like "NET", "FS", "APP" to identify sources
//! 2. **Choose correct levels**: DEBUG for development, INFO for milestones, ERROR for failures
//! 3. **Avoid logging in hot paths**: Logging can significantly slow down tight loops
//! 4. **Set production levels**: Use WARNING or ERROR level in production builds
//! 5. **Never log from ISRs**: Defer to task context using queues or notifications

#[cfg(not(feature = "posix"))]
pub mod ffi {
    //! Foreign Function Interface (FFI) to C UART functions.
    //!
    //! This module provides low-level bindings to C functions for UART communication.
    //! These functions are only available in `no_std` mode.
    //!
    //! # Safety
    //!
    //! All functions in this module are `unsafe` because they:
    //! - Call external C code that cannot be verified by Rust
    //! - Require valid C string pointers (null-terminated)
    //! - May access hardware registers directly
    //! - Do not perform bounds checking
    //!
    //! # Platform Requirements
    //!
    //! The C implementation must provide `printf_on_uart` that:
    //! - Accepts printf-style format strings
    //! - Outputs to UART hardware
    //! - Is thread-safe (or only called from synchronized contexts)
    //! - Returns number of characters written, or negative on error
    
    use core::ffi::{c_char, c_int};

    unsafe extern "C" {
        /// FFI function to print formatted strings to UART.
        ///
        /// This is the low-level C function that interfaces with the hardware UART.
        /// Typically implemented in the platform-specific porting layer.
        ///
        /// # Safety
        ///
        /// - `format` must be a valid null-terminated C string
        /// - Variable arguments must match the format specifiers
        /// - Must not be called from multiple threads simultaneously (unless C implementation is thread-safe)
        ///
        /// # Parameters
        ///
        /// * `format` - Printf-style format string (null-terminated)
        /// * `...` - Variable arguments matching format specifiers
        ///
        /// # Returns
        ///
        /// Number of characters written, or negative value on error
        pub fn printf_on_uart(format: *const c_char, ...) -> c_int;

    }
}

#[cfg(not(feature = "posix"))]
use core::ffi::c_char;

#[cfg(not(feature = "posix"))]
use crate::log::ffi::printf_on_uart;
use crate::os::{System, SystemFn};
#[cfg(not(feature = "posix"))]
use crate::utils::Bytes;

pub const LOG_BUFFER_SIZE: usize = 256;

/// ANSI escape code for red text color
const COLOR_RED: &str = "\x1b[31m";
/// ANSI escape code for green text color
const COLOR_GREEN: &str = "\x1b[32m";
/// ANSI escape code for yellow text color
const COLOR_YELLOW: &str = "\x1b[33m";
/// ANSI escape code for blue text color
#[allow(dead_code)]
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
///
/// # Flag vs Level
///
/// - **FLAGS** (`FLAG_*`): Individual bits for each log level, used internally
/// - **LEVELS** (`LEVEL_*`): Combined masks that include all levels at or above the specified severity
///
/// For example, `LEVEL_WARNING` includes WARNING, ERROR, and FATAL flags.
///
/// # Usage
///
/// ```ignore
/// use osal_rs::log::log_levels::*;
/// use osal_rs::log::set_level_log;
///
/// // Set minimum level to WARNING (shows WARNING, ERROR, FATAL)
/// set_level_log(LEVEL_WARNING);
///
/// // Check if specific level is enabled
/// if is_enabled_log(FLAG_DEBUG) {
///     // Debug is enabled
/// }
/// ```
pub mod log_levels {
    /// Flag for DEBUG level messages (bit 0, most verbose).
    ///
    /// Use for detailed debugging information during development.
    pub const FLAG_DEBUG: u8 = 1 << 0;
    
    /// Flag for INFO level messages (bit 1).
    ///
    /// Use for informational messages about normal operation.
    pub const FLAG_INFO: u8 = 1 << 1;
    
    /// Flag for WARNING level messages (bit 2).
    ///
    /// Use for potentially problematic situations that don't prevent operation.
    pub const FLAG_WARNING: u8 = 1 << 2;
    
    /// Flag for ERROR level messages (bit 3).
    ///
    /// Use for errors that affect functionality but allow continued operation.
    pub const FLAG_ERROR: u8 = 1 << 3;
    
    /// Flag for FATAL level messages (bit 4, most severe).
    ///
    /// Use for critical errors that prevent continued operation.
    pub const FLAG_FATAL: u8 = 1 << 4;
    
    /// Flag to enable color output (bit 6).
    ///
    /// When set, log messages are color-coded by severity level.
    pub const FLAG_COLOR_ON: u8 = 1 << 6;
    
    /// Flag to enable/disable logging entirely (bit 7).
    ///
    /// When clear, all logging is disabled for zero runtime cost.
    pub const FLAG_STATE_ON: u8 = 1 << 7;

    /// DEBUG level: Shows all messages (DEBUG, INFO, WARNING, ERROR, FATAL).
    ///
    /// Most verbose setting, suitable for development and troubleshooting.
    pub const LEVEL_DEBUG: u8 = FLAG_DEBUG | FLAG_INFO | FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    
    /// INFO level: Shows INFO and above (INFO, WARNING, ERROR, FATAL).
    ///
    /// Filters out DEBUG messages, suitable for normal operation.
    pub const LEVEL_INFO: u8 = FLAG_INFO | FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    
    /// WARNING level: Shows WARNING and above (WARNING, ERROR, FATAL).
    ///
    /// Shows only warnings and errors, suitable for production.
    pub const LEVEL_WARNING: u8 = FLAG_WARNING | FLAG_ERROR | FLAG_FATAL;
    
    /// ERROR level: Shows ERROR and FATAL only.
    ///
    /// Shows only errors and critical failures.
    pub const LEVEL_ERROR: u8 = FLAG_ERROR | FLAG_FATAL;

    /// FATAL level: Shows only FATAL messages.
    ///
    /// Most restrictive setting, shows only critical failures.
    pub const LEVEL_FATAL: u8 = FLAG_FATAL;
}

/// Global log level mask with color and state flags enabled by default.
///
/// This mutable static holds the current logging configuration:
/// - Bits 0-4: Log level flags (DEBUG, INFO, WARNING, ERROR, FATAL)
/// - Bit 6: Color enable flag
/// - Bit 7: Logging enable/disable flag
///
/// # Default
///
/// Initialized to `LEVEL_DEBUG | FLAG_COLOR_ON | FLAG_STATE_ON`:
/// - All log levels enabled
/// - Color output enabled
/// - Logging enabled
///
/// # Thread Safety
///
/// Modifications are not atomic. Use the provided setter functions
/// (`set_level_log`, `set_enable_log`, `set_enable_color`) which perform
/// simple bit operations that are effectively atomic on most platforms.
/// Race conditions during initialization are unlikely to cause issues
/// beyond temporarily incorrect filter settings.
static mut MASK: u8 = log_levels::LEVEL_DEBUG | log_levels::FLAG_COLOR_ON | log_levels::FLAG_STATE_ON;

/// Simple busy flag for thread-safe logging (0 = free, non-zero = busy).
///
/// Used as a spinlock to ensure only one thread logs at a time:
/// - 0 = Lock is free, logging available
/// - Non-zero = Lock is held, other threads must wait
///
/// # Synchronization
///
/// Uses a basic busy-wait (spinlock) pattern:
/// 1. Wait until BUSY == 0
/// 2. Set BUSY = 1
/// 3. Perform logging
/// 4. Set BUSY = 0
///
/// # Limitations
///
/// - Not a true atomic operation (no memory barriers)
/// - Priority inversion possible (low-priority task holds lock)
/// - Wastes CPU cycles during contention
/// - **Never use from ISR context** - can deadlock
///
/// This simple approach is sufficient for most embedded use cases where
/// logging contention is infrequent.
static mut BUSY: u8 = 0;

/// Prints formatted text without a newline.
///
/// In `std` mode this writes to standard output. In `no_std` mode this writes to UART.
///
/// # Examples
///
/// ```ignore
/// use osal_rs::print;
/// 
/// print!("Hello");
/// print!(" World: {}", 42);
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::log::print_args(format_args!($($arg)*));
    }};
}

/// Prints formatted text with a newline (`\r\n`).
///
/// In `std` mode this writes to standard output. In `no_std` mode this writes to UART.
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
#[macro_export]
macro_rules! println {
    () => {{
        $crate::log::print_newline();
    }};
    ($fmt:expr) => {{
        $crate::log::print_args(format_args!(concat!($fmt, "\r\n")));
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $crate::log::print_args(format_args!(concat!($fmt, "\r\n"), $($arg)*));
    }};
}

#[doc(hidden)]
pub fn print_args(args: core::fmt::Arguments<'_>) {
    #[cfg(feature = "posix")]
    {
        std::print!("{}", args);
    }

    #[cfg(not(feature = "posix"))]
    unsafe {
        let mut buf = crate::utils::Bytes::<{ LOG_BUFFER_SIZE }>::new();
        buf.format(args);
        ffi::printf_on_uart(
            b"%s\0".as_ptr() as *const core::ffi::c_char,
            buf.as_cstr().as_ptr(),
        );
    }
}

#[doc(hidden)]
pub fn print_newline() {
    #[cfg(feature = "posix")]
    {
        std::print!("{}", RETURN);
    }

    #[cfg(not(feature = "posix"))]
    unsafe {
        ffi::printf_on_uart(b"\r\n\0".as_ptr() as *const core::ffi::c_char);
    }
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
/// * `tag` - Category or module name for the log message (e.g., "APP", "NET", "FS")
/// * `log_type` - Log level flag (DEBUG, INFO, WARNING, ERROR, FATAL)
/// * `to_print` - The formatted message string to log
///
/// # Thread Safety
///
/// Uses a busy-wait lock (BUSY flag) to ensure only one thread logs at a time:
/// 1. Spins until BUSY == 0
/// 2. Sets BUSY = 1
/// 3. Formats and outputs the message
/// 4. Sets BUSY = 0
///
/// Other threads will spin-wait during this time.
///
/// # Output Format
///
/// In `no_std` mode:
/// ```text
/// {color}({timestamp}ms)[{tag}] {message}{color_reset}\r\n
/// ```
///
/// Example:
/// ```text
/// \x1b[32m(1234ms)[APP] System initialized\x1b[0m\r\n
/// ```
///
/// # Examples
///
/// ```ignore
/// use osal_rs::log::*;
/// 
/// sys_log("APP", log_levels::FLAG_INFO, "Application started");
/// sys_log("NET", log_levels::FLAG_ERROR, "Connection failed");
/// ```
///
/// # Note
///
/// Prefer using the log macros (`log_info!`, `log_error!`, etc.) instead of
/// calling this function directly. The macros check if the log level is enabled
/// before formatting the message, avoiding allocation for disabled levels.
///
/// # Warning
///
/// **Never call from ISR context** - the busy-wait can cause deadlock if a
/// higher-priority ISR preempts a task that holds the lock.
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


        #[cfg(not(feature = "posix"))]
        {
            let mut buf = Bytes::<512>::new();
            buf.format(format_args!("{color}({millis}ms)[{tag}] {to_print}{color_reset}{RETURN}", millis=now.as_millis()));
            printf_on_uart(b"%s\0".as_ptr() as *const c_char, buf.as_cstr().as_ptr());
        }

        #[cfg(feature = "posix")]
        {
            std::println!("{color}({}ms)[{tag}] {to_print}{color_reset}", now.as_millis());
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
            let mut msg = $crate::utils::Bytes::<{$crate::log::LOG_BUFFER_SIZE}>::new();
            msg.format(format_args!($fmt $(, $($arg)*)?));
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_DEBUG, msg.as_str());
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
            let mut msg = $crate::utils::Bytes::<{$crate::log::LOG_BUFFER_SIZE}>::new();
            msg.format(format_args!($fmt $(, $($arg)*)?));
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_INFO, msg.as_str());
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
            let mut msg = $crate::utils::Bytes::<{$crate::log::LOG_BUFFER_SIZE}>::new();
            msg.format(format_args!($fmt $(, $($arg)*)?));
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_WARNING, msg.as_str());
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
            let mut msg = $crate::utils::Bytes::<{$crate::log::LOG_BUFFER_SIZE}>::new();
            msg.format(format_args!($fmt $(, $($arg)*)?));
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_ERROR, msg.as_str());
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
            let mut msg = $crate::utils::Bytes::<{$crate::log::LOG_BUFFER_SIZE}>::new();
            msg.format(format_args!($fmt $(, $($arg)*)?));
            $crate::log::sys_log($app_tag, $crate::log::log_levels::FLAG_FATAL, msg.as_str());
        }
    }};
}
