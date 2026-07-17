/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

extern crate alloc;

use osal_rs::os::*;
use osal_rs::utils::{OsalRsBool, Result};
use osal_rs::{log_debug, log_info};

const TAG: &str = "MutexTests";

pub fn test_mutex_creation() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_creation");
    let _mutex = Mutex::new(0u32);
    log_info!(TAG, "test_mutex_creation PASSED");
    Ok(())
}

pub fn test_mutex_lock_unlock() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_lock_unlock");
    let mutex = Mutex::new(42u32);
    
    {
        let guard = mutex.lock();
        assert!(guard.is_ok());
        
        if let Ok(g) = guard {
            log_debug!(TAG, "Mutex locked, value: {}", *g);
            assert_eq!(*g, 42);
        }
    }
    
    {
        let guard = mutex.lock();
        assert!(guard.is_ok());
        log_debug!(TAG, "Mutex re-locked successfully");
    }
    log_info!(TAG, "test_mutex_lock_unlock PASSED");
    Ok(())
}

pub fn test_mutex_modify_data() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_modify_data");
    let mutex = Mutex::new(0u32);
    
    {
        let mut guard = mutex.lock()?;
        *guard = 100;
        log_debug!(TAG, "Modified value to: {}", *guard);
    }
    
    {
        let guard = mutex.lock()?;
        log_debug!(TAG, "Read value: {}", *guard);
        assert_eq!(*guard, 100);
    }
    log_info!(TAG, "test_mutex_modify_data PASSED");
    Ok(())
}

pub fn test_mutex_multiple_locks() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_multiple_locks");
    let mutex = Mutex::new(0u32);
    
    for i in 0..10 {
        let mut guard = mutex.lock()?;
        *guard += 1;
        assert_eq!(*guard, i + 1);
    }
    
    let guard = mutex.lock()?;
    log_debug!(TAG, "Final counter value: {}", *guard);
    assert_eq!(*guard, 10);
    log_info!(TAG, "test_mutex_multiple_locks PASSED");
    Ok(())
}

pub fn test_mutex_guard_drop() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_guard_drop");
    let mutex = Mutex::new(42u32);
    
    {
        let _guard = mutex.lock()?;
        log_debug!(TAG, "Guard acquired, will drop on scope exit");
    }
    
    let guard = mutex.lock();
    assert!(guard.is_ok());
    log_info!(TAG, "test_mutex_guard_drop PASSED");
    Ok(())
}

pub fn test_mutex_with_struct() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_with_struct");
    #[derive(Debug, PartialEq)]
    struct TestData {
        value: u32,
        flag: bool,
    }
    
    let mutex = Mutex::new(TestData { value: 0, flag: false });
    
    {
        let mut guard = mutex.lock()?;
        guard.value = 123;
        guard.flag = true;
        log_debug!(TAG, "Modified struct - value: {}, flag: {}", guard.value, guard.flag);
    }
    
    {
        let guard = mutex.lock()?;
        assert_eq!(guard.value, 123);
        assert_eq!(guard.flag, true);
    }
    log_info!(TAG, "test_mutex_with_struct PASSED");
    Ok(())
}

pub fn test_mutex_recursive() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_recursive");
    let mutex = Mutex::new(0u32);
    
    let _guard1 = mutex.lock()?;
    log_debug!(TAG, "Lock 1 acquired");
    let _guard2 = mutex.lock()?;
    log_debug!(TAG, "Lock 2 acquired");
    let _guard3 = mutex.lock()?;
    log_debug!(TAG, "Lock 3 acquired");
    log_info!(TAG, "test_mutex_recursive PASSED");
    Ok(())
}

pub fn test_mutex_drop() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_drop");
    let mutex = Mutex::new(42u32);
    drop(mutex);
    log_info!(TAG, "test_mutex_drop PASSED");
    Ok(())
}

pub fn test_raw_mutex_lifecycle() -> Result<()> {
    log_info!(TAG, "Starting test_raw_mutex_lifecycle");
    let mut raw_mutex = RawMutex::new()?;

    let lock_result = raw_mutex.lock();
    log_debug!(TAG, "RawMutex lock result: {:?}", lock_result);
    assert_eq!(lock_result, OsalRsBool::True);

    let unlock_result = raw_mutex.unlock();
    log_debug!(TAG, "RawMutex unlock result: {:?}", unlock_result);
    assert_eq!(unlock_result, OsalRsBool::True);

    let isr_lock_result = raw_mutex.lock_from_isr();
    log_debug!(TAG, "RawMutex lock_from_isr result: {:?}", isr_lock_result);
    assert_eq!(isr_lock_result, OsalRsBool::True);

    let isr_unlock_result = raw_mutex.unlock_from_isr();
    log_debug!(TAG, "RawMutex unlock_from_isr result: {:?}", isr_unlock_result);
    assert_eq!(isr_unlock_result, OsalRsBool::True);

    raw_mutex.delete();
    log_info!(TAG, "test_raw_mutex_lifecycle PASSED");
    Ok(())
}

pub fn test_mutex_lock_from_isr_explicit() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_lock_from_isr_explicit");
    let mutex = Mutex::new(7u32);

    let guard = mutex.lock_from_isr_explicit()?;
    log_debug!(TAG, "Locked from ISR, value: {}", *guard);
    assert_eq!(*guard, 7);
    drop(guard);

    log_info!(TAG, "test_mutex_lock_from_isr_explicit PASSED");
    Ok(())
}

pub fn test_mutex_get_mut() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_get_mut");
    let mut mutex = Mutex::new(1u32);
    *mutex.get_mut() = 99;
    let guard = mutex.lock()?;
    log_debug!(TAG, "Value after get_mut write: {}", *guard);
    assert_eq!(*guard, 99);
    log_info!(TAG, "test_mutex_get_mut PASSED");
    Ok(())
}

pub fn test_mutex_into_inner() -> Result<()> {
    log_info!(TAG, "Starting test_mutex_into_inner");
    let mutex = Mutex::new(55u32);
    let value = mutex.into_inner()?;
    log_debug!(TAG, "Recovered inner value: {}", value);
    assert_eq!(value, 55);
    log_info!(TAG, "test_mutex_into_inner PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Mutex Tests ==========");
    test_mutex_creation()?;
    test_mutex_lock_unlock()?;
    test_mutex_modify_data()?;
    test_mutex_multiple_locks()?;
    test_mutex_guard_drop()?;
    test_mutex_with_struct()?;
    test_mutex_recursive()?;
    test_raw_mutex_lifecycle()?;
    test_mutex_lock_from_isr_explicit()?;
    test_mutex_get_mut()?;
    test_mutex_into_inner()?;
    test_mutex_drop()?;
    log_info!(TAG, "========== All Mutex Tests PASSED ==========");
    Ok(())
}
