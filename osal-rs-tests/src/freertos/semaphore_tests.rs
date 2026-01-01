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
use osal_rs::utils::{Result, OsalRsBool};
use core::time::Duration;
use osal_rs::{log_debug, log_info};

const TAG: &str = "SemaphoreTests";

pub fn test_semaphore_creation() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_creation");
    let semaphore = Semaphore::new(5, 0);
    assert!(semaphore.is_ok());
    log_info!(TAG, "test_semaphore_creation PASSED");
    Ok(())
}

pub fn test_semaphore_creation_with_count() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_creation_with_count");
    let semaphore = Semaphore::new_with_count(3);
    assert!(semaphore.is_ok());
    log_info!(TAG, "test_semaphore_creation_with_count PASSED");
    Ok(())
}

pub fn test_semaphore_signal_wait() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_signal_wait");
    let semaphore = Semaphore::new(5, 0)?;
    
    let signal_result = semaphore.signal();
    log_debug!(TAG, "Semaphore signal result: {:?}", signal_result);
    assert_eq!(signal_result, OsalRsBool::True);
    
    let wait_result = semaphore.wait(Duration::from_millis(100));
    log_debug!(TAG, "Semaphore wait result: {:?}", wait_result);
    assert_eq!(wait_result, OsalRsBool::True);
    log_info!(TAG, "test_semaphore_signal_wait PASSED");
    Ok(())
}

pub fn test_semaphore_wait_timeout() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_wait_timeout");
    let semaphore = Semaphore::new(5, 0)?;
    
    let wait_result = semaphore.wait(Duration::from_millis(10));
    log_debug!(TAG, "Wait timeout result: {:?}", wait_result);
    assert_eq!(wait_result, OsalRsBool::False);
    log_info!(TAG, "test_semaphore_wait_timeout PASSED");
    Ok(())
}

pub fn test_semaphore_multiple_signals() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_multiple_signals");
    let semaphore = Semaphore::new(10, 0)?;
    
    log_debug!(TAG, "Signaling 5 times...");
    for _ in 0..5 {
        let result = semaphore.signal();
        assert_eq!(result, OsalRsBool::True);
    }
    
    log_debug!(TAG, "Waiting 5 times...");
    for _ in 0..5 {
        let result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(result, OsalRsBool::True);
    }
    
    let result = semaphore.wait(Duration::from_millis(10));
    assert_eq!(result, OsalRsBool::False);
    log_info!(TAG, "test_semaphore_multiple_signals PASSED");
    Ok(())
}

pub fn test_semaphore_max_count() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_max_count");
    let semaphore = Semaphore::new(3, 0)?;
    
    for _ in 0..3 {
        semaphore.signal();
    }
    log_debug!(TAG, "Signaled 3 times (max count)");
    
    for _ in 0..3 {
        let result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(result, OsalRsBool::True);
    }
    log_info!(TAG, "test_semaphore_max_count PASSED");
    Ok(())
}

pub fn test_semaphore_initial_count() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_initial_count");
    let semaphore = Semaphore::new(5, 3)?;
    
    log_debug!(TAG, "Testing initial count of 3...");
    for _ in 0..3 {
        let result = semaphore.wait(Duration::from_millis(100));
        assert_eq!(result, OsalRsBool::True);
    }
    
    let result = semaphore.wait(Duration::from_millis(10));
    assert_eq!(result, OsalRsBool::False);
    log_info!(TAG, "test_semaphore_initial_count PASSED");
    Ok(())
}

pub fn test_semaphore_binary() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_binary");
    let semaphore = Semaphore::new(1, 1)?;
    
    let result = semaphore.wait(Duration::from_millis(100));
    assert_eq!(result, OsalRsBool::True);
    
    let result = semaphore.wait(Duration::from_millis(10));
    assert_eq!(result, OsalRsBool::False);
    
    log_debug!(TAG, "Signaling binary semaphore...");
    semaphore.signal();
    
    let result = semaphore.wait(Duration::from_millis(100));
    assert_eq!(result, OsalRsBool::True);
    log_info!(TAG, "test_semaphore_binary PASSED");
    Ok(())
}

pub fn test_semaphore_drop() -> Result<()> {
    log_info!(TAG, "Starting test_semaphore_drop");
    let semaphore = Semaphore::new(5, 2)?;
    drop(semaphore);
    log_info!(TAG, "test_semaphore_drop PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Semaphore Tests ==========");
    test_semaphore_creation()?;
    test_semaphore_creation_with_count()?;
    test_semaphore_signal_wait()?;
    test_semaphore_wait_timeout()?;
    test_semaphore_multiple_signals()?;
    test_semaphore_max_count()?;
    test_semaphore_initial_count()?;
    test_semaphore_binary()?;
    test_semaphore_drop()?;
    log_info!(TAG, "========== All Semaphore Tests PASSED ==========");
    Ok(())
}
