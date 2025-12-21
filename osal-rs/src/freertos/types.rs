
include!(concat!(env!("OUT_DIR"), "/types_generated.rs"));    

pub use super::ffi::{ThreadHandle, QueueHandle, SemaphoreHandle, EventGroupHandle, TimerHandle, MutexHandle};

pub type EventBits = TickType;
