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
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 *
 ***************************************************************************/

pub mod thread_tests;
pub mod queue_tests;
pub mod mutex_tests;
pub mod semaphore_tests;
pub mod timer_tests;
pub mod event_group_tests;
pub mod duration_tests;
pub mod system_tests;

use osal_rs::utils::Result;
use osal_rs::log_info;

const TAG: &str = "FreeRTOSTests";

/// Run all available FreeRTOS tests
pub fn run_all_tests() -> Result<()> {
    log_info!(TAG, "\n\n========================================");
    log_info!(TAG, "   Starting FreeRTOS Test Suite");
    log_info!(TAG, "========================================\n");
    
    duration_tests::run_all_tests()?;
    event_group_tests::run_all_tests()?;
    mutex_tests::run_all_tests()?;
    queue_tests::run_all_tests()?;
    semaphore_tests::run_all_tests()?;
    system_tests::run_all_tests()?;
    thread_tests::run_all_tests()?;
    timer_tests::run_all_tests()?;
    
    log_info!(TAG, "\n========================================");
    log_info!(TAG, "   All FreeRTOS Tests PASSED!");
    log_info!(TAG, "========================================\n");
    Ok(())
}
