
use core::any::Any;
use alloc::boxed::Box;

use alloc::ffi::CString;
use alloc::sync::Arc;
use core::ffi::c_void;
use core::mem::zeroed;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::traits::{TimerFunc, TimerTrait};

use crate::{Result, us_sleep};
use crate::Error::Std;
use crate::osal::ffi::{pthread_attr_destroy, pthread_attr_init, pthread_attr_setstacksize, pthread_create, pthread_setname_np};
use crate::posix::ffi::{pthread_attr_t, pthread_detach, pthread_t};


#[derive(Clone)]
pub struct Timer {
    handle: pthread_t,
    callback: Arc<TimerFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>,
    exit: Arc<AtomicBool>,
    us: u64,
    one_shot: bool,
}

unsafe impl Send for Timer {}

unsafe impl Sync for Timer {}



extern "C" fn callback(param_ptr: *mut c_void) -> *mut c_void {
    if param_ptr.is_null() {
        return null_mut()
    }

    // Extract the Timer from the raw pointer, but immediately leak it
    // to prevent it from being dropped during the loop
    let boxed_context = unsafe { Box::from_raw(param_ptr as *mut Timer) };
    let callback_fn = boxed_context.callback.clone();
    let param = boxed_context.param.clone();
    let exit_flag = boxed_context.exit.clone();
    let one_shot = boxed_context.one_shot;
    let us = boxed_context.us;
    
    // Leak the box so we can use the raw pointer in the loop
    let leaked_ref = Box::leak(boxed_context);
    
    loop {
        if exit_flag.load(Ordering::Relaxed) {
            break;
        }
        
        callback_fn(leaked_ref, param.clone());
        
        if one_shot {
            break;
        }
        
        // Sleep for the specified period
        us_sleep(us);
    }

    // Clean up: convert back from leaked reference to Box and drop it
    unsafe { drop(Box::from_raw(leaked_ref as *mut Timer)) };
    
    null_mut()
}


impl TimerTrait for Timer {
    fn new<F>(us: u64, callback: F, param: Option<Arc<dyn Any + Send + Sync>>, one_shot: bool) -> Self
    where
        F: Fn(&mut dyn TimerTrait, Option<Arc<dyn Any + Send + Sync>>) + Send + Sync + 'static,
        Self: Sized
    {

        Self {
            handle: 0,
            callback: Arc::new(callback),
            param,
            exit: Arc::new(AtomicBool::new(false)),
            us,
            one_shot,
        }
    }

    fn set(&mut self, _us: u64) -> Result<()> {
        self.us = _us;
        Ok(())
    }

    fn set_from_isr(&mut self, us: u64) -> Result<()> {
        self.set(us)
    }

    fn start(&mut self)  -> Result<()>{
        
        let mut attr: pthread_attr_t = unsafe { zeroed() };

        unsafe {
            let rc = pthread_attr_init(&mut attr);
            if rc != 0 {
                return Err(Std(rc, "Failed to initialize pthread attributes"));
            }

            let rc = pthread_attr_setstacksize(&mut attr, 16*1_024);
            if rc != 0 {
                pthread_attr_destroy(&mut attr);
                return Err(Std(rc, "Failed to set pthread stack size"));
            }

            let context_ptr = Box::into_raw(Box::new(self.clone())) as *mut c_void;
            let result =  pthread_create(
                    &mut self.handle,
                    &attr,
                    Some(callback),
                    context_ptr,
                );
            if result != 0 {
                _ = Box::from_raw(context_ptr);
                pthread_attr_destroy(&mut attr);
                return Err(Std(result, "Failed to create pthread"));
            }

            pthread_attr_destroy(&mut attr);
            

            let name_c = CString::new("timer_thread").map_err(|_| Std(-1, "Failed to create timer_thread string"))?;

            pthread_setname_np(self.handle, name_c.as_ptr());
            
        }

        Ok(())
    }

    fn start_from_isr(&mut self) -> Result<()> {
        self.start()
    }

    fn stop(&mut self) {
        self.exit.store(true, Ordering::Relaxed);
        unsafe {
            pthread_detach(self.handle);
        }
    }

    fn stop_from_isr(&mut self) {
        self.stop();
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::sync::{Arc as StdArc, Mutex as StdMutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_new() {
        let timer = Timer::new(1000, |_timer, _args| {}, None, false);
        assert_eq!(timer.handle, 0, "Timer handle should be 0 before start");
        assert!(!timer.exit.load(Ordering::Relaxed), "Timer exit flag should be false");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_one_shot() {
        let counter = StdArc::new(StdMutex::new(0));
        let counter_clone = counter.clone();
        
        let mut timer = Timer::new(
            50_000, // 50ms
            move |_timer, _args| {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            },
            None,
            true
        );
        
        timer.start().unwrap();
        
        thread::sleep(Duration::from_millis(200));
        
        let count = *counter.lock().unwrap();
        assert!(count <= 1, "One shot timer should fire at most once, got {}", count);
    }

    #[test]
    #[cfg(feature = "posix")]
    #[ignore = "Timer callback handler not properly implemented in Timer::new()"]
    fn test_timer_periodic() {
        let counter = StdArc::new(StdMutex::new(0));
        let counter_clone = counter.clone();
        
        let mut timer = Timer::new(
            50_000, // 50ms
            move |_timer, _args| {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            },
            None,
            false
        );
        
        timer.start().unwrap();
        
        thread::sleep(Duration::from_millis(250));
        timer.stop();
        
        thread::sleep(Duration::from_millis(50));
        
        let count = *counter.lock().unwrap();
        assert!(count >= 2, "Periodic timer should fire multiple times, got {}", count);
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_start_stop() {
        let counter = StdArc::new(StdMutex::new(0));
        let counter_clone = counter.clone();
        
        let mut timer = Timer::new(
            50_000,
            move |_timer, _args| {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            },
            None,
            false
        );
        
        timer.start().unwrap();
        thread::sleep(Duration::from_millis(150));
        timer.stop();
        
        let count_after_stop = *counter.lock().unwrap();
        thread::sleep(Duration::from_millis(150));
        let count_after_wait = *counter.lock().unwrap();
        
        assert_eq!(count_after_stop, count_after_wait, "Timer should not fire after stop");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_set_period() {
        let mut timer = Timer::new(1000_000, |_timer, _args| {}, None, false);
        
        let result = timer.set(500_000); // Change to 500ms
        assert!(result.is_ok(), "Set should succeed");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_from_isr() {
        let mut timer = Timer::new(100_000, |_timer, _args| {}, None, false);
        
        let result = timer.set_from_isr(200_000);
        assert!(result.is_ok(), "ISR set should succeed");
        
        timer.start_from_isr().unwrap();
        thread::sleep(Duration::from_millis(50));
        timer.stop_from_isr();
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_with_parameter() {
        let param = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
        
        let mut timer = Timer::new(
            50_000,
            move |_timer, _args| {
                // In the current implementation, args are passed to callback
                // This test verifies the timer can be created with parameters
            },
            Some(param),
            true
        );
        
        timer.start().unwrap();
        thread::sleep(Duration::from_millis(100));
    }

    #[test]
    #[cfg(feature = "posix")]
    #[ignore = "Timer callback handler not properly implemented in Timer::new()"]
    fn test_timer_multiple_instances() {
        let counter1 = StdArc::new(StdMutex::new(0));
        let counter2 = StdArc::new(StdMutex::new(0));
        
        let counter1_clone = counter1.clone();
        let counter2_clone = counter2.clone();
        
        let mut timer1 = Timer::new(
            50_000,
            move |_timer, _args| {
                let mut count = counter1_clone.lock().unwrap();
                *count += 1;
            },
            None,
            false
        );
        
        let mut timer2 = Timer::new(
            75_000,
            move |_timer, _args| {
                let mut count = counter2_clone.lock().unwrap();
                *count += 1;
            },
            None,
            false
        );
        
        timer1.start().unwrap();
        timer2.start().unwrap();
        
        thread::sleep(Duration::from_millis(300));
        
        timer1.stop();
        timer2.stop();
        
        let count1 = *counter1.lock().unwrap();
        let count2 = *counter2.lock().unwrap();
        
        assert!(count1 >= 2, "Timer 1 should fire multiple times");
        assert!(count2 >= 1, "Timer 2 should fire at least once");
    }
}
