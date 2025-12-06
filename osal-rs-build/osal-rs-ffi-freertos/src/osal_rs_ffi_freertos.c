#include "osal_rs_ffi_freertos.h"

uint16_t osal_rs_ffi_freertos_get_type_size(osal_rs_ffi_freertos_types_t task_handle)
{
    switch (task_handle) {
        case OSAL_TickType:
            return sizeof(TickType_t);
        case OSAL_UBaseType:
            return sizeof(UBaseType_t);
        case OSAL_BaseType:
            return sizeof(BaseType_t);
        default:
            return 0; // Unknown type
    }
}


uint64_t osal_rs_ffi_freertos_get_config_value(osal_rs_ffi_freertos_config_t type)
{
    switch (type) {
        case OSAL_CPU_CLOCK_HZ:
            return configCPU_CLOCK_HZ;
        case OSAL_TICK_RATE_HZ:
            return configTICK_RATE_HZ;
        case OSAL_MAX_PRIORITIES:
            return configMAX_PRIORITIES;
        case OSAL_MINIMAL_STACK_SIZE:
            return configMINIMAL_STACK_SIZE;
        default:
            return 0; // Unknown configuration
    }
}