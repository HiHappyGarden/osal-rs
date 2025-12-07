
pub enum Error {
    OutOfMemory,
    QueueSendTimeout,
    QueueReceiveTimeout,
    MutexTimeout,
    Timeout,
    QueueFull,
    StringConversionError,
    TaskNotFound,
    InvalidQueueSize,
    Unhandled(&'static str)
}

pub type Result<T, E = Error> = core::result::Result<T, E>;