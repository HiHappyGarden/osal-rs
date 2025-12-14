
use core::fmt::Debug;
use core::ops::Deref;
use core::time::Duration;

use alloc::vec::Vec;

use super::ffi::{
    BLOCKED, DELETED, READY, RUNNING, SUSPENDED, TaskStatus, eTaskGetState, osal_rs_critical_section_enter, osal_rs_critical_section_exit, osal_rs_port_end_switching_isr, osal_rs_port_yield_from_isr, uxTaskGetNumberOfTasks, uxTaskGetSystemState, vTaskDelay, vTaskEndScheduler, vTaskStartScheduler, vTaskSuspendAll, xPortGetFreeHeapSize, xTaskDelayUntil, xTaskGetCurrentTaskHandle, xTaskGetTickCount, xTaskResumeAll
};
use super::thread::{ThreadState, ThreadMetadata};
use super::types::{BaseType, TickType, UBaseType};
use crate::tick_period_ms;
use crate::traits::{SystemFn, ToTick};
use crate::utils::{CpuRegisterSize::*, register_bit_size, OsalRsBool};

#[derive(Debug, Clone)]
pub struct SystemState {
    pub tasks: Vec<ThreadMetadata>,
    pub total_run_time: u32
}

impl Deref for SystemState {
    type Target = [ThreadMetadata];

    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

pub struct System;

impl SystemFn for System {
    fn start() {
        unsafe {
            vTaskStartScheduler();
        }
    }

    fn get_state() -> ThreadState {
        use super::thread::ThreadState::*;
        let state = unsafe { eTaskGetState(xTaskGetCurrentTaskHandle()) };
        match state {
            RUNNING => Running,
            READY => Ready,
            BLOCKED => Blocked,
            SUSPENDED => Suspended,
            DELETED => Deleted,
            _ => Invalid, // INVALID or unknown state
        }
    }

    fn suspend_all() {
        unsafe {
            vTaskSuspendAll();
        }
    }
    fn resume_all() -> BaseType {
        unsafe { xTaskResumeAll() }
    }

    fn stop() {
        unsafe {
            vTaskEndScheduler();
        }
    }

    fn get_tick_count() -> TickType {
        unsafe { xTaskGetTickCount() }
    }

    fn get_current_time_us () -> Duration {
        let ticks = Self::get_tick_count();
        Duration::from_millis( 1_000 * ticks as u64 / tick_period_ms!() as u64 )
    }

    fn get_us_from_tick(duration: &Duration) -> TickType {
        let millis = duration.as_millis() as TickType;
        millis / (1_000 * tick_period_ms!() as TickType) 
    }

    fn count_threads() -> usize {
        unsafe { uxTaskGetNumberOfTasks() as usize }
    }

    fn get_all_thread() -> SystemState {
        let threads_count = Self::count_threads();
        let mut threads: Vec<TaskStatus> = Vec::with_capacity(threads_count);
        let mut total_run_time: u32 = 0;

        unsafe {
            let count = uxTaskGetSystemState(
                threads.as_mut_ptr(),
                threads_count as UBaseType,
                &mut total_run_time as *mut u32,
            ) as usize;
            
            // Set the length only after data has been written by FreeRTOS
            threads.set_len(count);
        }

        let tasks = threads.into_iter()
            .map(|task_status| {
                ThreadMetadata::from((
                    task_status.xHandle, 
                    task_status
                ))
            }).collect();

        SystemState {
            tasks,
            total_run_time
        }
    }


    fn delay(ticks: impl ToTick){
        unsafe {
            vTaskDelay(ticks.to_tick());
        }
    }

    fn delay_until(previous_wake_time: &mut TickType, time_increment: impl ToTick) {
        unsafe {
            xTaskDelayUntil(
                previous_wake_time,
                time_increment.to_tick(),
            );
        }
    }

    fn critical_section_enter() {
        unsafe {
            osal_rs_critical_section_enter();
        }
    }
    
    fn critical_section_exit() {
        unsafe {
            osal_rs_critical_section_exit();
        }   
    }
    
    fn check_timer(timestamp: &Duration, time: &Duration) -> OsalRsBool {
        let temp_tick_time = Self::get_current_time_us();
        
        let time_passing = if temp_tick_time >= *timestamp {
            temp_tick_time - *timestamp
        } else {
            if register_bit_size() == Bit32 {
                // Handle tick count overflow for 32-bit TickType
                let overflow_correction = Duration::from_micros(0xff_ff_ff_ff_u64);
                overflow_correction - *timestamp + temp_tick_time
            } else {
                // Handle tick count overflow for 64-bit TickType
                let overflow_correction = Duration::from_micros(0xff_ff_ff_ff_ff_ff_ff_ff_u64);
                overflow_correction - *timestamp + temp_tick_time
            }
        };

        if time_passing >= *time {
            OsalRsBool::True
        } else {
            OsalRsBool::False
        }
    }

    fn yield_from_isr(higher_priority_task_woken: BaseType) {
        unsafe {
            osal_rs_port_yield_from_isr(higher_priority_task_woken);
        }
    }

    fn end_switching_isr( switch_required: BaseType ) {
        unsafe {
            osal_rs_port_end_switching_isr( switch_required );
        }
    }

    fn get_free_heap_size() -> usize {
        unsafe {
            xPortGetFreeHeapSize()
        }
    }

}

