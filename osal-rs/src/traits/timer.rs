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

use core::any::Any;

use alloc::{boxed::Box, sync::Arc};

use crate::os::types::TickType;
use crate::utils::{OsalRsBool, Result};


pub type TimerParam = Arc<dyn Any + Send + Sync>;
pub type TimerFnPtr = dyn Fn(Box<dyn Timer>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + 'static;

pub trait Timer {
    fn new<F>(name: &str, timer_period_in_ticks: TickType, auto_reload: bool, param: Option<TimerParam>, callback: F) -> Result<Self>
    where
        Self: Sized,
        F: Fn(Box<dyn Timer>, Option<TimerParam>) -> Result<TimerParam> + Send + Sync + Clone + 'static;

    fn start(&self, ticks_to_wait: TickType) -> OsalRsBool;
    fn stop(&self, ticks_to_wait: TickType)  -> OsalRsBool;
    fn reset(&self, ticks_to_wait: TickType) -> OsalRsBool;
    fn change_period(&self, new_period_in_ticks: TickType, new_period_ticks: TickType) -> OsalRsBool;
    fn delete(&mut self, ticks_to_wait: TickType) -> OsalRsBool;
}