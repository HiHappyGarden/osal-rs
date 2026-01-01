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
use osal_rs::os::{ToTick, FromTick};
use osal_rs::os::types::TickType;
use osal_rs::utils::Result;
use osal_rs::{log_debug, log_info};

const TAG: &str = "DurationTests";

pub fn test_duration_to_ticks() -> Result<()> {
    log_info!(TAG, "Starting test_duration_to_ticks");
    let duration = Duration::from_millis(100);
    let ticks = duration.to_ticks();
    log_debug!(TAG, "100ms = {} ticks", ticks);
    assert!(ticks > 0);
    log_info!(TAG, "test_duration_to_ticks PASSED");
    Ok(())
}

pub fn test_duration_from_ticks() -> Result<()> {
    log_info!(TAG, "Starting test_duration_from_ticks");
    let ticks: TickType = 1000;
    let mut duration = Duration::from_millis(0);
    duration.ticks(ticks);
    log_debug!(TAG, "1000 ticks = {} ms", duration.as_millis());
    assert!(duration.as_millis() > 0);
    log_info!(TAG, "test_duration_from_ticks PASSED");
    Ok(())
}

pub fn test_duration_conversion_roundtrip() -> Result<()> {
    log_info!(TAG, "Starting test_duration_conversion_roundtrip");
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
    log_debug!(TAG, "Original: {}ms, Converted: {}ms, Diff: {}ms", original.as_millis(), converted.as_millis(), diff.as_millis());
    assert!(diff.as_millis() < 10);
    log_info!(TAG, "test_duration_conversion_roundtrip PASSED");
    Ok(())
}

pub fn test_duration_zero() -> Result<()> {
    log_info!(TAG, "Starting test_duration_zero");
    let duration = Duration::from_millis(0);
    let ticks = duration.to_ticks();
    log_debug!(TAG, "Zero duration = {} ticks", ticks);
    assert_eq!(ticks, 0);
    log_info!(TAG, "test_duration_zero PASSED");
    Ok(())
}

pub fn test_duration_one_second() -> Result<()> {
    log_info!(TAG, "Starting test_duration_one_second");
    let duration = Duration::from_secs(1);
    let ticks = duration.to_ticks();
    log_debug!(TAG, "1 second = {} ticks", ticks);
    assert!(ticks >= 1000); // At least 1000 ticks for 1 second (1kHz tick rate)
    log_info!(TAG, "test_duration_one_second PASSED");
    Ok(())
}

pub fn test_duration_microseconds() -> Result<()> {
    log_info!(TAG, "Starting test_duration_microseconds");
    let duration = Duration::from_micros(1000); // 1 millisecond
    let ticks = duration.to_ticks();
    log_debug!(TAG, "1000 microseconds = {} ticks", ticks);
    assert!(ticks > 0);
    log_info!(TAG, "test_duration_microseconds PASSED");
    Ok(())
}

pub fn test_duration_large_value() -> Result<()> {
    log_info!(TAG, "Starting test_duration_large_value");
    let duration = Duration::from_secs(60); // 1 minute
    let ticks = duration.to_ticks();
    log_debug!(TAG, "60 seconds = {} ticks", ticks);
    assert!(ticks > 0);
    log_info!(TAG, "test_duration_large_value PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Duration Tests ==========");
    test_duration_to_ticks()?;
    test_duration_from_ticks()?;
    test_duration_conversion_roundtrip()?;
    test_duration_zero()?;
    test_duration_one_second()?;
    test_duration_microseconds()?;
    test_duration_large_value()?;
    log_info!(TAG, "========== All Duration Tests PASSED ==========");
    Ok(())
}
