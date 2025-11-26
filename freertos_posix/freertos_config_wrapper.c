#include "FreeRTOS.h"

// Expose FreeRTOSConfig.h constants that bindgen cannot parse as functions
unsigned long get_freertos_cpu_clock_hz(void) {
    return configCPU_CLOCK_HZ;
}

unsigned long get_freertos_tick_rate_hz(void) {
    return configTICK_RATE_HZ;
}

unsigned long get_freertos_minimal_stack_size(void) {
    return configMINIMAL_STACK_SIZE;
}

unsigned long get_freertos_total_heap_size(void) {
    return configTOTAL_HEAP_SIZE;
}

unsigned long get_freertos_timer_task_stack_depth(void) {
    return configTIMER_TASK_STACK_DEPTH;
}

TickType_t port_tick_period_ms() {
    return (TickType_t) 1000 / configTICK_RATE_HZ;
}
