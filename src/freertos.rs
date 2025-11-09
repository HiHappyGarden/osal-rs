// FreeRTOS FFI bindings
#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi;

// OSAL modules
pub mod queue;
pub mod thread;
pub mod semaphore;
pub mod stream_buffer;
pub mod mutex;
pub mod event;
pub mod timer;
pub mod time;
pub mod memory;
mod constants;