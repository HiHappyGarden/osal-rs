
use crate::SystemTrait;
use crate::freertos::ffi::{
    BLOCKED, DELETED, READY, RUNNING, SUSPENDED, eTaskGetState, uxTaskGetNumberOfTasks, vTaskEndScheduler, vTaskStartScheduler, vTaskSuspendAll, xTaskGetCurrentTaskHandle, xTaskGetTickCount, xTaskResumeAll
};
use crate::freertos::thread::ThreadState;
use crate::types::{BaseType, TickType};


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
}