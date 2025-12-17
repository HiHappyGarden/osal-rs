extern crate alloc;

use alloc::boxed::Box;
use core::time::Duration;
use osal_rs::os::*;
use osal_rs::utils::{Result, OsalRsBool};

pub fn test_system_get_tick_count() -> Result<()> {
    let tick_count = System::get_tick_count();
    assert!(tick_count >= 0);
    Ok(())
}

pub fn test_system_get_current_time() -> Result<()> {
    let time = System::get_current_time_us();
    assert!(time.as_micros() >= 0);
    Ok(())
}

pub fn test_system_count_threads() -> Result<()> {
    let count = System::count_threads();
    assert!(count > 0); // At least the idle task should exist
    Ok(())
}

pub fn test_system_get_all_threads() -> Result<()> {
    let state = System::get_all_thread();
    assert!(state.tasks.len() > 0);
    assert!(state.total_run_time >= 0);
    Ok(())
}

pub fn test_system_delay() -> Result<()> {
    let start = System::get_tick_count();
    System::delay(Duration::from_millis(10).to_ticks());
    let end = System::get_tick_count();
    
    assert!(end >= start);
    Ok(())
}

pub fn test_system_delay_until() -> Result<()> {
    let mut wake_time = System::get_tick_count();
    let increment = Duration::from_millis(10).to_ticks();
    
    System::delay_until(&mut wake_time, increment);
    
    assert!(wake_time > 0);
    Ok(())
}

pub fn test_system_critical_section() -> Result<()> {
    System::critical_section_enter();
    // Critical section code
    System::critical_section_exit();
    Ok(())
}

pub fn test_system_suspend_resume_all() -> Result<()> {
    System::suspend_all();
    let result = System::resume_all();
    assert!(result >= 0);
    Ok(())
}

pub fn test_system_check_timer() -> Result<()> {
    let timestamp = System::get_current_time_us();
    let wait_time = Duration::from_millis(10);
    
    // Should be false immediately
    let result = System::check_timer(&timestamp, &wait_time);
    assert_eq!(result, OsalRsBool::False);
    
    // Wait for the duration
    System::delay(wait_time.to_ticks());
    
    // Should be true after waiting
    let result = System::check_timer(&timestamp, &wait_time);
    assert_eq!(result, OsalRsBool::True);
    Ok(())
}

pub fn test_system_get_free_heap_size() -> Result<()> {
    let heap_size = System::get_free_heap_size();
    assert!(heap_size > 0);
    Ok(())
}

pub fn test_system_get_state() -> Result<()> {
    let state = System::get_state();
    // Current thread should be in Running state
    assert!(matches!(state, ThreadState::Running | ThreadState::Ready));
    Ok(())
}

pub fn test_system_time_conversion() -> Result<()> {
    let duration = Duration::from_millis(100);
    let ticks = System::get_us_from_tick(&duration);
    assert!(ticks >= 0);
    Ok(())
}

pub fn test_system_thread_metadata() -> Result<()> {
    let state = System::get_all_thread();
    
    for thread_meta in state.tasks.iter() {
        assert!(!thread_meta.thread.is_null());
        assert!(!thread_meta.name.is_empty());
        assert!(thread_meta.priority >= 0);
    }
    Ok(())
}

pub fn test_system_multiple_delays() -> Result<()> {
    let start = System::get_tick_count();
    
    for _ in 0..3 {
        System::delay(Duration::from_millis(5).to_ticks());
    }
    
    let end = System::get_tick_count();
    assert!(end > start);
    Ok(())
}

pub fn test_system_time_monotonic() -> Result<()> {
    let time1 = System::get_current_time_us();
    System::delay(Duration::from_millis(10).to_ticks());
    let time2 = System::get_current_time_us();
    
    assert!(time2 >= time1);
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
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
    Ok(())
}
