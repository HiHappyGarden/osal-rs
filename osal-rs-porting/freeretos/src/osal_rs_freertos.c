#include "osal_rs_freertos.h"


void osal_rs_critical_section_enter(void)
{
    taskENTER_CRITICAL();
}


void osal_rs_critical_section_exit(void)
{
    taskEXIT_CRITICAL();
}

void osal_rs_port_yield_from_isr(BaseType_t pxHigherPriorityTaskWoken)
{
    portYIELD_FROM_ISR(pxHigherPriorityTaskWoken);
}

void osal_rs_port_end_switching_isr( BaseType_t xSwitchRequired )
{
    portEND_SWITCHING_ISR( xSwitchRequired );
}

/* Timer wrappers for Rust FFI - these wrap FreeRTOS macros */
BaseType_t osal_rs_timer_start(TimerHandle_t xTimer, TickType_t xTicksToWait)
{
    return xTimerGenericCommand(xTimer, tmrCOMMAND_START, xTaskGetTickCount(), NULL, xTicksToWait);
}

BaseType_t osal_rs_timer_stop(TimerHandle_t xTimer, TickType_t xTicksToWait)
{
    return xTimerGenericCommand(xTimer, tmrCOMMAND_STOP, 0U, NULL, xTicksToWait);
}

BaseType_t osal_rs_timer_reset(TimerHandle_t xTimer, TickType_t xTicksToWait)
{
    return xTimerGenericCommand(xTimer, tmrCOMMAND_RESET, xTaskGetTickCount(), NULL, xTicksToWait);
}

BaseType_t osal_rs_timer_change_period(TimerHandle_t xTimer, TickType_t xNewPeriod, TickType_t xTicksToWait)
{
    return xTimerGenericCommand(xTimer, tmrCOMMAND_CHANGE_PERIOD, xNewPeriod, NULL, xTicksToWait);
}

BaseType_t osal_rs_timer_delete(TimerHandle_t xTimer, TickType_t xTicksToWait)
{
    return xTimerGenericCommand(xTimer, tmrCOMMAND_DELETE, 0U, NULL, xTicksToWait);
}
