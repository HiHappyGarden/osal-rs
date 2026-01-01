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

use alloc::sync::Arc;
use core::any::Any;
use core::sync::atomic::{AtomicU32, Ordering};
use osal_rs::os::*;
use osal_rs::utils::{Result, OsalRsBool};
use core::time::Duration;
use osal_rs::{log_debug, log_info};

const TAG: &str = "TimerTests";

pub fn test_timer_creation() -> Result<()> {
    log_info!(TAG, "Starting test_timer_creation");
    let timer = Timer::new(
        "test_timer",
        Duration::from_millis(100).to_ticks(),
        false,
        None,
        |_timer, param| {
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    );

    assert!(timer.is_ok());
    log_info!(TAG, "test_timer_creation PASSED");
    Ok(())
}

pub fn test_timer_one_shot() -> Result<()> {
    log_info!(TAG, "Starting test_timer_one_shot");
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    
    let timer = Timer::new(
        "oneshot_timer",
        Duration::from_millis(50).to_ticks(),
        false,
        None,
        |_timer, param| {
            COUNTER.fetch_add(1, Ordering::SeqCst);
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    let result = timer.start(Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Timer started, waiting for fire...");
    assert_eq!(result, OsalRsBool::True);
    
    // Wait for timer to fire
    let _ = Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(200).to_ticks());
    
    let count = COUNTER.load(Ordering::SeqCst);
    log_debug!(TAG, "Timer fired {} times", count);
    assert!(count >= 1);
    log_info!(TAG, "test_timer_one_shot PASSED");
    Ok(())
}

pub fn test_timer_auto_reload() -> Result<()> {
    log_info!(TAG, "Starting test_timer_auto_reload");
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    
    let timer = Timer::new(
        "autoreload_timer",
        Duration::from_millis(50).to_ticks(),
        true,
        None,
        |_timer, param| {
            COUNTER.fetch_add(1, Ordering::SeqCst);
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    let result = timer.start(Duration::from_millis(10).to_ticks());
    assert_eq!(result, OsalRsBool::True);
    
    let _ = Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(300).to_ticks());
    
    let count = COUNTER.load(Ordering::SeqCst);
    log_debug!(TAG, "Auto-reload timer fired {} times", count);
    assert!(count >= 2);
    
    timer.stop(Duration::from_millis(10).to_ticks());
    log_info!(TAG, "test_timer_auto_reload PASSED");
    Ok(())
}

pub fn test_timer_start_stop() -> Result<()> {
    log_info!(TAG, "Starting test_timer_start_stop");
    let timer = Timer::new(
        "startstop_timer",
        Duration::from_millis(100).to_ticks(),
        false,
        None,
        |_timer, param| {
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    let start_result = timer.start(Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Timer started");
    assert_eq!(start_result, OsalRsBool::True);
    
    let stop_result = timer.stop(Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Timer stopped");
    assert_eq!(stop_result, OsalRsBool::True);
    log_info!(TAG, "test_timer_start_stop PASSED");
    Ok(())
}

pub fn test_timer_reset() -> Result<()> {
    log_info!(TAG, "Starting test_timer_reset");
    let timer = Timer::new(
        "reset_timer",
        Duration::from_millis(100).to_ticks(),
        false,
        None,
        |_timer, param| {
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    timer.start(Duration::from_millis(10).to_ticks());
    
    let reset_result = timer.reset(Duration::from_millis(10).to_ticks());
    log_debug!(TAG, "Timer reset");
    assert_eq!(reset_result, OsalRsBool::True);
    
    timer.stop(Duration::from_millis(10).to_ticks());
    log_info!(TAG, "test_timer_reset PASSED");
    Ok(())
}

pub fn test_timer_change_period() -> Result<()> {
    log_info!(TAG, "Starting test_timer_change_period");
    let timer = Timer::new(
        "period_timer",
        Duration::from_millis(100).to_ticks(),
        false,
        None,
        |_timer, param| {
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    timer.start(Duration::from_millis(10).to_ticks());
    
    log_debug!(TAG, "Changing period from 100ms to 200ms");
    let change_result = timer.change_period(
        Duration::from_millis(200).to_ticks(),
        Duration::from_millis(10).to_ticks()
    );
    assert_eq!(change_result, OsalRsBool::True);
    
    timer.stop(Duration::from_millis(10).to_ticks());
    log_info!(TAG, "test_timer_change_period PASSED");
    Ok(())
}

pub fn test_timer_with_param() -> Result<()> {
    log_info!(TAG, "Starting test_timer_with_param");
    let test_value: u32 = 42;
    let param: Arc<dyn Any + Send + Sync> = Arc::new(test_value);
    
    static RECEIVED_VALUE: AtomicU32 = AtomicU32::new(0);
    
    let timer = Timer::new(
        "param_timer",
        Duration::from_millis(50).to_ticks(),
        false,
        Some(param),
        |_timer, param| {
            if let Some(ref p) = param {
                if let Some(val) = p.downcast_ref::<u32>() {
                    RECEIVED_VALUE.store(*val, Ordering::SeqCst);
                }
            }
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    timer.start(Duration::from_millis(10).to_ticks());
    
    let _ = Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(200).to_ticks());
    
    let received = RECEIVED_VALUE.load(Ordering::SeqCst);
    log_debug!(TAG, "Received parameter value: {}", received);
    assert_eq!(received, 42);
    log_info!(TAG, "test_timer_with_param PASSED");
    Ok(())
}

pub fn test_timer_delete() -> Result<()> {
    log_info!(TAG, "Starting test_timer_delete");
    let mut timer = Timer::new(
        "delete_timer",
        Duration::from_millis(100).to_ticks(),
        false,
        None,
        |_timer, param| {
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    let delete_result = timer.delete(Duration::from_millis(10).to_ticks());
    assert_eq!(delete_result, OsalRsBool::True);
    log_info!(TAG, "test_timer_delete PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Timer Tests ==========");
    test_timer_creation()?;
    test_timer_one_shot()?;
    test_timer_auto_reload()?;
    test_timer_start_stop()?;
    test_timer_reset()?;
    test_timer_change_period()?;
    test_timer_with_param()?;
    test_timer_delete()?;
    log_info!(TAG, "========== All Timer Tests PASSED ==========");
    Ok(())
}
