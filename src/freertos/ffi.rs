// freertos/ffi.rs - FFI bindings for FreeRTOS
// This module provides direct access to FreeRTOS C functions

#[cfg(feature = "bindgen")]
include!(concat!(env!("OUT_DIR"), "/freertos_bindings.rs"));

// Manual FFI declarations when not using bindgen
#[cfg(not(feature = "bindgen"))]
pub mod manual {
    use core::ffi::{c_char, c_void, c_int, c_uint};

    // FreeRTOS types
    pub type BaseType_t = c_int;
    pub type UBaseType_t = c_uint;
    pub type TickType_t = u32;
    pub type QueueHandle_t = *mut c_void;
    pub type SemaphoreHandle_t = *mut c_void;
    pub type TimerHandle_t = *mut c_void;
    pub type EventGroupHandle_t = *mut c_void;
    pub type StreamBufferHandle_t = *mut c_void;


    // FreeRTOS constants
    pub const pdTRUE: BaseType_t = 1;
    pub const pdFALSE: BaseType_t = 0;
    pub const pdPASS: BaseType_t = 1;
    pub const pdFAIL: BaseType_t = 0;

    unsafe extern "C" {
        // Task Management

        // Queue Management

        // Semaphore Management
        pub fn xSemaphoreCreateBinary() -> SemaphoreHandle_t;
        pub fn xSemaphoreCreateCounting(uxMaxCount: UBaseType_t, uxInitialCount: UBaseType_t) -> SemaphoreHandle_t;
        pub fn xSemaphoreCreateMutex() -> SemaphoreHandle_t;
        pub fn vSemaphoreDelete(xSemaphore: SemaphoreHandle_t);

        // Timer Management
        pub fn xTimerCreate(
            pcTimerName: *const c_char,
            xTimerPeriodInTicks: TickType_t,
            uxAutoReload: UBaseType_t,
            pvTimerID: *mut c_void,
            pxCallbackFunction: unsafe extern "C" fn(TimerHandle_t),
        ) -> TimerHandle_t;
        pub fn xTimerDelete(xTimer: TimerHandle_t, xBlockTime: TickType_t) -> BaseType_t;
        pub fn xTimerStart(xTimer: TimerHandle_t, xBlockTime: TickType_t) -> BaseType_t;
        pub fn xTimerStop(xTimer: TimerHandle_t, xBlockTime: TickType_t) -> BaseType_t;

        // Event Group Management
        pub fn xEventGroupCreate() -> EventGroupHandle_t;
        pub fn vEventGroupDelete(xEventGroup: EventGroupHandle_t);

        // Stream Buffer Management
        pub fn xStreamBufferCreate(xBufferSizeBytes: usize, xTriggerLevelBytes: usize) -> StreamBufferHandle_t;
        pub fn vStreamBufferDelete(xStreamBuffer: StreamBufferHandle_t);

        // Memory Management
        pub fn pvPortMalloc(xWantedSize: usize) -> *mut c_void;
        pub fn vPortFree(pv: *mut c_void);
    }
}

// Re-export manual bindings when not using bindgen
#[cfg(not(feature = "bindgen"))]
pub use manual::*;

