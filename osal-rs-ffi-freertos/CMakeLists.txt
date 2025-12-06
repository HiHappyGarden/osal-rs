cmake_minimum_required(VERSION 3.16)

project(osal-rs-ffi-freertos C)

# Create the library
add_library(osal-rs-ffi-freertos STATIC
    src/osal_rs_ffi_freertos.c
)

# Add include directories
target_include_directories(osal-rs-ffi-freertos PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/inc
)

# Add FreeRTOS includes
target_include_directories(osal-rs-ffi-freertos PRIVATE
    ${CMAKE_SOURCE_DIR}/FreeRTOS-Kernel/include
    ${CMAKE_SOURCE_DIR}/FreeRTOS-Kernel/portable/ThirdParty/Community-Supported-Ports/GCC/RP2350_ARM_NTZ/non_secure
    ${CMAKE_SOURCE_DIR}/inc/hhg-config/pico
)

# Link against pico platform for basic includes (INTERFACE only, no source code)
target_link_libraries(osal-rs-ffi-freertos PRIVATE
    pico_platform
    hardware_sync
    pico_runtime_init
)
