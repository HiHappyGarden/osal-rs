#ifndef OSAL_RS_FREERTOS_H
#define OSAL_RS_FREERTOS_H

#include "FreeRTOS.h"
#include "semphr.h"
#include "portmacro.h"

void osal_rs_critical_section_enter(void);

void osal_rs_critical_section_exit(void);

void osal_rs_port_yield_from_isr(BaseType_t pxHigherPriorityTaskWoken); 

void osal_rs_port_end_switching_isr( BaseType_t xSwitchRequired );

#endif /* OSAL_RS_FREERTOS_H */