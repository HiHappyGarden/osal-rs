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

pub trait BytesHasLen {
    fn len(&self) -> usize;
}

pub trait ToBytes {
    fn to_bytes(&self) -> &[u8];
}

impl<T, const N: usize> BytesHasLen for [T; N] 
where 
    T: ToBytes + Sized {
    fn len(&self) -> usize {
        N
    }
}

pub trait FromBytes: Sized
where
    Self: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}




