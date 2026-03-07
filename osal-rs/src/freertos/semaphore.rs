/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Counting semaphore synchronization primitives for FreeRTOS.
//!
//! This module provides counting semaphores for resource management and signaling
//! between threads and ISRs. Semaphores maintain a count and can be used to
//! coordinate access to shared resources or signal event completion.

use core::fmt::{Debug, Display};
use core::ops::Deref;
use core::ptr::null_mut;

use super::ffi::{SemaphoreHandle, pdFAIL, pdFALSE};
use super::system::System;
use super::types::{BaseType, UBaseType};
use crate::traits::{SemaphoreFn, SystemFn, ToTick};
use crate::utils::{Error, Result, OsalRsBool};
use crate::{vSemaphoreDelete, xSemaphoreCreateCounting, xSemaphoreGive, xSemaphoreGiveFromISR, xSemaphoreTake, xSemaphoreTakeFromISR};

/// A counting semaphore for resource management and signaling.
///
/// Semaphores maintain a count that can be incremented (signaled) and decremented (waited).
/// They are useful for:
/// - Resource counting (e.g., managing a pool of N resources)
/// - Event signaling between threads or from ISRs
/// - Producer-consumer synchronization
///
/// # Examples
///
/// ## Basic binary semaphore (mutex alternative)
///
/// ```ignore
/// use osal_rs::os::{Semaphore, SemaphoreFn};
/// use core::time::Duration;
/// 
/// // Create a binary semaphore (max_count = 1)
/// let sem = Semaphore::new(1, 1).unwrap();
/// 
/// // Wait (take) the semaphore
/// if sem.wait(Duration::from_millis(100)).into() {
///     // Critical section
///     println!("Acquired semaphore");
///     
///     // Signal (give) the semaphore
///     sem.signal();
/// }
/// ```
///
/// ## Resource pool management
///
/// ```ignore
/// use osal_rs::os::{Semaphore, SemaphoreFn, Thread};
/// use alloc::sync::Arc;
/// use core::time::Duration;
/// 
/// // Create semaphore for 5 resources
/// let resources = Arc::new(Semaphore::new(5, 5).unwrap());
/// 
/// let sem_clone = resources.clone();
/// let worker = Thread::new("worker", 2048, 5, move || {
///     loop {
///         // Wait for an available resource
///         if sem_clone.wait(Duration::from_secs(1)).into() {
///             println!("Resource acquired");
///             
///             // Use resource...
///             Duration::from_millis(500).sleep();
///             
///             // Release resource
///             sem_clone.signal();
///         }
///     }
/// }).unwrap();
/// ```
///
/// ## Event signaling from ISR
///
/// ```ignore
/// use osal_rs::os::{Semaphore, SemaphoreFn};
/// use alloc::sync::Arc;
/// 
/// let event_sem = Arc::new(Semaphore::new(1, 0).unwrap());  // Initially unavailable
/// let sem_clone = event_sem.clone();
/// 
/// // In interrupt handler:
/// // sem_clone.signal_from_isr();  // Signal event occurred
/// 
/// // In thread:
/// if event_sem.wait(1000).into() {
///     println!("Event received!");
/// }
/// ```
///
/// ## Counting events
///
/// ```ignore
/// use osal_rs::os::{Semaphore, SemaphoreFn};
/// use core::time::Duration;
/// 
/// // Create semaphore with max_count=10, initially empty
/// let counter = Semaphore::new(10, 0).unwrap();
/// 
/// // Signal 3 times
/// counter.signal();
/// counter.signal();
/// counter.signal();
/// 
/// // Process 3 events
/// for _ in 0..3 {
///     if counter.wait(Duration::from_millis(10)).into() {
///         println!("Processing event");
///     }
/// }
/// ```
pub struct Semaphore (SemaphoreHandle);

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
        /// Creates a new counting semaphore.
    ///
    /// # Parameters
    ///
    /// * `max_count` - Maximum count value the semaphore can reach
    /// * `initial_count` - Initial count value
    ///
    /// # Returns
    ///
    /// * `Ok(Semaphore)` - Semaphore created successfully
    /// * `Err(Error::OutOfMemory)` - Failed to allocate semaphore
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Semaphore, SemaphoreFn};
    /// 
    /// // Binary semaphore
    /// let binary_sem = Semaphore::new(1, 1).unwrap();
    /// 
    /// // Counting semaphore for 5 resources
    /// let counting_sem = Semaphore::new(5, 5).unwrap();
    /// ```
    pub fn new(max_count: UBaseType, initial_count: UBaseType) -> Result<Self> {
        let handle = xSemaphoreCreateCounting!(max_count, initial_count);
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

    /// Creates a counting semaphore with maximum possible count.
    ///
    /// Sets `max_count` to `UBaseType::MAX`.
    ///
    /// # Parameters
    ///
    /// * `initial_count` - Initial count value
    ///
    /// # Returns
    ///
    /// * `Ok(Semaphore)` - Semaphore created successfully
    /// * `Err(Error::OutOfMemory)` - Failed to allocate
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Semaphore, SemaphoreFn};
    /// 
    /// let sem = Semaphore::new_with_count(0).unwrap();
    /// ```
    pub fn new_with_count(initial_count: UBaseType) -> Result<Self> {
        let handle = xSemaphoreCreateCounting!(UBaseType::MAX, initial_count);
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

}

impl SemaphoreFn for Semaphore {

    /// Waits to acquire the semaphore (decrements count).
    ///
    /// Blocks until semaphore is available or timeout expires.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Maximum time to wait (supports `Duration` via `ToTick`)
    ///
    /// # Returns
    ///
    /// * `OsalRsBool::True` - Semaphore acquired
    /// * `OsalRsBool::False` - Timeout or error
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Semaphore, SemaphoreFn};
    /// use core::time::Duration;
    /// 
    /// let sem = Semaphore::new(1, 1).unwrap();
    /// if sem.wait(Duration::from_millis(100)).into() {
    ///     // Critical section
    ///     sem.signal();
    /// }
    /// ```
    fn wait(&self, ticks_to_wait: impl ToTick) -> OsalRsBool {
        if xSemaphoreTake!(self.0, ticks_to_wait.to_ticks()) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    /// Waits to acquire the semaphore from ISR context (non-blocking).
    ///
    /// # Returns
    ///
    /// * `OsalRsBool::True` - Semaphore acquired
    /// * `OsalRsBool::False` - Semaphore not available
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In ISR:
    /// if sem.wait_from_isr().into() {
    ///     // Handle event
    /// }
    /// ```
    fn wait_from_isr(&self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreTakeFromISR!(self.0, &mut higher_priority_task_woken) != pdFAIL {

            System::yield_from_isr(higher_priority_task_woken);

            OsalRsBool::True
        } else {

            OsalRsBool::False
        }
    }
    
    /// Signals (releases) the semaphore (increments count).
    ///
    /// # Returns
    ///
    /// * `OsalRsBool::True` - Semaphore signaled successfully
    /// * `OsalRsBool::False` - Error (e.g., count already at maximum)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Semaphore, SemaphoreFn};
    /// 
    /// let sem = Semaphore::new(1, 0).unwrap();
    /// sem.signal();  // Make semaphore available
    /// ```
    fn signal(&self) -> OsalRsBool {
        if xSemaphoreGive!(self.0) != pdFAIL {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }
    
    /// Signals the semaphore from ISR context.
    ///
    /// Automatically yields to higher priority tasks if needed.
    ///
    /// # Returns
    ///
    /// * `OsalRsBool::True` - Semaphore signaled successfully
    /// * `OsalRsBool::False` - Error
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // In ISR:
    /// sem.signal_from_isr();
    /// ```
    fn signal_from_isr(&self) -> OsalRsBool {
        let mut higher_priority_task_woken: BaseType = pdFALSE;
        if xSemaphoreGiveFromISR!(self.0, &mut higher_priority_task_woken) != pdFAIL {
            
            System::yield_from_isr(higher_priority_task_woken);

            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }
    
    /// Deletes the semaphore and frees its resources.
    ///
    /// After calling this, the semaphore handle is set to null and should not be used.
    ///
    /// # Safety
    ///
    /// Ensure no threads are waiting on this semaphore before deleting it.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{Semaphore, SemaphoreFn};
    /// 
    /// let mut sem = Semaphore::new(1, 1).unwrap();
    /// // Use the semaphore...
    /// sem.delete();
    /// ```
    fn delete(&mut self) {
        vSemaphoreDelete!(self.0);
        self.0 = null_mut();
    }


}


/// Automatically deletes the semaphore when it goes out of scope.
/// 
/// This ensures proper cleanup of FreeRTOS resources.
impl Drop for Semaphore {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
    }
}

/// Allows dereferencing to the underlying FreeRTOS semaphore handle.
impl Deref for Semaphore {
    type Target = SemaphoreHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Formats the semaphore for debugging purposes.
impl Debug for Semaphore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Semaphore")
            .field("handle", &self.0)
            .finish()
    }
}

/// Formats the semaphore for display purposes.
impl Display for Semaphore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Semaphore {{ handle: {:?} }}", self.0)
    }
}