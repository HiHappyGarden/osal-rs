mod posix_allocator;

#[global_allocator]
static ALLOCATOR: PosixAllocator = PosixAllocator;