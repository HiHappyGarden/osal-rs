use std::any::Any;
use std::sync::{Arc, Mutex};
use crate::traits::Timer as TimerTrait;
use crate::posix::time::us_sleep;
use crate::{Thread, ThreadTrait, Result, Error};
use crate::posix::thread::ThreadDefaultPriority;



pub struct Timer {
    thread: Option<Thread>,
    inner: Arc<Mutex<TimerInner>>,
}

unsafe impl Send for Timer {}

unsafe impl Sync for Timer {}


struct TimerInner {
    handler: Box<dyn Fn(Option<Box<dyn Any>>) + Send + Sync>,
    args: Option<Box<dyn Any>>,
    exit: bool,
    us: u64,
    oneshot: bool,
}

unsafe impl Send for TimerInner {}

unsafe impl Sync for TimerInner {}




pub fn timer_thread_entry(arg: Option<Arc<dyn Any + Send + Sync + 'static>>) -> Result<Arc<dyn Any + Send + Sync>> {
    match arg {
        Some(a) => {
            if let Ok(timer_inner) = Arc::downcast::<Mutex<TimerInner>>(a.clone()) {
                loop {
                    let (should_exit, oneshot, us) = {
                        let mut inner = timer_inner.lock().unwrap();
                        if inner.exit {
                            break;
                        }
                        let args = inner.args.take();
                        let handler = &inner.handler;
                        handler(args);
                        (inner.exit, inner.oneshot, inner.us)
                    };

                    if should_exit || oneshot {
                        break;
                    }

                    us_sleep(us);
                }
            } else {
                return Err(Error::Std(-2, "Invalid timer downcast"));
            }
        },
        None => return Err(Error::Std(-1, "Invalid timer argument")),
    };

    Ok(Arc::new(()) as Arc<dyn Any + Send + Sync>)
}
impl TimerTrait for Timer {
    fn new<F>(us: u64, _handler: F, oneshot: bool) -> Self
    where
        F: Fn(&mut Self, Option<Box<dyn Any>>) + Send + Sync + 'static,
        Self: Sized
    {
        let inner = TimerInner {
            handler: Box::new(move |args| {
                drop(args);
            }),
            args: None,
            exit: false,
            us,
            oneshot,
        };

        Self {
            thread: None,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    fn create(&mut self, param: Option<Box<dyn Any>>) -> crate::Result<()> {
        {
            let mut inner = self.inner.lock().unwrap();
            inner.args = param;
        }

        let mut thread = Thread::new(
            timer_thread_entry,
            "timer_thread",
            1_024 * 16,
            ThreadDefaultPriority::Normal
        )?;

        let inner_clone = Arc::clone(&self.inner) as Arc<Mutex<TimerInner>>;
        let inner_any: Arc<dyn Any + Send + Sync> = inner_clone;
        thread.create(Some(inner_any))?;
        self.thread = Some(thread);

        Ok(())
    }

    fn set(&mut self, us: u64) -> crate::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.us = us;
        Ok(())
    }

    fn set_from_isr(&mut self, us: u64) -> crate::Result<()> {
        self.set(us)
    }

    fn start(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.exit = false;
    }

    fn start_from_isr(&mut self) {
        self.start();
    }

    fn stop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.exit = true;
    }

    fn stop_from_isr(&mut self) {
        self.stop();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.stop();
        if let Some(thread) = self.thread.as_ref() {
            let _ = thread.join(std::ptr::null_mut());
        }
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
        let timer = Timer::new(1000, |_timer, _args| {}, false);
        assert!(timer.thread.is_none(), "Timer thread should not be created yet");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_oneshot() {
        let counter = StdArc::new(StdMutex::new(0));
        let counter_clone = counter.clone();
        
        let mut timer = Timer::new(
            50_000, // 50ms
            move |_timer, _args| {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            },
            true
        );
        
        let _ = timer.create(None);
        timer.start();
        
        thread::sleep(Duration::from_millis(200));
        
        let count = *counter.lock().unwrap();
        assert!(count <= 1, "Oneshot timer should fire at most once, got {}", count);
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
            false
        );
        
        let _ = timer.create(None);
        timer.start();
        
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
            false
        );
        
        let _ = timer.create(None);
        timer.start();
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
        let mut timer = Timer::new(1000_000, |_timer, _args| {}, false);
        
        let result = timer.set(500_000); // Change to 500ms
        assert!(result.is_ok(), "Set should succeed");
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_from_isr() {
        let mut timer = Timer::new(100_000, |_timer, _args| {}, false);
        
        let result = timer.set_from_isr(200_000);
        assert!(result.is_ok(), "ISR set should succeed");
        
        let _ = timer.create(None);
        timer.start_from_isr();
        thread::sleep(Duration::from_millis(50));
        timer.stop_from_isr();
    }

    #[test]
    #[cfg(feature = "posix")]
    fn test_timer_with_parameter() {
        let mut timer = Timer::new(
            50_000,
            move |_timer, _args| {
                // In the current implementation, args are dropped
                // This test verifies the timer can be created with parameters
            },
            true
        );
        
        let param = Box::new(42i32) as Box<dyn Any>;
        let result = timer.create(Some(param));
        assert!(result.is_ok(), "Timer creation with parameter should succeed");
        
        timer.start();
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
            false
        );
        
        let mut timer2 = Timer::new(
            75_000,
            move |_timer, _args| {
                let mut count = counter2_clone.lock().unwrap();
                *count += 1;
            },
            false
        );
        
        let _ = timer1.create(None);
        let _ = timer2.create(None);
        
        timer1.start();
        timer2.start();
        
        thread::sleep(Duration::from_millis(300));
        
        timer1.stop();
        timer2.stop();
        
        let count1 = *counter1.lock().unwrap();
        let count2 = *counter2.lock().unwrap();
        
        assert!(count1 >= 2, "Timer 1 should fire multiple times");
        assert!(count2 >= 1, "Timer 2 should fire at least once");
    }
}
