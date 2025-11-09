
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
