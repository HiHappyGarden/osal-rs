/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
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

//! Event group synchronization primitives for FreeRTOS.
//!
//! Event groups allow threads to synchronize on multiple events simultaneously.
//! Each event group contains a set of event bits (flags) that can be set, cleared,
//! and waited upon. This is useful for complex synchronization scenarios where
//! multiple conditions must be met.

use core::fmt::{Debug, Display, Formatter};
use core::ops::Deref;
use core::ptr::null_mut;

use super::ffi::{EventGroupHandle, pdFAIL, pdFALSE, vEventGroupDelete, xEventGroupClearBits, xEventGroupClearBitsFromISR, xEventGroupCreate, xEventGroupGetBitsFromISR, xEventGroupSetBits, xEventGroupSetBitsFromISR};
use super::system::System;
use super::types::{BaseType, EventBits, TickType};
use crate::traits::{ToTick, EventGroupFn, SystemFn};
use crate::utils::{Result, Error};
use crate::xEventGroupGetBits;

/// A set of event flags for thread synchronization.
///
/// Event groups contain multiple event bits (typically 24 bits) that can be
/// manipulated independently. Threads can wait for specific combinations of bits
/// to be set, making them ideal for complex synchronization scenarios.
///
/// # Examples
///
/// ## Basic event signaling
///
/// ```ignore
/// use osal_rs::os::{EventGroup, EventGroupFn};
/// use core::time::Duration;
/// 
/// const EVENT_A: u32 = 0b0001;
/// const EVENT_B: u32 = 0b0010;
/// const EVENT_C: u32 = 0b0100;
/// 
/// let events = EventGroup::new().unwrap();
/// 
/// // Set event A
/// events.set(EVENT_A);
/// 
/// // Check if event A is set
/// let current = events.get();
/// if current & EVENT_A != 0 {
///     println!("Event A is set");
/// }
/// 
/// // Clear event A
/// events.clear(EVENT_A);
/// ```
///
/// ## Waiting for multiple events
///
/// ```ignore
/// use osal_rs::os::{EventGroup, EventGroupFn, Thread};
/// use alloc::sync::Arc;
/// use core::time::Duration;
/// 
/// const READY: u32 = 0b0001;
/// const DATA_AVAILABLE: u32 = 0b0010;
/// const STOP: u32 = 0b0100;
/// 
/// let events = Arc::new(EventGroup::new().unwrap());
/// let events_clone = events.clone();
/// 
/// // Worker thread waits for events
/// let worker = Thread::new("worker", 2048, 5, move || {
///     loop {
///         // Wait for either READY or STOP
///         let bits = events_clone.wait_with_to_tick(
///             READY | STOP,
///             Duration::from_secs(1)
///         );
///         
///         if bits & STOP != 0 {
///             println!("Stopping...");
///             break;
///         }
///         
///         if bits & READY != 0 {
///             println!("Ready to work!");
///         }
///     }
/// }).unwrap();
/// 
/// worker.start().unwrap();
/// 
/// // Signal events
/// events.set(READY);
/// Duration::from_secs(2).sleep();
/// events.set(STOP);
/// ```
///
/// ## State machine synchronization
///
/// ```ignore
/// use osal_rs::os::{EventGroup, EventGroupFn};
/// use core::time::Duration;
/// 
/// const INIT_COMPLETE: u32 = 1 << 0;
/// const CONFIG_LOADED: u32 = 1 << 1;
/// const NETWORK_UP: u32 = 1 << 2;
/// const READY_TO_RUN: u32 = INIT_COMPLETE | CONFIG_LOADED | NETWORK_UP;
/// 
/// let state = EventGroup::new().unwrap();
/// 
/// // Different subsystems set their bits
/// state.set(INIT_COMPLETE);
/// state.set(CONFIG_LOADED);
/// state.set(NETWORK_UP);
/// 
/// // Wait for all systems to be ready
/// let current = state.wait_with_to_tick(READY_TO_RUN, Duration::from_secs(5));
/// 
/// if (current & READY_TO_RUN) == READY_TO_RUN {
///     println!("All systems ready!");
/// }
/// ```
///
/// ## ISR to thread signaling
///
/// ```ignore
/// use osal_rs::os::{EventGroup, EventGroupFn, Thread};
/// use alloc::sync::Arc;
/// 
/// const IRQ_EVENT: u32 = 1 << 0;
/// 
/// let events = Arc::new(EventGroup::new().unwrap());
/// let events_isr = events.clone();
/// 
/// // In interrupt handler:
/// // events_isr.set_from_isr(IRQ_EVENT).ok();
/// 
/// // Handler thread
/// let handler = Thread::new("handler", 2048, 5, move || {
///     loop {
///         let bits = events.wait(IRQ_EVENT, 1000);
///         if bits & IRQ_EVENT != 0 {
///             println!("Handling interrupt event");
///             events.clear(IRQ_EVENT);
///         }
///     }
/// }).unwrap();
/// ```
pub struct EventGroup (EventGroupHandle);

unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}

impl EventGroup {
    pub fn wait_with_to_tick(&self, mask: EventBits, timeout_ticks: impl ToTick) -> EventBits {
        self.wait(mask, timeout_ticks.to_ticks())
    }
}


impl EventGroup {
    /// Creates a new event group.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created event group
    /// * `Err(Error)` - Creation failed (out of memory, etc.)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osal_rs::os::{EventGroup, EventGroupFn};
    /// 
    /// let events = EventGroup::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        let handle = unsafe { xEventGroupCreate() };
        if handle.is_null() {
            Err(Error::OutOfMemory)
        } else {
            Ok(Self (handle))
        }
    }

}
impl EventGroupFn for EventGroup {

    fn set(&self, bits: EventBits) -> EventBits {
        unsafe { xEventGroupSetBits(self.0, bits) }
    }

    fn set_from_isr(&self, bits: EventBits) -> Result<()> {

        let mut higher_priority_task_woken: BaseType = pdFALSE;

        let ret = unsafe { xEventGroupSetBitsFromISR(self.0, bits, &mut higher_priority_task_woken) };
        if ret != pdFAIL {

            System::yield_from_isr(higher_priority_task_woken);
            
            Ok(())
        } else {
            Err(Error::QueueFull)
        }
    }

    fn get(&self) -> EventBits {
        xEventGroupGetBits!(self.0) 
    }

    fn get_from_isr(&self) -> EventBits{
        unsafe { xEventGroupGetBitsFromISR(self.0) }
    }


    fn clear(&self, bits: EventBits) -> EventBits {
        unsafe { xEventGroupClearBits(self.0, bits) }
    }

    fn clear_from_isr(&self, bits: EventBits) -> Result<()> {
        let ret = unsafe { xEventGroupClearBitsFromISR(self.0, bits) };
        if ret != pdFAIL {
            Ok(())
        } else {
            Err(Error::QueueFull)
        }
    }

    fn wait(&self, mask: EventBits, timeout_ticks: TickType) -> EventBits {
        unsafe {
            crate::freertos::ffi::xEventGroupWaitBits(
                self.0,
                mask,
                pdFALSE, 
                pdFALSE, 
                timeout_ticks,
            )
        }
    }

    fn delete(&mut self) {
        unsafe {
            vEventGroupDelete(self.0);
            self.0 = null_mut();
        }
    }
}

impl Drop for EventGroup {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        self.delete();
    }
}

impl Deref for EventGroup {
    type Target = EventGroupHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for EventGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "EventGroup {{ handle: {:?} }}", self.0)
    }
}

impl Display for EventGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "EventGroup {{ handle: {:?} }}", self.0)
    }
}