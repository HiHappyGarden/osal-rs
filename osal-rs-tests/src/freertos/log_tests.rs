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

//! Tests for `osal_rs::log`: the level/colour mask accessors, the five
//! severity macros and the `print!`/`println!` shims.
//!
//! Mirrors `osal-rs/tests/std_log_tests.rs`; keep the two in sync. The mask is
//! a process-wide global, so everything is asserted from one function and the
//! original mask is restored on the way out.

extern crate alloc;

use osal_rs::log::{
    get_enable_log, get_level_log, is_enabled_log, log_levels, set_enable_color, set_enable_log,
    set_level_log, sys_log, LOG_BUFFER_SIZE, RETURN,
};
use osal_rs::utils::Result;
use osal_rs::{log_debug, log_error, log_fatal, log_info, log_warning, print, println};

const TAG: &str = "LogTests";

pub fn test_log_mask_and_macros() -> Result<()> {
    log_info!(TAG, "Starting test_log_mask_and_macros");

    // --- constants -------------------------------------------------------
    assert_eq!(LOG_BUFFER_SIZE, 256);
    assert_eq!(RETURN, "\r\n");

    // Level constants are cumulative: each one is a superset of the stricter
    // level above it.
    assert_eq!(
        log_levels::LEVEL_DEBUG,
        log_levels::FLAG_DEBUG | log_levels::LEVEL_INFO
    );
    assert_eq!(
        log_levels::LEVEL_INFO,
        log_levels::FLAG_INFO | log_levels::LEVEL_WARNING
    );
    assert_eq!(
        log_levels::LEVEL_WARNING,
        log_levels::FLAG_WARNING | log_levels::LEVEL_ERROR
    );
    assert_eq!(
        log_levels::LEVEL_ERROR,
        log_levels::FLAG_ERROR | log_levels::LEVEL_FATAL
    );
    assert_eq!(log_levels::LEVEL_FATAL, log_levels::FLAG_FATAL);

    // --- enable / disable ------------------------------------------------
    assert!(get_enable_log(), "logging is on by default");

    set_enable_log(false);
    assert!(!get_enable_log());
    // With the state flag clear, *no* level reports as enabled regardless of
    // the level mask.
    assert!(!is_enabled_log(log_levels::FLAG_FATAL));
    assert!(!is_enabled_log(log_levels::FLAG_DEBUG));
    // Macros must be silent no-ops in this state (nothing to assert beyond
    // "does not panic"; the mask below proves the gate was taken).
    log_debug!(TAG, "suppressed debug");
    log_fatal!(TAG, "suppressed fatal");

    set_enable_log(true);
    assert!(get_enable_log());

    // --- level threshold -------------------------------------------------
    let original_level = get_level_log();

    set_level_log(log_levels::LEVEL_DEBUG);
    assert_eq!(get_level_log(), log_levels::LEVEL_DEBUG);
    assert!(is_enabled_log(log_levels::FLAG_DEBUG));
    assert!(is_enabled_log(log_levels::FLAG_INFO));
    assert!(is_enabled_log(log_levels::FLAG_WARNING));
    assert!(is_enabled_log(log_levels::FLAG_ERROR));
    assert!(is_enabled_log(log_levels::FLAG_FATAL));

    // All five macros, both the bare and the formatted form, at the most
    // permissive level so every one of them reaches `sys_log`.
    log_debug!(TAG, "debug message");
    log_info!(TAG, "info message");
    log_warning!(TAG, "warning message");
    log_error!(TAG, "error message");
    log_fatal!(TAG, "fatal message");
    log_debug!(TAG, "debug {} {}", 1, "arg");
    log_info!(TAG, "info {}", 2);
    log_warning!(TAG, "warning {}", 3);
    log_error!(TAG, "error {}", 4);
    log_fatal!(TAG, "fatal {}", 5);

    set_level_log(log_levels::LEVEL_ERROR);
    assert_eq!(get_level_log(), log_levels::LEVEL_ERROR);
    assert!(!is_enabled_log(log_levels::FLAG_DEBUG));
    assert!(!is_enabled_log(log_levels::FLAG_INFO));
    assert!(!is_enabled_log(log_levels::FLAG_WARNING));
    assert!(is_enabled_log(log_levels::FLAG_ERROR));
    assert!(is_enabled_log(log_levels::FLAG_FATAL));

    // Below-threshold macros take the early-return path inside the macro.
    log_debug!(TAG, "filtered out");
    log_info!(TAG, "filtered out");
    log_warning!(TAG, "filtered out");
    log_error!(TAG, "still printed");

    set_level_log(log_levels::LEVEL_FATAL);
    assert_eq!(get_level_log(), log_levels::LEVEL_FATAL);
    assert!(!is_enabled_log(log_levels::FLAG_ERROR));
    assert!(is_enabled_log(log_levels::FLAG_FATAL));

    // `set_level_log` must not clobber the state flag.
    assert!(get_enable_log());

    // --- colour ----------------------------------------------------------
    set_level_log(log_levels::LEVEL_DEBUG);

    set_enable_color(true);
    // One message per severity so every arm of `sys_log`'s colour match runs.
    sys_log(TAG, log_levels::FLAG_DEBUG, "coloured debug");
    sys_log(TAG, log_levels::FLAG_INFO, "coloured info");
    sys_log(TAG, log_levels::FLAG_WARNING, "coloured warning");
    sys_log(TAG, log_levels::FLAG_ERROR, "coloured error");
    sys_log(TAG, log_levels::FLAG_FATAL, "coloured fatal");
    // An unknown severity falls through to the default (no colour) arm.
    sys_log(TAG, 0, "coloured unknown");

    set_enable_color(false);
    // Colour off: `sys_log` takes the `else` branch that blanks both the
    // colour and the reset sequence.
    sys_log(TAG, log_levels::FLAG_DEBUG, "plain debug");
    sys_log(TAG, log_levels::FLAG_FATAL, "plain fatal");

    // Colour is not part of the level threshold.
    assert_eq!(get_level_log(), log_levels::LEVEL_DEBUG);

    set_enable_color(true);

    // --- print! / println! ----------------------------------------------
    print!("print without args\r\n");
    print!("print with {} {}\r\n", "two", "args");
    println!();
    println!("println without args");
    println!("println with {} args", 1);

    // --- restore ---------------------------------------------------------
    set_level_log(original_level);
    set_enable_log(true);
    set_enable_color(true);
    assert_eq!(get_level_log(), original_level);

    log_info!(TAG, "test_log_mask_and_macros PASSED");
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "========== Running Log Tests ==========");
    test_log_mask_and_macros()?;
    log_info!(TAG, "========== All Log Tests PASSED ==========");
    Ok(())
}
