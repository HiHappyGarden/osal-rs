#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/posix_bindings.rs"));

    use core::ffi::{c_char, c_int, c_void};

    unsafe extern "C" {
        pub(crate) fn pthread_setname_np(thread: pthread_t, name: *const c_char) -> c_int;

        pub(crate) fn pthread_getname_np(thread: pthread_t, name: *mut c_char, len: usize) -> c_int;
    }

    impl Default for pthread_mutex_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_cond_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_condattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_mutexattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for timespec {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

}

pub mod event;

pub mod mutex;

mod posix_allocator;

pub mod queue;

pub mod semaphore;

pub mod stream_buffer;

pub mod system;

pub mod thread;

pub mod time;

pub mod timer;


