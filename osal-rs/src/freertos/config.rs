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

pub mod ffi {
    use crate::freertos::types::{TickType, StackType};

    unsafe extern "C" {
        pub fn osal_rs_config_cpu_clock_hz() -> u64;
        pub fn osal_rs_config_tick_rate_hz() -> TickType;
        pub fn osal_rs_config_max_priorities() -> u32;
        pub fn osal_rs_config_minimal_stack_size() -> StackType;
        pub fn osal_rs_config_max_task_name_len() -> u32;
    }
}

#[macro_export]
macro_rules! tick_period_ms {
    () => {
        // CHECK (1000 / $crate::freertos::config::TICK_RATE_HZ)
        (unsafe { $crate::freertos::config::ffi::osal_rs_config_tick_rate_hz() })
    };
}

#[macro_export]
macro_rules! tick_rate_hz {
    () => {
        (unsafe { $crate::freertos::config::ffi::osal_rs_config_tick_rate_hz() })
    };
}


#[macro_export]
macro_rules! cpu_clock_hz {
    () => {
        (unsafe { $crate::freertos::config::ffi::osal_rs_config_cpu_clock_hz() })
    };
}

#[macro_export]
macro_rules! max_priorities {
    () => {
        (unsafe { $crate::freertos::config::ffi::osal_rs_config_max_priorities() })
    };
}

#[macro_export]
macro_rules! minimal_stack_size {
    () => {
        ( unsafe { $crate::freertos::config::ffi::osal_rs_config_minimal_stack_size() })
    };
}   

#[macro_export]
macro_rules! max_task_name_len {
    () => {
        (unsafe { $crate::freertos::config::ffi::osal_rs_config_max_task_name_len() })
    };
}
