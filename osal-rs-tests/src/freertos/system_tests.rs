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

use core::time::Duration;
use osal_rs::os::*;
use osal_rs::utils::{Result, OsalRsBool};
use osal_rs::{log_debug, log_info};

const TAG: &str = "SystemTests";

pub fn test_system_get_tick_count() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_tick_count");
    let tick_count = System::get_tick_count();
    log_debug!(TAG, "Current tick count: {}", tick_count);
    assert!(tick_count > 0);
    log_info!(TAG, "test_system_get_tick_count PASSED");
    Ok(())
}

pub fn test_system_get_current_time() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_current_time");
    let time = System::get_current_time_us();
    log_debug!(TAG, "Current time: {} us", time.as_micros());
    assert!(time.as_micros() > 0);
    log_info!(TAG, "test_system_get_current_time PASSED");
    Ok(())
}

pub fn test_system_count_threads() -> Result<()> {
    log_info!(TAG, "Starting test_system_count_threads");
    let count = System::count_threads();
    log_debug!(TAG, "Number of threads: {}", count);
    assert!(count > 0); // At least the idle task should exist
    log_info!(TAG, "test_system_count_threads PASSED");
    Ok(())
}

pub fn test_system_get_all_threads() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_all_threads");
    let state = System::get_all_thread();
    log_debug!(TAG, "Total threads: {}, Total runtime: {}", state.tasks.len(), state.total_run_time);
    assert!(state.tasks.len() > 0);
    assert!(state.total_run_time > 0);
    log_info!(TAG, "test_system_get_all_threads PASSED");
    Ok(())
}

pub fn test_system_delay() -> Result<()> {
    log_info!(TAG, "Starting test_system_delay");
    let start = System::get_tick_count();
    log_debug!(TAG, "Delaying 10ms...");
    System::delay(Duration::from_millis(10).to_ticks());
    let end = System::get_tick_count();
    
    log_debug!(TAG, "Delay completed. Start: {}, End: {}", start, end);
    assert!(end >= start);
    log_info!(TAG, "test_system_delay PASSED");
    Ok(())
}

pub fn test_system_delay_until() -> Result<()> {
    log_info!(TAG, "Starting test_system_delay_until");
    let mut wake_time = System::get_tick_count();
    let increment = Duration::from_millis(10).to_ticks();
    
    log_debug!(TAG, "Wake time: {}, Increment: {}", wake_time, increment);
    System::delay_until(&mut wake_time, increment);
    
    assert!(wake_time > 0);
    log_info!(TAG, "test_system_delay_until PASSED");
    Ok(())
}

pub fn test_system_critical_section() -> Result<()> {
    log_info!(TAG, "Starting test_system_critical_section");
    log_debug!(TAG, "Entering critical section");
    System::critical_section_enter();
    // Critical section code
    System::critical_section_exit();
    log_debug!(TAG, "Exited critical section");
    log_info!(TAG, "test_system_critical_section PASSED");
    Ok(())
}

pub fn test_system_suspend_resume_all() -> Result<()> {
    log_info!(TAG, "Starting test_system_suspend_resume_all");
    log_debug!(TAG, "Suspending all threads");
    System::suspend_all();
    let result = System::resume_all();
    log_debug!(TAG, "Resumed all threads, result: {}", result);
    assert!(result >= 0);
    log_info!(TAG, "test_system_suspend_resume_all PASSED");
    Ok(())
}

pub fn test_system_check_timer() -> Result<()> {
    log_info!(TAG, "Starting test_system_check_timer");
    let timestamp = System::get_current_time_us();
    let wait_time = Duration::from_millis(10);
    
    // Should be false immediately
    let result = System::check_timer(&timestamp, &wait_time);
    log_debug!(TAG, "Check timer immediately: {:?}", result);
    assert_eq!(result, OsalRsBool::False);
    
    // Wait for the duration
    System::delay(wait_time.to_ticks());
    
    // Should be true after waiting
    let result = System::check_timer(&timestamp, &wait_time);
    log_debug!(TAG, "Check timer after delay: {:?}", result);
    assert_eq!(result, OsalRsBool::True);
    log_info!(TAG, "test_system_check_timer PASSED");
    Ok(())
}

pub fn test_system_get_free_heap_size() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_free_heap_size");
    let heap_size = System::get_free_heap_size();
    log_debug!(TAG, "Free heap size: {} bytes", heap_size);
    assert!(heap_size > 0);
    log_info!(TAG, "test_system_get_free_heap_size PASSED");
    Ok(())
}

pub fn test_system_get_state() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_state");
    let state = System::get_state();
    log_debug!(TAG, "Current thread state: {:?}", state);
    // Current thread should be in Running state
    assert!(matches!(state, ThreadState::Running | ThreadState::Ready));
    log_info!(TAG, "test_system_get_state PASSED");
    Ok(())
}

pub fn test_system_time_conversion() -> Result<()> {
    log_info!(TAG, "Starting test_system_time_conversion");
    let duration = Duration::from_millis(100);
    let ticks = System::get_us_from_tick(&duration);
    log_debug!(TAG, "100ms = {} ticks", ticks);
    assert!(ticks > 0);
    log_info!(TAG, "test_system_time_conversion PASSED");
    Ok(())
}

pub fn test_system_thread_metadata() -> Result<()> {
    log_info!(TAG, "Starting test_system_thread_metadata");
    let state = System::get_all_thread();
    
    for thread_meta in state.tasks.iter() {
        assert!(!thread_meta.thread.is_null());
        assert!(!thread_meta.name.is_empty());
        assert!(thread_meta.priority > 0);
    }
    log_debug!(TAG, "Verified metadata for {} threads", state.tasks.len());
    log_info!(TAG, "test_system_thread_metadata PASSED");
    Ok(())
}

pub fn test_system_multiple_delays() -> Result<()> {
    log_info!(TAG, "Starting test_system_multiple_delays");
    let start = System::get_tick_count();
    
    log_debug!(TAG, "Performing 3 delays of 5ms each");
    for _ in 0..3 {
        System::delay(Duration::from_millis(5).to_ticks());
    }
    
    let end = System::get_tick_count();
    log_debug!(TAG, "Total delay completed. Start: {}, End: {}", start, end);
    assert!(end > start);
    log_info!(TAG, "test_system_multiple_delays PASSED");
    Ok(())
}

pub fn test_system_time_monotonic() -> Result<()> {
    log_info!(TAG, "Starting test_system_time_monotonic");
    let time1 = System::get_current_time_us();
    System::delay(Duration::from_millis(10).to_ticks());
    let time2 = System::get_current_time_us();
    
    log_debug!(TAG, "Time1: {} us, Time2: {} us", time1.as_micros(), time2.as_micros());
    assert!(time2 >= time1);
    log_info!(TAG, "test_system_time_monotonic PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running System Tests ==========");
    test_system_get_tick_count()?;
    test_system_get_current_time()?;
    test_system_count_threads()?;
    test_system_get_all_threads()?;
    test_system_delay()?;
    test_system_delay_until()?;
    test_system_critical_section()?;
    test_system_suspend_resume_all()?;
    test_system_check_timer()?;
    test_system_get_free_heap_size()?;
    test_system_get_state()?;
    test_system_time_conversion()?;
    test_system_thread_metadata()?;
    test_system_multiple_delays()?;
    test_system_time_monotonic()?;
    log_info!(TAG, "========== All System Tests PASSED ==========");
    Ok(())
}
