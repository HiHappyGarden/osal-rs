#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_uint, c_void};
use core::ptr;

use super::types::{BaseType, StackType, UBaseType, TickType, EventBits};

pub type ThreadHandle = *const c_void;
pub type QueueHandle = *const c_void;
pub type SemaphoreHandle = *const c_void;
pub type EventGroupHandle = *const c_void;
pub type TimerHandle = *const c_void;
pub type MutexHandle = *const c_void;
pub type TimerCallback = unsafe extern "C" fn(timer: TimerHandle);
pub type TaskState = c_uint;

pub const RUNNING: TaskState = 0;
pub const READY: TaskState = 1;
pub const BLOCKED: TaskState = 2;
pub const SUSPENDED: TaskState = 3;
pub const DELETED: TaskState = 4;
pub const INVALID: TaskState = 5;


pub const pdFALSE: BaseType = 0;

pub const pdTRUE: BaseType = 1;

pub const pdPASS: BaseType = pdTRUE;

pub const pdFAIL: BaseType = pdFALSE;

pub const tskDEFAULT_INDEX_TO_NOTIFY: UBaseType = 0;

pub const semBINARY_SEMAPHORE_QUEUE_LENGTH: u8 = 1;

pub const semSEMAPHORE_QUEUE_ITEM_LENGTH: u8 = 0;

pub const semGIVE_BLOCK_TIME: TickType = 0;

pub const queueSEND_TO_BACK: BaseType = 0;

pub const queueSEND_TO_FRONT: BaseType = 1;

pub const queueOVERWRITE: BaseType = 2;

pub const queueQUEUE_TYPE_BASE: u8 = 0;

pub const queueQUEUE_TYPE_MUTEX: u8 = 1;

pub const queueQUEUE_TYPE_COUNTING_SEMAPHORE: u8 = 2;

pub const queueQUEUE_TYPE_BINARY_SEMAPHORE: u8 = 3;

pub const queueQUEUE_TYPE_RECURSIVE_MUTEX: u8 = 4;

pub const queueQUEUE_TYPE_SET: u8 = 5;



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

    pub fn vEventGroupDelete(xEventGroup: EventGroupHandle);

    pub fn xEventGroupCreate() -> EventGroupHandle;

    pub fn osal_rs_critical_section_enter();

    pub fn osal_rs_critical_section_exit();

    pub fn osal_rs_port_yield_from_isr(pxHigherPriorityTaskWoken: BaseType);

    pub fn osal_rs_port_end_switching_isr( xSwitchRequired: BaseType );

    pub fn xQueueCreateMutex(ucQueueType: u8) -> QueueHandle;
    
    pub fn xQueueCreateCountingSemaphore(
        uxMaxCount: UBaseType,
        uxInitialCount: UBaseType,
    ) -> QueueHandle;

    pub fn xQueueSemaphoreTake(xQueue: QueueHandle, xTicksToWait: TickType) -> BaseType;

    pub fn xQueueReceiveFromISR(
        xQueue: QueueHandle,
        pvBuffer: *mut c_void,
        pxHigherPriorityTaskWoken: *mut BaseType,
    ) -> BaseType;

    pub fn xQueueGenericSend(
        xQueue: QueueHandle,
        pvItemToQueue: *const c_void,
        xTicksToWait: TickType,
        xCopyPosition: BaseType,
    ) -> BaseType;

    pub fn xQueueGiveFromISR(
        xQueue: QueueHandle,
        pxHigherPriorityTaskWoken: *mut BaseType,
    ) -> BaseType;

     pub fn vQueueDelete(xQueue: QueueHandle);

    pub fn xQueueGenericCreate(
        uxQueueLength: UBaseType,
        uxItemSize: UBaseType,
        ucQueueType: u8,
    ) -> QueueHandle;

    pub fn xQueueReceive(
        xQueue: QueueHandle,
        pvBuffer: *mut c_void,
        xTicksToWait: TickType,
    ) -> BaseType;

    pub fn xQueueGenericSendFromISR(
        xQueue: QueueHandle,
        pvItemToQueue: *const c_void,
        pxHigherPriorityTaskWoken: *mut BaseType,
        xCopyPosition: BaseType,
    ) -> BaseType;

    pub fn xQueueTakeMutexRecursive(xMutex: QueueHandle, xTicksToWait: TickType) -> BaseType;

    pub fn xQueueGiveMutexRecursive(xMutex: QueueHandle) -> BaseType;

    pub fn xPortGetFreeHeapSize() -> usize;

    pub fn xTimerCreateTimerTask() -> BaseType;

    pub fn xTimerCreate(
        pcTimerName: *const c_char,
        xTimerPeriodInTicks: TickType,
        xAutoReload: BaseType,
        pvTimerID: *mut c_void,
        pxCallbackFunction: Option<TimerCallback>,
    ) -> TimerHandle;

    pub fn osal_rs_timer_start(xTimer: TimerHandle, xTicksToWait: TickType) -> BaseType;

    pub fn osal_rs_timer_stop(xTimer: TimerHandle, xTicksToWait: TickType) -> BaseType;

    pub fn osal_rs_timer_reset(xTimer: TimerHandle, xTicksToWait: TickType) -> BaseType;

    pub fn osal_rs_timer_change_period(
        xTimer: TimerHandle,
        xNewPeriodInTicks: TickType,
        xTicksToWait: TickType,
    ) -> BaseType;

    pub fn osal_rs_timer_delete(xTimer: TimerHandle, xTicksToWait: TickType) -> BaseType;

    pub fn pvTimerGetTimerID(xTimer: TimerHandle) -> *mut c_void;

    pub fn printf(fmt: *const u8, ...) -> i32; 
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

#[macro_export]
macro_rules! xSemaphoreCreateCounting {
    ($uxMaxCount:expr, $uxInitialCount:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueCreateCountingSemaphore(
                $uxMaxCount,
                $uxInitialCount
            )
        }
    };
}

#[macro_export]
macro_rules! xSemaphoreTake {
    ($xSemaphore:expr, $xBlockTime:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueSemaphoreTake(
                $xSemaphore,
                $xBlockTime
            )
        }
    };
}

#[macro_export]
macro_rules! xSemaphoreTakeFromISR {
    ($xSemaphore:expr, $pxHigherPriorityTaskWoken:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueReceiveFromISR(
                $xSemaphore,
                core::ptr::null_mut(),
                $pxHigherPriorityTaskWoken
            )
        }
    };
}

#[macro_export]
macro_rules! xSemaphoreGive {
    ($xSemaphore:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueGenericSend(
                $xSemaphore,
                core::ptr::null(),
                $crate::freertos::ffi::semGIVE_BLOCK_TIME,
                $crate::freertos::ffi::queueSEND_TO_BACK
            )
        }
    };
}

#[macro_export]
macro_rules! xSemaphoreGiveFromISR {
    ($xSemaphore:expr, $pxHigherPriorityTaskWoken:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueGiveFromISR(
                $xSemaphore,
                $pxHigherPriorityTaskWoken
            )
        }
    };
}

#[macro_export]
macro_rules! vSemaphoreDelete {
    ($xSemaphore:expr) => {
        unsafe {
            $crate::freertos::ffi::vQueueDelete($xSemaphore)
        }
    };
}

#[macro_export]
macro_rules! xQueueCreate {
    ($uxQueueLength:expr, $uxItemSize:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueGenericCreate(
                $uxQueueLength,
                $uxItemSize,
                $crate::freertos::ffi::queueQUEUE_TYPE_BASE
            )
        }
    };
}

#[macro_export]
macro_rules! xQueueSendToBackFromISR {
    ($xQueue:expr, $pvItemToQueue:expr, $pxHigherPriorityTaskWoken:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueGenericSendFromISR(
                $xQueue,
                $pvItemToQueue,
                $pxHigherPriorityTaskWoken,
                $crate::freertos::ffi::queueSEND_TO_BACK
            )
        }
    };
}

#[macro_export]
macro_rules! xQueueSendToBack {
    ($xQueue:expr, $pvItemToQueue:expr, $xTicksToWait:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueGenericSend(
                $xQueue,
                $pvItemToQueue,
                $xTicksToWait,
                $crate::freertos::ffi::queueSEND_TO_BACK
            )
        }
    };
}

#[macro_export]
macro_rules! xSemaphoreCreateRecursiveMutex {
    () => {
        unsafe {
            $crate::freertos::ffi::xQueueCreateMutex(
                $crate::freertos::ffi::queueQUEUE_TYPE_RECURSIVE_MUTEX
            )
        }
    };
}

#[macro_export]
macro_rules! xSemaphoreTakeRecursive {
    ($xMutex:expr, $xBlockTime:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueTakeMutexRecursive(
                $xMutex,
                $xBlockTime
            )
        }
    };
}

#[macro_export]
macro_rules! xSemaphoreGiveRecursive {
    ($xMutex:expr) => {
        unsafe {
            $crate::freertos::ffi::xQueueGiveMutexRecursive($xMutex)
        }
    };
}
