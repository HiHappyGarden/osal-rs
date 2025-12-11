use core::ffi::{c_char, c_uint, c_void};
use core::ptr;
use crate::freertos::types::{BaseType, StackType, UBaseType, TickType, EventBits};

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
#[allow(non_upper_case_globals)]
pub const tskDEFAULT_INDEX_TO_NOTIFY: UBaseType = 0;

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

    pub fn xTaskDelayUntil(
        pxPreviousWakeTime: *mut TickType,
        xTimeIncrement: TickType,
    ) -> BaseType;


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

    pub fn ulTaskGenericNotifyTake(uxIndexToWaitOn: UBaseType, xClearCountOnExit: BaseType, xTicksToWait: TickType) -> u32;


    pub fn xTaskGenericNotifyWait(
        uxIndexToWaitOn: UBaseType,
        ulBitsToClearOnEntry: u32,
        ulBitsToClearOnExit: u32,
        pulNotificationValue: *mut u32,
        xTicksToWait: TickType,
    ) -> BaseType;


    pub fn xTaskGenericNotify(
        xTaskToNotify: ThreadHandle,
        uxIndexToNotify: UBaseType,
        ulValue: u32,
        eAction: u32,
        pulPreviousNotificationValue: *mut u32,
    ) -> BaseType;


    pub fn xTaskGenericNotifyFromISR(
        xTaskToNotify: ThreadHandle,
        uxIndexToNotify: UBaseType,
        ulValue: u32,
        eAction: u32,
        pulPreviousNotificationValue: *mut u32,
        pxHigherPriorityTaskWoken: *mut BaseType,
    ) -> BaseType;
    
    pub fn xEventGroupWaitBits(
        xEventGroup: EventGroupHandle,
        uxBitsToWaitFor: EventBits,
        xClearOnExit: BaseType,
        xWaitForAllBits: BaseType,
        xTicksToWait: TickType,
    ) -> EventBits;

    pub fn xEventGroupClearBits(
        xEventGroup: EventGroupHandle,
        uxBitsToClear: EventBits,
    ) -> EventBits;

    pub fn xEventGroupClearBitsFromISR(
        xEventGroup: EventGroupHandle,
        uxBitsToClear: EventBits,
    ) -> BaseType;

        pub fn xEventGroupSetBits(
        xEventGroup: EventGroupHandle,
        uxBitsToSet: EventBits,
    ) -> EventBits;


    pub fn xEventGroupSetBitsFromISR(
        xEventGroup: EventGroupHandle,
        uxBitsToSet: EventBits,
        pxHigherPriorityTaskWoken: *mut BaseType,
    ) -> BaseType;

    pub fn xEventGroupGetBitsFromISR(xEventGroup: EventGroupHandle) -> EventBits;

    pub fn xEventGroupCreate() -> EventGroupHandle;
}

#[macro_export]
macro_rules! ulTaskNotifyTake {
    ($xClearCountOnExit:expr, $xTicksToWait:expr) => {
        unsafe {
            $crate::freertos::ffi::ulTaskGenericNotifyTake(
                $crate::freertos::ffi::tskDEFAULT_INDEX_TO_NOTIFY,
                $xClearCountOnExit,
                $xTicksToWait
            )
        }
    };
}

#[macro_export]
macro_rules! xTaskNotifyWait {
    ($ulBitsToClearOnEntry:expr, $ulBitsToClearOnExit:expr, $pulNotificationValue:expr, $xTicksToWait:expr) => {
        unsafe {
            $crate::freertos::ffi::xTaskGenericNotifyWait(
                $crate::freertos::ffi::tskDEFAULT_INDEX_TO_NOTIFY,
                $ulBitsToClearOnEntry,
                $ulBitsToClearOnExit,
                $pulNotificationValue,
                $xTicksToWait
            )
        }
    };
}

#[macro_export]
macro_rules! xTaskNotify {
    ($xTaskToNotify:expr, $ulValue:expr, $eAction:expr) => {
        unsafe {
            $crate::freertos::ffi::xTaskGenericNotify(
                $xTaskToNotify,
                $crate::freertos::ffi::tskDEFAULT_INDEX_TO_NOTIFY,
                $ulValue,
                $eAction,
                core::ptr::null_mut()
            )
        }
    };
}

#[macro_export]
macro_rules! xTaskNotifyFromISR {
    ($xTaskToNotify:expr, $ulValue:expr, $eAction:expr, $pxHigherPriorityTaskWoken:expr) => {
        unsafe {
            $crate::freertos::ffi::xTaskGenericNotifyFromISR(
                $xTaskToNotify,
                $crate::freertos::ffi::tskDEFAULT_INDEX_TO_NOTIFY,
                $ulValue,
                $eAction,
                core::ptr::null_mut(),
                $pxHigherPriorityTaskWoken
            )
        }
    };
}

#[macro_export]
macro_rules! xTaskNotifyAndQuery {
    ($xTaskToNotify:expr, $ulValue:expr, $eAction:expr, $pulPreviousNotificationValue:expr) => {
        unsafe {
            $crate::freertos::ffi::xTaskGenericNotify(
                $xTaskToNotify,
                $crate::freertos::ffi::tskDEFAULT_INDEX_TO_NOTIFY,
                $ulValue,
                $eAction,
                $pulPreviousNotificationValue
            )
        }
    };
}

#[macro_export]
macro_rules! vTaskDelayUntil {
    ($pxPreviousWakeTime:expr, $xTimeIncrement:expr) => {
        unsafe {
            $crate::freertos::ffi::xTaskDelayUntil(
                $pxPreviousWakeTime,
                $xTimeIncrement
            );
        }
    };
}

#[macro_export]
macro_rules! xEventGroupGetBits {
    ($xEventGroup:expr) => {
        unsafe {
            $crate::freertos::ffi::xEventGroupClearBits($xEventGroup, 0)
        }
    };
}