
#ifndef OSAL_RS_FFI_FREEERTOS_H
#define OSAL_RS_FFI_FREEERTOS_H

#include "FreeRTOS.h"
#include "task.h"

#include <stdint.h>


#ifdef __cplusplus
extern "C" {
#endif

typedef enum  {
    OSAL_TickType,
    OSAL_UBaseType,
    OSAL_BaseType,
} osal_rs_ffi_freertos_types_t;

typedef enum {
    OSAL_CPU_CLOCK_HZ,
    OSAL_TICK_RATE_HZ,
    OSAL_MAX_PRIORITIES,
    OSAL_MINIMAL_STACK_SIZE
} osal_rs_ffi_freertos_config_t;

uint16_t osal_rs_ffi_freertos_get_type_size(osal_rs_ffi_freertos_types_t task_handle);
uint64_t osal_rs_ffi_freertos_get_config_value(osal_rs_ffi_freertos_config_t type);


#ifdef __cplusplus
}
#endif

#endif /* OSAL_RS_FFI_FREEERTOS_H */
