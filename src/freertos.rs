// Central FFI bindings module - included only once
pub(crate) mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clashing_extern_declarations)]
    include!(concat!(env!("OUT_DIR"), "/freertos_bindings.rs"));
}

pub mod event;

pub mod free_rtos_allocator;

pub mod mutex;

pub mod queue;

pub mod semaphore;

pub mod stream_buffer;

pub mod system;

pub mod thread;

pub mod time;

pub mod timer;

