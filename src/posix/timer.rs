#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/posix_bindings.rs"));

    impl Default for pthread_mutex_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_cond_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_condattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_mutexattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for timespec {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }
}

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
