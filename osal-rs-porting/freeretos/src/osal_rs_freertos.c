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
