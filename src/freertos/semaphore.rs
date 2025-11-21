#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/freertos_bindings.rs"));
}
pub struct Semaphore {

}

impl Semaphore {
    pub fn new() -> Self {
        Semaphore {}
    }
}

impl Default for Semaphore {
    fn default() -> Self {
        Self::new()
    }
}
