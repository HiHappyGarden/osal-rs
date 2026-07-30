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

//! Ported 1:1 from osal-rs-tests' FreeRTOS suite (`system_tests.rs`) to run
//! against the POSIX backend. `posix::System` is backed by real
//! `std::time::Instant`/`std::thread::sleep`, so this module is expected to pass.

#![cfg(feature = "posix")]

use core::time::Duration;
use osal_rs::os::*;
use osal_rs::utils::{OsalRsBool, Result};
use osal_rs::{log_debug, log_info};

const TAG: &str = "SystemTests";

#[test]
fn test_system_get_tick_count() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_tick_count");
    let tick_count = System::get_tick_count();
    log_debug!(TAG, "Current tick count: {}", tick_count);
    assert!(tick_count > 0);
    log_info!(TAG, "test_system_get_tick_count PASSED");
    Ok(())
}

#[test]
fn test_system_get_current_time() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_current_time");
    let time = System::get_current_time();
    log_debug!(TAG, "Current time: {} us", time.as_micros());
    assert!(time.as_micros() > 0);
    log_info!(TAG, "test_system_get_current_time PASSED");
    Ok(())
}

#[test]
fn test_system_count_threads() -> Result<()> {
    log_info!(TAG, "Starting test_system_count_threads");
    let count = System::count_threads();
    log_debug!(TAG, "Number of threads: {}", count);
    assert!(count > 0); // At least the idle task should exist
    log_info!(TAG, "test_system_count_threads PASSED");
    Ok(())
}

#[test]
fn test_system_get_all_threads() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_all_threads");
    let state = System::get_all_thread();
    log_debug!(TAG, "Total threads: {}, Total runtime: {}", state.tasks.len(), state.total_run_time);
    assert!(state.tasks.len() > 0);
    assert!(state.total_run_time > 0);
    log_info!(TAG, "test_system_get_all_threads PASSED");
    Ok(())
}

#[test]
fn test_system_delay() -> Result<()> {
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

#[test]
fn test_system_delay_until() -> Result<()> {
    log_info!(TAG, "Starting test_system_delay_until");
    let mut wake_time = System::get_tick_count();
    let increment = Duration::from_millis(10).to_ticks();

    log_debug!(TAG, "Wake time: {}, Increment: {}", wake_time, increment);
    System::delay_until(&mut wake_time, increment);

    assert!(wake_time > 0);
    log_info!(TAG, "test_system_delay_until PASSED");
    Ok(())
}

#[test]
fn test_system_suspend_resume_all() -> Result<()> {
    log_info!(TAG, "Starting test_system_suspend_resume_all");
    log_debug!(TAG, "Suspending all threads");
    System::suspend_all();
    let result = System::resume_all();
    log_debug!(TAG, "Resumed all threads, result: {}", result);
    assert!(result >= 0);
    log_info!(TAG, "test_system_suspend_resume_all PASSED");
    Ok(())
}

#[test]
fn test_system_check_timer() -> Result<()> {
    log_info!(TAG, "Starting test_system_check_timer");
    let timestamp = System::get_current_time();
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

#[test]
fn test_system_get_free_heap_size() -> Result<()> {
    log_info!(TAG, "Starting test_system_get_free_heap_size");
    let heap_size = System::get_free_heap_size();
    log_debug!(TAG, "Free heap size: {} bytes", heap_size);
    assert!(heap_size > 0);
    log_info!(TAG, "test_system_get_free_heap_size PASSED");
    Ok(())
}

#[test]
fn test_system_time_conversion() -> Result<()> {
    log_info!(TAG, "Starting test_system_time_conversion");
    let duration = Duration::from_millis(100);
    let ticks = System::get_from_tick(&duration);
    log_debug!(TAG, "100ms = {} ticks", ticks);
    assert!(ticks > 0);
    log_info!(TAG, "test_system_time_conversion PASSED");
    Ok(())
}

#[test]
fn test_system_thread_metadata() -> Result<()> {
    log_info!(TAG, "Starting test_system_thread_metadata");
    let state = System::get_all_thread();

    for thread_meta in state.tasks.iter() {
        assert!(thread_meta.thread != 0);
        assert!(!thread_meta.name.is_empty());
        // 0 is a legitimate, real priority (POSIX SCHED_OTHER threads and
        // FreeRTOS's idle task both report it), not a sign of missing data.
        #[cfg(feature = "real_time")]
        {
            assert!(thread_meta.priority > 0);
        }
    }
    log_debug!(TAG, "Verified metadata for {} threads", state.tasks.len());
    log_info!(TAG, "test_system_thread_metadata PASSED");
    Ok(())
}

#[test]
fn test_system_multiple_delays() -> Result<()> {
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

#[test]
fn test_system_time_monotonic() -> Result<()> {
    log_info!(TAG, "Starting test_system_time_monotonic");
    let time1 = System::get_current_time();
    System::delay(Duration::from_millis(10).to_ticks());
    let time2 = System::get_current_time();

    log_debug!(TAG, "Time1: {} us, Time2: {} us", time1.as_micros(), time2.as_micros());
    assert!(time2 >= time1);
    log_info!(TAG, "test_system_time_monotonic PASSED");
    Ok(())
}

#[test]
fn test_system_delay_with_to_tick() -> Result<()> {
    log_info!(TAG, "Starting test_system_delay_with_to_tick");
    let start = System::get_tick_count();
    System::delay_with_to_tick(Duration::from_millis(10));
    let end = System::get_tick_count();
    log_debug!(TAG, "delay_with_to_tick: start={}, end={}", start, end);
    assert!(end >= start);
    log_info!(TAG, "test_system_delay_with_to_tick PASSED");
    Ok(())
}

#[test]
fn test_system_delay_until_with_to_tick() -> Result<()> {
    log_info!(TAG, "Starting test_system_delay_until_with_to_tick");
    let mut wake_time = System::get_tick_count();
    System::delay_until_with_to_tick(&mut wake_time, Duration::from_millis(10));
    log_debug!(TAG, "New wake time: {}", wake_time);
    assert!(wake_time > 0);
    log_info!(TAG, "test_system_delay_until_with_to_tick PASSED");
    Ok(())
}

// NOTE: `System::start`/`stop`, `yield_from_isr`, `end_switching_isr`,
// `critical_section_enter_from_isr`/`critical_section_exit_from_isr` are intentionally NOT
// invoked here. On the FreeRTOS backend `start`/`stop` control the real
// scheduler and calling them from an already-running task would hang or
// corrupt the test run; the ISR-only functions are only meaningful from
// actual interrupt context. All five are already signature-checked as
// function pointers in `std_api_surface.rs`.
