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

int printf_on_uart(const char *format, ...);

#endif /* OSAL_RS_FREERTOS_H */