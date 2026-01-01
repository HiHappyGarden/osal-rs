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

use crate::os::ToBytes;
use crate::os::types::{UBaseType, TickType};
use crate::utils::Result;



pub trait Queue {
    fn new (size: UBaseType, message_size: UBaseType) -> Result<Self>
    where 
        Self: Sized;

    fn fetch(&self, buffer: &mut [u8], time: TickType) -> Result<()>;

    fn fetch_from_isr(&self, buffer: &mut [u8]) -> Result<()>;
    
    fn post(&self, item: &[u8], time: TickType) -> Result<()>;
    fn post_from_isr(&self, item: &[u8]) -> Result<()>;

    fn delete(&mut self);
}

pub trait QueueStreamed<T> 
where 
    T: ToBytes + Sized {

    fn new (size: UBaseType, message_size: UBaseType) -> Result<Self>
    where 
        Self: Sized;

    fn fetch(&self, buffer: &mut T, time: TickType) -> Result<()>;

    fn fetch_from_isr(&self, buffer: &mut T) -> Result<()>;
    
    fn post(&self, item: &T, time: TickType) -> Result<()>;

    fn post_from_isr(&self, item: &T) -> Result<()>;

    fn delete(&mut self);
}