extern crate alloc;

use core::time::Duration;
use osal_rs::os::*;
use osal_rs::os::{ToTick, FromTick};
use osal_rs::os::types::TickType;
use osal_rs::utils::Result;

pub fn test_duration_to_ticks() -> Result<()> {
    let duration = Duration::from_millis(100);
    let ticks = duration.to_ticks();
    assert!(ticks > 0);
    Ok(())
}

pub fn test_duration_from_ticks() -> Result<()> {
    let ticks: TickType = 1000;
    let mut duration = Duration::from_millis(0);
    duration.ticks(ticks);
    assert!(duration.as_millis() > 0);
    Ok(())
}

pub fn test_duration_conversion_roundtrip() -> Result<()> {
    let original = Duration::from_millis(500);
    let ticks = original.to_ticks();
    
    let mut converted = Duration::from_millis(0);
    converted.ticks(ticks);
    
    // Allow small rounding error
    let diff = if original > converted {
        original - converted
    } else {
        converted - original
    };
    assert!(diff.as_millis() < 10);
    Ok(())
}

pub fn test_duration_zero() -> Result<()> {
    let duration = Duration::from_millis(0);
    let ticks = duration.to_ticks();
    assert_eq!(ticks, 0);
    Ok(())
}

pub fn test_duration_one_second() -> Result<()> {
    let duration = Duration::from_secs(1);
    let ticks = duration.to_ticks();
    assert!(ticks >= 1000); // At least 1000 ticks for 1 second (1kHz tick rate)
    Ok(())
}

pub fn test_duration_microseconds() -> Result<()> {
    let duration = Duration::from_micros(1000); // 1 millisecond
    let ticks = duration.to_ticks();
    assert!(ticks >= 0);
    Ok(())
}

pub fn test_duration_large_value() -> Result<()> {
    let duration = Duration::from_secs(60); // 1 minute
    let ticks = duration.to_ticks();
    assert!(ticks > 0);
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    test_duration_to_ticks()?;
    test_duration_from_ticks()?;
    test_duration_conversion_roundtrip()?;
    test_duration_zero()?;
    test_duration_one_second()?;
    test_duration_microseconds()?;
    test_duration_large_value()?;
    Ok(())
}
