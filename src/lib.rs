//! OSAL-RS: Operating System Abstraction Layer for Rust
//!
//! This library provides a unified interface for operating system primitives
//! that works across POSIX systems (Linux, macOS, Unix) and FreeRTOS.
//!
//! # Features
//!
//! - Thread/Task management
//! - Mutex and Semaphore primitives
//! - Message queues
//! - Timers
//! - Time/Clock functions
//!
//! # Platform Support
//!
//! - **POSIX**: Uses standard Unix/POSIX APIs (enabled by default)
//! - **FreeRTOS**: Uses FreeRTOS APIs (experimental)
//!
//! # Examples
//!
//! ```rust
//! use osal_rs::{Thread, Mutex};
//!
//! // Create and run a thread
//! let thread = Thread::new("worker", || {
//!     println!("Hello from thread!");
//! });
//!
//! // Use a mutex
//! let mutex = Mutex::new(42);
//! let guard = mutex.lock();
//! println!("Value: {}", *guard);
//! ```

// Ensure only one platform feature is enabled
#[cfg(all(feature = "posix", feature = "freertos"))]
compile_error!("Cannot enable both 'posix' and 'freertos' features simultaneously");

#[cfg(not(any(feature = "posix", feature = "freertos")))]
compile_error!("Must enable either 'posix' or 'freertos' feature");

pub mod thread;
pub mod mutex;
pub mod semaphore;
pub mod queue;
pub mod timer;
pub mod time;

// Re-export main types
pub use thread::Thread;
pub use mutex::Mutex;
pub use semaphore::{Semaphore, BinarySemaphore};
pub use queue::Queue;
pub use timer::Timer;
pub use time::{Duration, Instant};

/// Result type used throughout the OSAL library
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur in OSAL operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Resource is currently unavailable
    WouldBlock,
    /// Operation timed out
    Timeout,
    /// Invalid parameter provided
    InvalidParameter,
    /// Resource limit reached
    ResourceExhausted,
    /// Operation not supported on this platform
    NotSupported,
    /// Other platform-specific error
    Other(&'static str),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::WouldBlock => write!(f, "Operation would block"),
            Error::Timeout => write!(f, "Operation timed out"),
            Error::InvalidParameter => write!(f, "Invalid parameter"),
            Error::ResourceExhausted => write!(f, "Resource exhausted"),
            Error::NotSupported => write!(f, "Operation not supported"),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
