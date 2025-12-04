
use core::ffi::c_void;

unsafe extern "C" {
    /// Allocate memory from the FreeRTOS heap
    /// 
    /// # Arguments
    /// * `size` - The number of bytes to allocate
    /// 
    /// # Returns
    /// A pointer to the allocated memory, or null if allocation fails
    pub fn pvPortMalloc(size: usize) -> *mut c_void;

    /// Free memory previously allocated by pvPortMalloc
    /// 
    /// # Arguments
    /// * `pv` - Pointer to the memory to free
    pub fn vPortFree(pv: *mut c_void);
}
