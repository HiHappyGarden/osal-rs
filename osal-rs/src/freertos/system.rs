
use core::ops::Deref;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::{SystemTrait};
use crate::freertos::ffi::{
    BLOCKED, DELETED, READY, RUNNING, SUSPENDED, TaskStatus, eTaskGetState, uxTaskGetNumberOfTasks, uxTaskGetSystemState, vTaskEndScheduler, vTaskStartScheduler, vTaskSuspendAll, xTaskGetCurrentTaskHandle, xTaskGetTickCount, xTaskResumeAll
};
use crate::freertos::thread::{ ThreadState, ThreadMetadata, Thread};
use crate::freertos::types::{BaseType, TickType, UBaseType};

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

impl SystemTrait for System {
    fn start() {
        unsafe {
            vTaskStartScheduler();
        }
    }

    fn get_state() -> ThreadState {
        let state = unsafe { eTaskGetState(xTaskGetCurrentTaskHandle()) };
        match state {
            RUNNING => ThreadState::Running,
            READY => ThreadState::Ready,
            BLOCKED => ThreadState::Blocked,
            SUSPENDED => ThreadState::Suspended,
            DELETED => ThreadState::Deleted,
            _ => ThreadState::Invalid, // INVALID or unknown state
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

    fn count_threads() -> usize {
        unsafe { uxTaskGetNumberOfTasks() as usize }
    }

    fn get_all_thread() -> SystemState {
        let threads_count = Self::count_threads();
        let mut threads: Vec<TaskStatus> = Vec::with_capacity(threads_count);
        let mut total_run_time: u32 = 0;

        unsafe {

            let retrieved_threads = uxTaskGetSystemState(
                threads.as_mut_ptr(),
                threads_count as UBaseType,
                &mut total_run_time as *mut u32,
            ) as usize;

            threads.set_len(retrieved_threads);
        }

        let tasks = threads.into_iter()
        .map(|t| {
            ThreadMetadata {
                thread: Thread::new(t.xHandle, 
                unsafe {
                        let c_str = core::ffi::CStr::from_ptr(t.pcTaskName);
                        String::from_utf8_lossy(c_str.to_bytes()).to_string()
                    }),
                thread_number: t.xTaskNumber,
                state: match t.eCurrentState {
                    RUNNING => ThreadState::Running,
                    READY => ThreadState::Ready,
                    BLOCKED => ThreadState::Blocked,
                    SUSPENDED => ThreadState::Suspended,
                    DELETED => ThreadState::Deleted,
                    _ => ThreadState::Invalid,
                },
                current_priority: t.uxCurrentPriority,
                base_priority: t.uxBasePriority,
                run_time_counter: t.ulRunTimeCounter,
                stack_high_water_mark: t.usStackHighWaterMark,
            }
        }).collect();

        SystemState {
            tasks,
            total_run_time
        }
        
    }
}