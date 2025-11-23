#include "FreeRTOS.h"

// Expose FreeRTOSConfig.h constants that bindgen cannot parse
const unsigned long FREERTOS_CPU_CLOCK_HZ = configCPU_CLOCK_HZ;
const unsigned long FREERTOS_TICK_RATE_HZ = configTICK_RATE_HZ;
const unsigned long FREERTOS_MINIMAL_STACK_SIZE = configMINIMAL_STACK_SIZE;
const unsigned long FREERTOS_TOTAL_HEAP_SIZE = configTOTAL_HEAP_SIZE;
const unsigned long FREERTOS_TIMER_TASK_STACK_DEPTH = configTIMER_TASK_STACK_DEPTH;
