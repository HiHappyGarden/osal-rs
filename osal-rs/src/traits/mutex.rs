use crate::utils::{OsalRsBool, Result};

pub trait RawMutex
where
    Self: Sized,
{
    fn new() -> Result<Self>
    where 
        Self: Sized;

    fn lock(&self) -> OsalRsBool;

    fn lock_from_isr(&self) -> OsalRsBool;

    fn unlock(&self) -> OsalRsBool;

    fn unlock_from_isr(&self) -> OsalRsBool;

    fn delete(&mut self);
}

pub trait MutexGuard<'a, T: ?Sized + 'a> {}

pub trait Mutex<T: ?Sized> {
    type Guard<'a>: MutexGuard<'a, T> where Self: 'a, T: 'a;
    type GuardFromIsr<'a>: MutexGuard<'a, T> where Self: 'a, T: 'a;

    /// Creates a new mutex wrapping the supplied data
    fn new(data: T) -> Result<Self> 
    where 
        Self: Sized,
        T: Sized;

    /// Acquires the mutex, blocking the current thread until it is able to do so
    fn lock(&self) -> Result<Self::Guard<'_>>;
    
    /// Acquires the mutex from ISR context
    fn lock_from_isr(&self) -> Result<Self::GuardFromIsr<'_>>;

    /// Attempts to consume this mutex, returning the underlying data
    fn into_inner(self) -> Result<T> 
    where 
        Self: Sized, 
        T: Sized;

    /// Returns a mutable reference to the underlying data
    fn get_mut(&mut self) -> &mut T;
}
