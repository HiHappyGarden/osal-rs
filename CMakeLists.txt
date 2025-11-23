cmake_minimum_required(VERSION 3.15)
project(osal_rs_freertos C)

# Set build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Define FreeRTOS version
set(FREERTOS_VERSION "V11.2.0" CACHE STRING "FreeRTOS version to download")

# Define configurable parameters for FreeRTOS
set(FREERTOS_PORT "" CACHE STRING "FreeRTOS port to use (e.g., GCC/ARM_CM4F, ThirdParty/GCC/Posix)")
set(FREERTOS_HEAP "4" CACHE STRING "FreeRTOS heap implementation (1-5)")

# Display configuration summary
message(STATUS "FreeRTOS Configuration:")
message(STATUS "  Build Type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  FREERTOS_VERSION: ${FREERTOS_VERSION}")
message(STATUS "  FREERTOS_PORT: ${FREERTOS_PORT}")
message(STATUS "  FREERTOS_HEAP: ${FREERTOS_HEAP}")


# Include module to download FreeRTOS
include(cmake/FreeRTOS.cmake)

# Determine port based on system if not specified
if(NOT FREERTOS_PORT)
    if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
        set(FREERTOS_PORT "ThirdParty/GCC/Posix")
        set(FREERTOS_PORT_ASM "")
    else()
        message(FATAL_ERROR "FREERTOS_PORT not defined")
    endif()
endif()
message(STATUS "Using FreeRTOS port: ${FREERTOS_PORT}")

# Heap validation
if(NOT (FREERTOS_HEAP MATCHES "^[1-5]$"))
    message(FATAL_ERROR "FREERTOS_HEAP must be 1, 2, 3, 4, or 5. Got: ${FREERTOS_HEAP}")
endif()

message(STATUS "Using FreeRTOS heap implementation: heap_${FREERTOS_HEAP}.c")

# Prepare source files list
set(FREERTOS_SOURCES
    ${FREERTOS_SOURCE_DIR}/tasks.c
    ${FREERTOS_SOURCE_DIR}/queue.c
    ${FREERTOS_SOURCE_DIR}/list.c
    ${FREERTOS_SOURCE_DIR}/timers.c
    ${FREERTOS_SOURCE_DIR}/event_groups.c
    ${FREERTOS_SOURCE_DIR}/stream_buffer.c
    ${FREERTOS_SOURCE_DIR}/portable/MemMang/heap_${FREERTOS_HEAP}.c
    ${FREERTOS_SOURCE_DIR}/portable/${FREERTOS_PORT}/port.c
    ${FREERTOS_PORT_ASM}
)

# Add event_posix.c if using POSIX port
if(FREERTOS_PORT MATCHES "Posix")
    list(APPEND FREERTOS_SOURCES freertos_posix/event_posix.c)
    message(STATUS "Adding event_posix.c for POSIX port support")
endif()

# Create FreeRTOS library
add_library(freertos STATIC ${FREERTOS_SOURCES})

# Add include directories
target_include_directories(freertos PUBLIC
    ${FREERTOS_SOURCE_DIR}/include
    ${FREERTOS_SOURCE_DIR}/portable/${FREERTOS_PORT}
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Compilation options
target_compile_options(freertos PRIVATE
    -Wall
    -Wextra
    -ffunction-sections
    -fdata-sections
)

# Link pthread if using POSIX port
if(FREERTOS_PORT MATCHES "Posix")
    target_link_libraries(freertos PUBLIC pthread)
    message(STATUS "Linking pthread for POSIX port")
endif()

# Install library
install(TARGETS freertos
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)

# Install headers
install(DIRECTORY ${FREERTOS_SOURCE_DIR}/include/
    DESTINATION include/freertos
    FILES_MATCHING PATTERN "*.h"
)

# Install portable headers
install(DIRECTORY ${FREERTOS_SOURCE_DIR}/portable/${FREERTOS_PORT}/
    DESTINATION include/freertos/portable
    FILES_MATCHING PATTERN "*.h"
)
