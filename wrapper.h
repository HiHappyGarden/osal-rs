// wrapper.h - FreeRTOS header wrapper for bindgen
// This file includes all FreeRTOS headers needed for Rust bindings

#include "FreeRTOS.h"
#include "task.h"
#include "queue.h"
#include "semphr.h"
#include "timers.h"
#include "event_groups.h"
#include "stream_buffer.h"
#include "message_buffer.h"

