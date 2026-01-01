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

use crate::utils::Result;
use crate::os::types::{EventBits, TickType};

pub trait EventGroup {
    fn new() -> Result<Self> 
    where 
        Self: Sized;

    fn set(&self, bits: EventBits) -> EventBits;

    fn set_from_isr(&self, bits: EventBits) -> Result<()>;

    fn get(&self) -> EventBits;

    fn get_from_isr(&self) -> EventBits;

    fn clear(&self, bits: EventBits) -> EventBits;
    
    fn clear_from_isr(&self, bits: EventBits) -> Result<()>;

    fn wait(&self, mask: EventBits, timeout_ticks: TickType) -> EventBits;

    fn delete(&mut self);
}