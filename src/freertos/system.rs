use super::constants::CONFIG_TICK_RATE_HZ;
use crate::osal::ffi::TickType_t;
use crate::osal::system::ffi::{vTaskDelay, vTaskEndScheduler, vTaskStartScheduler, xTaskGetTickCount};

#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    use crate::freertos::ffi::TickType_t;
    unsafe extern "C" {
        pub fn vTaskDelayUntil(pxPreviousWakeTime: *mut TickType_t, xTimeIncrement: TickType_t);
        pub fn xTaskGetTickCount() -> TickType_t;
        pub fn vTaskStartScheduler();
        pub fn vTaskEndScheduler();
        pub fn vTaskDelay( xTicksToDelay :  TickType_t );
    }
}

#[macro_export]
macro_rules! ms_to_us {
    (ms:expr) => {
        { (ms as u64) * 1_000 }
    };
}

#[macro_export]
macro_rules! sec_to_us {
    (ms:expr) => {
        { (ms as u64) * 1_000_000 }
    };
}

#[cfg(feature = "freertos")]
pub fn os_version() -> &'static str {
    "FreeRTOS V11.2.0"
}

pub fn us_sleep(us: u64) {
    unsafe {
        vTaskDelay( ( us / (CONFIG_TICK_RATE_HZ as u64) / 1_000 )as TickType_t);
    }
}

pub fn ticks_sleep(ticks_to_delay: u32) {
    unsafe {
        vTaskDelay(ticks_to_delay);
    }
}

pub fn tick_current () -> TickType_t {
    unsafe {
        xTaskGetTickCount()
    }
}

pub fn us_to_ticks(us: u64) -> TickType_t {
    super::constants::ms_to_ticks((us / 1_000) as u32) as TickType_t
}

pub fn ticks_to_us(ticks: TickType_t) -> u64 {
    (super::constants::ticks_to_ms(ticks) * 1_000) as u64
}

pub fn start_scheduler() {
    unsafe {
        vTaskStartScheduler();
    }
}

pub fn end_scheduler() {
    unsafe {
        vTaskEndScheduler();
    }
}


