
use core::ffi::{c_uint, c_void};
use crate::types::{BaseType, UBaseType};

use super::types::TickType;

pub type TaskHandle = *const c_void;
pub type QueueHandle = *const c_void;
pub type SemaphoreHandle = *const c_void;
pub type EventGroupHandle = *const c_void;
pub type TaskFunction = *const c_void;
pub type TimerHandle = *const c_void;
pub type TimerCallback = *const c_void;
pub type StackType = *const c_void;
pub type TaskState = c_uint;

pub const RUNNING: TaskState = 0;
pub const READY: TaskState = 1;
pub const BLOCKED: TaskState = 2;
pub const SUSPENDED: TaskState = 3;
pub const DELETED: TaskState = 4;
pub const INVALID: TaskState = 5;



unsafe extern "C" {


    /// Allocate memory from the  heap
    /// 
    /// # Arguments
    /// * `size` - The number of bytes to allocate
    /// 
    /// # Returns
    /// A pointer to the allocated memory, or null if allocation fails
    pub fn pvPortMalloc(size: usize) -> *mut c_void;

    /// Free memory previously allocated by pvPortMalloc
    /// 
    /// # Arguments
    /// * `pv` - Pointer to the memory to free
    pub fn vPortFree(pv: *mut c_void);

    pub fn vTaskDelay(xTicksToDelay: TickType);

    pub fn xTaskGetTickCount() -> TickType;

    pub fn vTaskStartScheduler();

    pub fn vTaskEndScheduler();

    pub fn vTaskSuspendAll();

    pub fn xTaskResumeAll() -> BaseType;

    pub fn xTaskGetCurrentTaskHandle() -> TaskHandle;

    pub fn eTaskGetState(xTask: TaskHandle) -> TaskState;

    pub fn uxTaskGetNumberOfTasks() -> UBaseType;
}

