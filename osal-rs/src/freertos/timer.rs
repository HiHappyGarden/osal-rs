use super::ffi::TimerHandle;



pub struct Timer (TimerHandle);

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl Drop for Timer {
    fn drop(&mut self) {
        // Add any necessary cleanup code here
    }
}


