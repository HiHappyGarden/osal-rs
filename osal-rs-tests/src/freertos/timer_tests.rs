extern crate alloc;

use alloc::sync::Arc;
use core::any::Any;
use core::sync::atomic::{AtomicU32, Ordering};
use osal_rs::os::*;
use osal_rs::utils::{Result, OsalRsBool};
use core::time::Duration;

pub fn test_timer_creation() -> Result<()> {
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
    Ok(())
}

pub fn test_timer_one_shot() -> Result<()> {
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
    assert_eq!(result, OsalRsBool::True);
    
    // Wait for timer to fire
    let _ = Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(200).to_ticks());
    
    assert!(COUNTER.load(Ordering::SeqCst) >= 1);
    Ok(())
}

pub fn test_timer_auto_reload() -> Result<()> {
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
    
    assert!(COUNTER.load(Ordering::SeqCst) >= 2);
    
    timer.stop(Duration::from_millis(10).to_ticks());
    Ok(())
}

pub fn test_timer_start_stop() -> Result<()> {
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
    assert_eq!(start_result, OsalRsBool::True);
    
    let stop_result = timer.stop(Duration::from_millis(10).to_ticks());
    assert_eq!(stop_result, OsalRsBool::True);
    Ok(())
}

pub fn test_timer_reset() -> Result<()> {
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
    assert_eq!(reset_result, OsalRsBool::True);
    
    timer.stop(Duration::from_millis(10).to_ticks());
    Ok(())
}

pub fn test_timer_change_period() -> Result<()> {
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
    
    let change_result = timer.change_period(
        Duration::from_millis(200).to_ticks(),
        Duration::from_millis(10).to_ticks()
    );
    assert_eq!(change_result, OsalRsBool::True);
    
    timer.stop(Duration::from_millis(10).to_ticks());
    Ok(())
}

pub fn test_timer_with_param() -> Result<()> {
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
    
    assert_eq!(RECEIVED_VALUE.load(Ordering::SeqCst), 42);
    Ok(())
}

pub fn test_timer_delete() -> Result<()> {
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
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    test_timer_creation()?;
    test_timer_one_shot()?;
    test_timer_auto_reload()?;
    test_timer_start_stop()?;
    test_timer_reset()?;
    test_timer_change_period()?;
    test_timer_with_param()?;
    test_timer_delete()?;
    Ok(())
}
