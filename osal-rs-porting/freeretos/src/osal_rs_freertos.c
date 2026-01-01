/***************************************************************************
 *
 * osal-rs
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

#include "osal_rs_freertos.h"

#include <stdarg.h>
#include <stdio.h>

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

int printf_on_uart(const char *format, ...)
{
    va_list args;
    va_start(args, format);
    int ret = vprintf(format, args);
    va_end(args);
    return ret;
}
