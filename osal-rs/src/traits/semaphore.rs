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

use crate::os::types::UBaseType;
use crate::utils::{OsalRsBool, Result};
use super::ToTick;



pub trait Semaphore {
 
    fn new(max_count: UBaseType, initial_count: UBaseType) -> Result<Self> 
    where 
        Self: Sized;

    fn new_with_count(initial_count: UBaseType) -> Result<Self> 
    where 
        Self: Sized;

    fn wait(&self, ticks_to_wait: impl ToTick) -> OsalRsBool;

    fn wait_from_isr(&self) -> OsalRsBool;

    fn signal(&self) -> OsalRsBool;
    
    fn signal_from_isr(&self) -> OsalRsBool;
    
    fn delete(&mut self);

}
