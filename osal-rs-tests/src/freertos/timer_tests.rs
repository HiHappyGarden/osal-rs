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

pub fn test_timer_with_to_tick_variants() -> Result<()> {
    log_info!(TAG, "Starting test_timer_with_to_tick_variants");
    let mut timer = Timer::new_with_to_tick(
        "with_to_tick_timer",
        Duration::from_millis(100),
        false,
        None,
        |_timer, param| {
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    let start_result = timer.start_with_to_tick(Duration::from_millis(10));
    log_debug!(TAG, "start_with_to_tick: {:?}", start_result);
    assert_eq!(start_result, OsalRsBool::True);

    let reset_result = timer.reset_with_to_tick(Duration::from_millis(10));
    log_debug!(TAG, "reset_with_to_tick: {:?}", reset_result);
    assert_eq!(reset_result, OsalRsBool::True);

    let change_result = timer.change_period_with_to_tick(Duration::from_millis(200), Duration::from_millis(10));
    log_debug!(TAG, "change_period_with_to_tick: {:?}", change_result);
    assert_eq!(change_result, OsalRsBool::True);

    let stop_result = timer.stop_with_to_tick(Duration::from_millis(10));
    log_debug!(TAG, "stop_with_to_tick: {:?}", stop_result);
    assert_eq!(stop_result, OsalRsBool::True);

    let delete_result = timer.delete_with_to_tick(Duration::from_millis(10));
    log_debug!(TAG, "delete_with_to_tick: {:?}", delete_result);
    assert_eq!(delete_result, OsalRsBool::True);

    log_info!(TAG, "test_timer_with_to_tick_variants PASSED");
    Ok(())
}

// ---------------------------------------------------------------------------
// Ownership
//
// Mirrors the block of the same name at the end of the POSIX suite
// (`osal-rs/tests/std_timer_tests.rs`): both backends share one timer between
// every clone of a `Timer`, hand the callback a borrowed handle rather than an
// owning one, thread the callback's return value into the next firing, and
// destroy the timer when the last handle is dropped.
// ---------------------------------------------------------------------------

pub fn test_timer_callback_handle_is_not_owning() -> Result<()> {
    log_info!(TAG, "Starting test_timer_callback_handle_is_not_owning");

    static FIRES: AtomicU32 = AtomicU32::new(0);
    static NULL_SEEN: AtomicU32 = AtomicU32::new(0);
    FIRES.store(0, Ordering::SeqCst);
    NULL_SEEN.store(0, Ordering::SeqCst);

    // `TimerFnPtr` takes its `Box<dyn TimerFn>` by value and drops it on
    // return, so the handle the callback is given must not own the timer -
    // otherwise an auto-reload timer would delete itself at its own first
    // firing.
    let timer = Timer::new(
        "cb_handle_timer",
        Duration::from_millis(20).to_ticks(),
        true,
        None,
        |handle, param| {
            if handle.is_null() {
                NULL_SEEN.fetch_add(1, Ordering::SeqCst);
            }
            FIRES.fetch_add(1, Ordering::SeqCst);
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    timer.start(0);
    System::delay(Duration::from_millis(150).to_ticks());

    let fires = FIRES.load(Ordering::SeqCst);
    log_debug!(TAG, "auto-reload timer fired {} time(s)", fires);
    assert_eq!(NULL_SEEN.load(Ordering::SeqCst), 0);
    assert!(fires >= 3);

    log_info!(TAG, "test_timer_callback_handle_is_not_owning PASSED");
    Ok(())
}

pub fn test_timer_param_carries_forward() -> Result<()> {
    log_info!(TAG, "Starting test_timer_param_carries_forward");

    static LAST: AtomicU32 = AtomicU32::new(0);
    LAST.store(0, Ordering::SeqCst);

    // Each firing is handed whatever the previous one returned, which is what
    // makes `TimerFnPtr`'s `Result<TimerParam>` return type useful: an
    // auto-reload timer can carry state forward without a static of its own.
    let seed: TimerParam = Arc::new(1u32);

    let timer = Timer::new(
        "carry_forward_timer",
        Duration::from_millis(20).to_ticks(),
        true,
        Some(seed),
        |_handle, param| {
            let previous = param.and_then(|p| p.downcast_ref::<u32>().copied()).unwrap_or(0);
            LAST.store(previous, Ordering::SeqCst);
            let next: TimerParam = Arc::new(previous + 1);
            Ok(next)
        }
    )?;

    timer.start(0);
    System::delay(Duration::from_millis(150).to_ticks());
    timer.stop(0);

    let last = LAST.load(Ordering::SeqCst);
    log_debug!(TAG, "last parameter handed to the callback: {}", last);
    assert!(last >= 3);

    log_info!(TAG, "test_timer_param_carries_forward PASSED");
    Ok(())
}

pub fn test_timer_clone_shares_one_timer() -> Result<()> {
    log_info!(TAG, "Starting test_timer_clone_shares_one_timer");

    static FIRES: AtomicU32 = AtomicU32::new(0);
    FIRES.store(0, Ordering::SeqCst);

    let timer = Timer::new(
        "shared_timer",
        Duration::from_millis(20).to_ticks(),
        true,
        None,
        |_handle, param| {
            FIRES.fetch_add(1, Ordering::SeqCst);
            Ok(param.unwrap_or_else(|| Arc::new(())))
        }
    )?;

    let mut clone = timer.clone();

    // Starting through one handle and stopping through the other has to act
    // on the same timer.
    assert_eq!(timer.start(0), OsalRsBool::True);
    System::delay(Duration::from_millis(80).to_ticks());
    assert_eq!(clone.stop(0), OsalRsBool::True);

    let fired = FIRES.load(Ordering::SeqCst);
    assert!(fired >= 2);

    // And a deletion through one handle has to be visible from the other.
    assert_eq!(clone.delete(0), OsalRsBool::True);
    assert!(clone.is_null());
    assert!(timer.is_null());
    assert_eq!(timer.start(0), OsalRsBool::False);

    log_info!(TAG, "test_timer_clone_shares_one_timer PASSED");
    Ok(())
}

pub fn test_timer_dropping_last_handle_stops_it() -> Result<()> {
    log_info!(TAG, "Starting test_timer_dropping_last_handle_stops_it");

    static FIRES: AtomicU32 = AtomicU32::new(0);
    FIRES.store(0, Ordering::SeqCst);

    {
        let timer = Timer::new(
            "dropped_timer",
            Duration::from_millis(20).to_ticks(),
            true,
            None,
            |_handle, param| {
                FIRES.fetch_add(1, Ordering::SeqCst);
                Ok(param.unwrap_or_else(|| Arc::new(())))
            }
        )?;

        // A clone keeps the timer alive; only the *last* handle going away
        // may tear it down.
        let clone = timer.clone();
        assert_eq!(timer.start(0), OsalRsBool::True);
        System::delay(Duration::from_millis(80).to_ticks());
        drop(timer);

        System::delay(Duration::from_millis(50).to_ticks());
        assert!(!clone.is_null());
    }

    let fired_while_alive = FIRES.load(Ordering::SeqCst);
    log_debug!(TAG, "fired {} time(s) before the last handle was dropped", fired_while_alive);
    assert!(fired_while_alive >= 2);

    // On this backend the teardown hands the final deletion to the timer
    // daemon (see `TimerShared::drop`), so allow it a few ticks to land.
    System::delay(Duration::from_millis(120).to_ticks());

    assert_eq!(FIRES.load(Ordering::SeqCst), fired_while_alive);

    log_info!(TAG, "test_timer_dropping_last_handle_stops_it PASSED");
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
    test_timer_with_to_tick_variants()?;
    test_timer_delete()?;
    test_timer_callback_handle_is_not_owning()?;
    test_timer_param_carries_forward()?;
    test_timer_clone_shares_one_timer()?;
    test_timer_dropping_last_handle_stops_it()?;
    log_info!(TAG, "========== All Timer Tests PASSED ==========");
    Ok(())
}
