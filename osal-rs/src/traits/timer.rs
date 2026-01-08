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

//! Software timer trait for delayed and periodic callbacks.
//!
//! Timers execute callback functions in the context of a timer service task,
//! enabling delayed operations and periodic tasks without dedicated threads.

use core::any::Any;

use alloc::{boxed::Box, sync::Arc};

use crate::os::types::TickType;
use crate::utils::{OsalRsBool, Result};

/// Type-erased parameter for timer callbacks.
///
/// Allows passing arbitrary data to timer callback functions.
pub type TimerParam = Arc<dyn Any + Send + Sync>;

/// Timer callback function pointer type.
///
/// Callbacks receive the timer handle and optional parameter,
/// and can return an updated parameter value.
pub type TimerFnPtr = dyn Fn(Box<dyn Timer>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + 'static;

/// Software timer for delayed and periodic callbacks.
///
/// Timers run callbacks in the timer service task context, not ISR context.
/// They can be one-shot or auto-reloading (periodic).
///
/// # Examples
///
/// ## One-shot Timer
///
/// ```ignore
/// use osal_rs::os::{Timer, TimerFn};
/// use core::time::Duration;
/// 
/// let timer = Timer::new(
///     "one_shot",
///     Duration::from_secs(5),
///     false,  // One-shot
///     None,
///     |_timer, _param| {
///         println!("Timer expired!");
///         Ok(None)
///     }
/// ).unwrap();
/// 
/// timer.start(0);
/// ```
///
/// ## Periodic Timer
///
/// ```ignore
/// let periodic = Timer::new(
///     "periodic",
///     Duration::from_millis(100),
///     true,  // Auto-reload
///     None,
///     |_timer, _param| {
///         println!("Tick!");
///         Ok(None)
///     }
/// ).unwrap();
/// 
/// periodic.start(0);
/// // Runs every 100ms until stopped
/// ```
pub trait Timer {
    /// Starts or restarts the timer.
    ///
    /// If the timer is already running, it is reset to its full period.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Max ticks to wait if command queue is full
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully
    /// * `False` - Failed to send command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// timer.start(100);  // Start with 100 tick timeout
    /// ```
    fn start(&self, ticks_to_wait: TickType) -> OsalRsBool;
    
    /// Stops the timer.
    ///
    /// The timer will not expire until started again.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Max ticks to wait if command queue is full
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully
    /// * `False` - Failed to send command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// timer.stop(100);
    /// ```
    fn stop(&self, ticks_to_wait: TickType)  -> OsalRsBool;
    
    /// Resets the timer to its full period.
    ///
    /// If the timer is running, it restarts from the beginning.
    /// If stopped, it starts the timer.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Max ticks to wait if command queue is full
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully
    /// * `False` - Failed to send command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// timer.reset(100);  // Restart timer
    /// ```
    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool;
    
    /// Changes the timer period.
    ///
    /// The new period takes effect immediately if the timer is running.
    ///
    /// # Parameters
    ///
    /// * `new_period_in_ticks` - New timer period in ticks
    /// * `new_period_ticks` - Max ticks to wait if command queue is full
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully
    /// * `False` - Failed to send command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Change period to 500 ticks
    /// timer.change_period(500, 100);
    /// ```
    fn change_period(&self, new_period_in_ticks: TickType, new_period_ticks: TickType) -> OsalRsBool;
    
    /// Deletes the timer and frees its resources.
    ///
    /// The timer must be stopped before deletion.
    ///
    /// # Parameters
    ///
    /// * `ticks_to_wait` - Max ticks to wait if command queue is full
    ///
    /// # Returns
    ///
    /// * `True` - Command sent successfully
    /// * `False` - Failed to send command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut timer = Timer::new(...).unwrap();
    /// timer.stop(100);
    /// timer.delete(100);
    /// ```
    fn delete(&mut self, ticks_to_wait: TickType) -> OsalRsBool;
}