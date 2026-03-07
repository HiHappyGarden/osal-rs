/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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

//! FreeRTOS configuration access macros.
//!
//! This module provides macros to access FreeRTOS configuration values at runtime
//! through FFI calls. These values are typically defined in FreeRTOSConfig.h and
//! are made available to Rust code through C wrapper functions.
//!
//! # Available Configuration Values
//!
//! - **CPU Clock**: System clock frequency in Hz
//! - **Tick Rate**: RTOS tick frequency (ticks per second)
//! - **Max Priorities**: Number of priority levels available
//! - **Minimal Stack Size**: Minimum stack size for tasks
//! - **Max Task Name Length**: Maximum characters in task names
//!
//! # Examples
//!
//! ```ignore
//! use osal_rs::{tick_rate_hz, cpu_clock_hz, max_priorities};
//! 
//! let tick_rate = tick_rate_hz!();
//! println!("Tick rate: {} Hz", tick_rate);
//! 
//! let cpu_freq = cpu_clock_hz!();
//! println!("CPU frequency: {} Hz", cpu_freq);
//! 
//! let priorities = max_priorities!();
//! println!("Max priorities: {}", priorities);
//! ```

/// FFI declarations for FreeRTOS configuration functions.
///
/// These C functions are implemented in the porting layer and return
/// configuration values from FreeRTOSConfig.h.
pub mod ffi {
    use crate::freertos::types::{TickType, StackType};

    unsafe extern "C" {
        /// Returns the CPU clock frequency in Hz.
        ///
        /// Typically corresponds to `configCPU_CLOCK_HZ` in FreeRTOSConfig.h.
        pub fn osal_rs_config_cpu_clock_hz() -> u64;
        
        /// Returns the RTOS tick rate in Hz (ticks per second).
        ///
        /// Corresponds to `configTICK_RATE_HZ` in FreeRTOSConfig.h.
        pub fn osal_rs_config_tick_rate_hz() -> TickType;
        
        /// Returns the maximum number of priority levels.
        ///
        /// Corresponds to `configMAX_PRIORITIES` in FreeRTOSConfig.h.
        pub fn osal_rs_config_max_priorities() -> u32;
        
        /// Returns the minimum stack size for tasks.
        ///
        /// Corresponds to `configMINIMAL_STACK_SIZE` in FreeRTOSConfig.h.
        pub fn osal_rs_config_minimal_stack_size() -> StackType;
        
        /// Returns the maximum length for task names.
        ///
        /// Corresponds to `configMAX_TASK_NAME_LEN` in FreeRTOSConfig.h.
        pub fn osal_rs_config_max_task_name_len() -> u32;
    }
}

/// Returns the tick period in milliseconds.
///
/// This macro calculates the duration of one RTOS tick in milliseconds
/// based on the configured tick rate.
///
/// # Returns
///
/// Tick rate in Hz (ticks per second)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::tick_period_ms;
/// 
/// let period = tick_period_ms!();
/// println!("Each tick is {} Hz", period);
/// ```
///
/// # Note
///
/// Currently returns tick rate, not period. May be updated in future.
#[macro_export]
macro_rules! tick_period_ms {
    () => {
        // CHECK (1000 / $crate::freertos::config::TICK_RATE_HZ)
        (unsafe { $crate::os::config::ffi::osal_rs_config_tick_rate_hz() })
    };
}

/// Returns the RTOS tick rate in Hz.
///
/// This is the frequency at which the RTOS tick interrupt occurs,
/// determining the resolution of time-based operations.
///
/// # Returns
///
/// Tick rate in Hz (ticks per second)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::tick_rate_hz;
/// 
/// let rate = tick_rate_hz!();
/// println!("Tick rate: {} Hz", rate);
/// println!("Tick period: {} ms", 1000 / rate);
/// ```
#[macro_export]
macro_rules! tick_rate_hz {
    () => {
        (unsafe { $crate::os::config::ffi::osal_rs_config_tick_rate_hz() })
    };
}

/// Returns the CPU clock frequency in Hz.
///
/// This is the main system clock frequency used by the processor.
///
/// # Returns
///
/// CPU frequency in Hz
///
/// # Examples
///
/// ```ignore
/// use osal_rs::cpu_clock_hz;
/// 
/// let freq = cpu_clock_hz!();
/// println!("CPU running at {} MHz", freq / 1_000_000);
/// ```
#[macro_export]
macro_rules! cpu_clock_hz {
    () => {
        (unsafe { $crate::os::config::ffi::osal_rs_config_cpu_clock_hz() })
    };
}

/// Returns the maximum number of priority levels.
///
/// Tasks can have priorities from 0 (lowest) to max_priorities-1 (highest).
///
/// # Returns
///
/// Maximum number of priority levels
///
/// # Examples
///
/// ```ignore
/// use osal_rs::max_priorities;
/// 
/// let max = max_priorities!();
/// println!("Priority range: 0 to {}", max - 1);
/// ```
#[macro_export]
macro_rules! max_priorities {
    () => {
        (unsafe { $crate::os::config::ffi::osal_rs_config_max_priorities() })
    };
}

/// Returns the minimum stack size for tasks.
///
/// This is the smallest stack size that should be used when creating tasks,
/// typically measured in words (not bytes).
///
/// # Returns
///
/// Minimum stack size in words
///
/// # Examples
///
/// ```ignore
/// use osal_rs::minimal_stack_size;
/// 
/// let min_stack = minimal_stack_size!();
/// println!("Minimum stack: {} words", min_stack);
/// ```
#[macro_export]
macro_rules! minimal_stack_size {
    () => {
        ( unsafe { $crate::os::config::ffi::osal_rs_config_minimal_stack_size() })
    };
}   

/// Returns the maximum length for task names.
///
/// Task names longer than this will be truncated.
///
/// # Returns
///
/// Maximum number of characters in task names (including null terminator)
///
/// # Examples
///
/// ```ignore
/// use osal_rs::max_task_name_len;
/// 
/// let max_len = max_task_name_len!();
/// println!("Max task name length: {} characters", max_len);
/// ```
#[macro_export]
macro_rules! max_task_name_len {
    () => {
        (unsafe { $crate::os::config::ffi::osal_rs_config_max_task_name_len() })
    };
}