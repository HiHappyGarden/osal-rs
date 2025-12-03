
mod free_rtos_allocator;


#[global_allocator]
static ALLOCATOR: FreeRtosAllocator = FreeRtosAllocator;