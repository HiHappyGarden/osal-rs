# FreeRTOS.cmake - Download and configure FreeRTOS kernel from GitHub

include(FetchContent)

# Set FreeRTOS version (v11.2.0)
set(FREERTOS_VERSION "V11.2.0" CACHE STRING "FreeRTOS version")

message(STATUS "Downloading FreeRTOS kernel version ${FREERTOS_VERSION} from GitHub...")

# Download FreeRTOS kernel from GitHub
FetchContent_Declare(
    freertos_kernel
    GIT_REPOSITORY https://github.com/FreeRTOS/FreeRTOS-Kernel.git
    GIT_TAG        ${FREERTOS_VERSION}
    GIT_SHALLOW    TRUE
    GIT_PROGRESS   TRUE
)

# Make FreeRTOS available
FetchContent_GetProperties(freertos_kernel)
if(NOT freertos_kernel_POPULATED)
    message(STATUS "Populating FreeRTOS kernel...")
    FetchContent_Populate(freertos_kernel)
    set(FREERTOS_SOURCE_DIR ${freertos_kernel_SOURCE_DIR})
    message(STATUS "FreeRTOS kernel downloaded to: ${FREERTOS_SOURCE_DIR}")
endif()

# Export FreeRTOS source directory
set(FREERTOS_SOURCE_DIR ${freertos_kernel_SOURCE_DIR} PARENT_SCOPE)
