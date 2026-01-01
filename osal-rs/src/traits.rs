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

mod byte;
mod event_group;
mod mutex;
mod queue;
mod semaphore;
mod system;
mod thread;
mod tick;
mod timer;

pub use crate::traits::byte::*;
pub use crate::traits::event_group::EventGroup as EventGroupFn;
pub use crate::traits::mutex::{Mutex as MutexFn, MutexGuard as MutexGuardFn, RawMutex as RawMutexFn};
pub use crate::traits::queue::{Queue as QueueFn, QueueStreamed as QueueStreamedFn};
pub use crate::traits::semaphore::Semaphore as SemaphoreFn;
pub use crate::traits::system::System as SystemFn;
pub use crate::traits::thread::{Thread as ThreadFn, ThreadParam, ThreadFnPtr, ThreadSimpleFnPtr, ThreadNotification};
pub use crate::traits::tick::*;
pub use crate::traits::timer::{Timer as TimerFn, TimerParam, TimerFnPtr};
