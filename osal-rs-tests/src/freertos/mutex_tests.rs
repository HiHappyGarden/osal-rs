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

extern crate alloc;

use osal_rs::os::*;
use osal_rs::utils::Result;
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

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Mutex Tests ==========");
    test_mutex_creation()?;
    test_mutex_lock_unlock()?;
    test_mutex_modify_data()?;
    test_mutex_multiple_locks()?;
    test_mutex_guard_drop()?;
    test_mutex_with_struct()?;
    test_mutex_recursive()?;
    test_mutex_drop()?;
    log_info!(TAG, "========== All Mutex Tests PASSED ==========");
    Ok(())
}
