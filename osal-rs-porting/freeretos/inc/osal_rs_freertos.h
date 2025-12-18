#ifndef OSAL_RS_FREERTOS_H
#define OSAL_RS_FREERTOS_H

#include "FreeRTOS.h"
#include "semphr.h"
#include "portmacro.h"
#include "timers.h"
#include "task.h"

void osal_rs_critical_section_enter(void);

void osal_rs_critical_section_exit(void);

void osal_rs_port_yield_from_isr(BaseType_t pxHigherPriorityTaskWoken); 

void osal_rs_port_end_switching_isr( BaseType_t xSwitchRequired );

/* Timer wrappers for Rust FFI */
BaseType_t osal_rs_timer_start(TimerHandle_t xTimer, TickType_t xTicksToWait);
BaseType_t osal_rs_timer_stop(TimerHandle_t xTimer, TickType_t xTicksToWait);
BaseType_t osal_rs_timer_reset(TimerHandle_t xTimer, TickType_t xTicksToWait);
BaseType_t osal_rs_timer_change_period(TimerHandle_t xTimer, TickType_t xNewPeriod, TickType_t xTicksToWait);
BaseType_t osal_rs_timer_delete(TimerHandle_t xTimer, TickType_t xTicksToWait);

#endif /* OSAL_RS_FREERTOS_H */