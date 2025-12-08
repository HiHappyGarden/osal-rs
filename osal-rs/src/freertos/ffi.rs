use core::ffi::{c_char, c_uint, c_void};
use core::ptr;
use crate::freertos::types::{BaseType, StackType, UBaseType, TickType};

pub type ThreadHandle = *const c_void;
pub type QueueHandle = *const c_void;
pub type SemaphoreHandle = *const c_void;
pub type EventGroupHandle = *const c_void;
pub type TimerHandle = *const c_void;
pub type TimerCallback = *const c_void;
pub type TaskState = c_uint;

pub const RUNNING: TaskState = 0;
pub const READY: TaskState = 1;
pub const BLOCKED: TaskState = 2;
pub const SUSPENDED: TaskState = 3;
pub const DELETED: TaskState = 4;
pub const INVALID: TaskState = 5;

#[allow(non_upper_case_globals)]
pub const pdPASS: BaseType = 0;
#[allow(non_upper_case_globals)]
pub const pdTRUE: BaseType = 1;
#[allow(non_upper_case_globals)]
pub const pdFALSE: BaseType = 1;

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TaskStatus {
    pub xHandle: ThreadHandle,
    pub pcTaskName: *const c_char,
    pub xTaskNumber: UBaseType,
    pub eCurrentState: TaskState,
    pub uxCurrentPriority: UBaseType,
    pub uxBasePriority: UBaseType,
    pub ulRunTimeCounter: u32,
    pub pxStackBase: *mut StackType,
    pub usStackHighWaterMark: StackType
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus {
            xHandle: ptr::null(),
            pcTaskName: ptr::null(),
            xTaskNumber: 0,
            eCurrentState: INVALID,
            uxCurrentPriority: 0,
            uxBasePriority: 0,
            ulRunTimeCounter: 0,
            pxStackBase: ptr::null_mut(),
            usStackHighWaterMark: 0,
        }
    }
}

pub type TaskFunction = Option<unsafe extern "C" fn(arg: *mut c_void)>;

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

    pub fn xTaskGetCurrentTaskHandle() -> ThreadHandle;

    pub fn eTaskGetState(xTask: ThreadHandle) -> TaskState;

    pub fn uxTaskGetNumberOfTasks() -> UBaseType;

    pub fn uxTaskGetSystemState(
        pxTaskStatusArray: *mut TaskStatus,
        uxArraySize: UBaseType,
        pulTotalRunTime: *mut u32,
    ) -> UBaseType;

    pub fn xTaskCreate(
        pxTaskCode: TaskFunction,
        pcName: *const c_char,
        uxStackDepth: StackType,
        pvParameters: *mut c_void,
        uxPriority: UBaseType,
        pxCreatedTask: *mut ThreadHandle,
    ) -> BaseType;

    pub fn vTaskDelete(xTaskToDelete: ThreadHandle);

    pub fn vTaskSuspend(xTaskToSuspend: ThreadHandle);

    pub fn vTaskResume(xTaskToResume: ThreadHandle);

    pub fn vTaskGetInfo(
        xTask: ThreadHandle,
        pxTaskStatus: *mut TaskStatus,
        xGetFreeStackSpace: BaseType,
        eState: TaskState,
    );
}

