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

use core::fmt::{Debug, Display};


/// Trait for types that can provide a string reference in a thread-safe manner.
///
/// This trait extends the basic string reference functionality with thread-safety
/// guarantees by requiring both `Sync` and `Send` bounds. It's useful for types
/// that need to provide string data across thread boundaries in a concurrent
/// environment.
///
/// # Thread Safety
///
/// Implementors must be both `Sync` (safe to share references across threads) and
/// `Send` (safe to transfer ownership across threads).
///
/// # Examples
///
/// ```ignore
/// use osal_rs::utils::AsSyncStr;
/// 
/// struct ThreadSafeName {
///     name: &'static str,
/// }
/// 
/// impl AsSyncStr for ThreadSafeName {
///     fn as_str(&self) -> &str {
///         self.name
///     }
/// }
/// 
/// // Can be safely shared across threads
/// fn use_in_thread(item: &dyn AsSyncStr) {
///     println!("Name: {}", item.as_str());
/// }
/// ```
pub trait AsSyncStr : Sync + Send { 
    /// Returns a string slice reference.
    ///
    /// This method provides access to the underlying string data in a way
    /// that is safe to use across thread boundaries.
    ///
    /// # Returns
    ///
    /// A reference to a string slice with lifetime tied to `self`.
    fn as_str(&self) -> &str;
}

impl PartialEq for dyn AsSyncStr {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for dyn AsSyncStr {}

impl Debug for dyn AsSyncStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for dyn AsSyncStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

