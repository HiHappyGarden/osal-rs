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

//! Scheduler-lifecycle and ISR-compatibility tests for the POSIX `System`.
//!
//! These live in their own test binary (i.e. their own process) on purpose:
//!
//! * `System::start`/`stop` share a single process-wide `RUN` flag that
//!   `stop()` clears permanently, so a second `start()` in the same process
//!   would return without ever entering its loop.
//! * `System::suspend_all`/`resume_all` act on *every* thread registered by
//!   this crate, including helper threads belonging to whatever other test
//!   happens to be running concurrently in the same binary.
//!
//! Both hazards are avoided by keeping them here, and by sequencing the two
//! inside a single `#[test]` (Cargo runs the tests of one binary on parallel
//! threads).
//!
//! POSIX-only: on the FreeRTOS backend `start`/`stop` drive the real
//! scheduler, and calling them from an already-running task would hang the
//! run - see the note at the end of `std_system_tests.rs`.

#![cfg(feature = "posix")]

use core::ptr::null_mut;
use core::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use osal_rs::os::*;
use osal_rs::utils::Result;
use osal_rs::{log_debug, log_info};

const TAG: &str = "SystemLifecycleTests";

#[test]
fn test_system_suspend_resume_all_then_start_stop() -> Result<()> {
    log_info!(TAG, "Starting test_system_suspend_resume_all_then_start_stop");

    // --- suspend_all / resume_all with a live worker ---------------------
    let counter = Arc::new(AtomicU32::new(0));
    let keep_going = Arc::new(AtomicBool::new(true));

    let worker_counter = counter.clone();
    let worker_keep_going = keep_going.clone();

    let mut worker = Thread::new("lifecycle-worker", 8192, 5);
    let spawned_worker = worker.spawn_simple(move || {
        while worker_keep_going.load(Ordering::Acquire) {
            worker_counter.fetch_add(1, Ordering::AcqRel);
            System::delay(5);
        }
        Ok(Arc::new(()))
    })?;

    // Let it get going so it is registered as Ready/Running rather than
    // still starting up.
    System::delay(40);
    let running_at = counter.load(Ordering::Acquire);
    assert!(running_at > 0, "worker should have made progress");

    System::suspend_all();
    System::delay(40);
    let suspended_at = counter.load(Ordering::Acquire);
    log_debug!(TAG, "counter frozen at {} while suspended", suspended_at);

    let resumed = System::resume_all();
    log_debug!(TAG, "resume_all resumed {} thread(s)", resumed);
    assert!(resumed >= 1, "the suspended worker should have been resumed");

    System::delay(40);
    assert!(
        counter.load(Ordering::Acquire) > suspended_at,
        "worker should make progress again after resume_all"
    );

    keep_going.store(false, Ordering::Release);
    spawned_worker.join(null_mut())?;

    // A second `resume_all` with nothing suspended resumes nothing.
    assert_eq!(System::resume_all(), 0);

    // --- start / stop ----------------------------------------------------
    // `start()` spins on `RUN` with a 500ms delay per iteration, so the
    // stopper must clear the flag from another thread for it to return.
    let stopped = Arc::new(AtomicBool::new(false));
    let stopper_mark = stopped.clone();

    let mut stopper = Thread::new("lifecycle-stopper", 8192, 5);
    let spawned_stopper = stopper.spawn_simple(move || {
        System::delay(50);
        stopper_mark.store(true, Ordering::Release);
        System::stop();
        Ok(Arc::new(()))
    })?;

    let before = System::get_current_time();
    System::start();
    let elapsed = System::get_current_time().checked_sub(before).unwrap_or_default();

    log_debug!(TAG, "System::start returned after {:?}", elapsed);
    assert!(stopped.load(Ordering::Acquire), "stop() must have run first");

    spawned_stopper.join(null_mut())?;

    // `RUN` stays cleared, so a subsequent `start()` returns without ever
    // entering the delay.
    System::start();

    log_info!(TAG, "test_system_suspend_resume_all_then_start_stop PASSED");
    Ok(())
}

#[test]
fn test_system_isr_yield_helpers() -> Result<()> {
    log_info!(TAG, "Starting test_system_isr_yield_helpers");

    // Non-zero asks for a reschedule (`sched_yield`), zero is a no-op; both
    // arms must be side-effect free from the caller's point of view.
    System::yield_from_isr(1);
    System::yield_from_isr(0);
    System::yield_from_isr(-1);

    System::end_switching_isr(1);
    System::end_switching_isr(0);
    System::end_switching_isr(-1);

    log_info!(TAG, "test_system_isr_yield_helpers PASSED");
    Ok(())
}

#[test]
fn test_system_critical_sections() -> Result<()> {
    log_info!(TAG, "Starting test_system_critical_sections");

    // Task-level critical section: no-ops on POSIX, but must remain callable
    // and nestable so portable code compiles and runs unchanged.
    System::critical_section_enter();
    System::critical_section_enter();
    System::critical_section_exit();
    System::critical_section_exit();

    // ISR-level: `enter` returns the saved interrupt state to hand back to
    // `exit`. POSIX has nothing to save, so it is always 0.
    let saved = System::critical_section_enter_from_isr();
    log_debug!(TAG, "critical_section_enter_from_isr -> {}", saved);
    assert_eq!(saved, 0);
    System::critical_section_exit_from_isr(saved);

    let nested_outer = System::critical_section_enter_from_isr();
    let nested_inner = System::critical_section_enter_from_isr();
    System::critical_section_exit_from_isr(nested_inner);
    System::critical_section_exit_from_isr(nested_outer);

    log_info!(TAG, "test_system_critical_sections PASSED");
    Ok(())
}

#[test]
#[allow(deprecated)] // the point of this test is the deprecated aliases
fn test_system_deprecated_time_aliases() -> Result<()> {
    log_info!(TAG, "Starting test_system_deprecated_time_aliases");

    // `get_current_time_ms` is a pre-rename alias of `get_current_time`, so
    // both must read the same clock.
    let before = System::get_current_time();
    let alias = System::get_current_time_ms();
    let after = System::get_current_time();
    log_debug!(TAG, "current_time_ms {:?} between {:?} and {:?}", alias, before, after);
    assert!(alias >= before && alias <= after);

    // Same for `get_ms_from_tick` vs `get_from_tick`.
    for millis in [0u64, 1, 250, 1_000] {
        let duration = Duration::from_millis(millis);
        assert_eq!(
            System::get_ms_from_tick(&duration),
            System::get_from_tick(&duration)
        );
        assert_eq!(System::get_from_tick(&duration), millis as types::TickType);
    }

    log_info!(TAG, "test_system_deprecated_time_aliases PASSED");
    Ok(())
}

#[test]
fn test_system_check_timer_before_timestamp() -> Result<()> {
    log_info!(TAG, "Starting test_system_check_timer_before_timestamp");

    // A timestamp in the future underflows the subtraction; `check_timer`
    // saturates to zero elapsed rather than panicking.
    let future = System::get_current_time() + Duration::from_secs(60);
    assert_eq!(
        System::check_timer(&future, &Duration::from_millis(1)),
        osal_rs::utils::OsalRsBool::False
    );

    // A zero threshold is always already elapsed.
    let now = System::get_current_time();
    assert_eq!(
        System::check_timer(&now, &Duration::ZERO),
        osal_rs::utils::OsalRsBool::True
    );

    log_info!(TAG, "test_system_check_timer_before_timestamp PASSED");
    Ok(())
}

#[test]
fn test_system_delay_until_already_past() -> Result<()> {
    log_info!(TAG, "Starting test_system_delay_until_already_past");

    // Wake time already in the past: `delay_until` must not sleep, but must
    // still advance `previous_wake_time` by the full increment.
    let mut previous = System::get_tick_count().saturating_sub(1_000);
    let expected = previous + 10;

    let before = System::get_current_time();
    System::delay_until(&mut previous, 10);
    let elapsed = System::get_current_time().checked_sub(before).unwrap_or_default();

    assert_eq!(previous, expected);
    assert!(
        elapsed < Duration::from_millis(10),
        "a wake time in the past must not sleep, slept {:?}",
        elapsed
    );

    // A zero increment is the degenerate case of the same branch: the wake
    // time never moves and nothing sleeps.
    let mut unchanged = System::get_tick_count().saturating_sub(1_000);
    let snapshot = unchanged;
    System::delay_until(&mut unchanged, 0);
    assert_eq!(unchanged, snapshot);

    log_info!(TAG, "test_system_delay_until_already_past PASSED");
    Ok(())
}

#[test]
fn test_system_state_deref() -> Result<()> {
    log_info!(TAG, "Starting test_system_state_deref");

    let state = System::get_all_thread();

    // `SystemState` derefs to its task slice, so both spellings must agree.
    assert_eq!(state.len(), state.tasks.len());
    for (from_deref, from_field) in state.iter().zip(state.tasks.iter()) {
        assert_eq!(from_deref.thread, from_field.thread);
    }
    log_debug!(TAG, "SystemState holds {} task(s)", state.len());

    log_info!(TAG, "test_system_state_deref PASSED");
    Ok(())
}
