//! POSIX mutex implementation

use crate::{Error, Result};
use std::sync::{Mutex as StdMutex, MutexGuard as StdMutexGuard, TryLockError};

pub struct PosixMutex<T> {
    inner: StdMutex<T>,
}

impl<T> PosixMutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: StdMutex::new(value),
        }
    }

    pub fn lock(&self) -> PosixMutexGuard<'_, T> {
        PosixMutexGuard {
            guard: self.inner.lock().expect("Mutex poisoned"),
        }
    }

    pub fn try_lock(&self) -> Result<PosixMutexGuard<'_, T>> {
        match self.inner.try_lock() {
            Ok(guard) => Ok(PosixMutexGuard { guard }),
            Err(TryLockError::WouldBlock) => Err(Error::WouldBlock),
            Err(TryLockError::Poisoned(_)) => Err(Error::Other("Mutex poisoned")),
        }
    }
}

pub struct PosixMutexGuard<'a, T> {
    guard: StdMutexGuard<'a, T>,
}

impl<'a, T> std::ops::Deref for PosixMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> std::ops::DerefMut for PosixMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}
