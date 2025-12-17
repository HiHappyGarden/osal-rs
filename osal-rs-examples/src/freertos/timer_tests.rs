#[cfg(test)]
mod tests {
    extern crate alloc;
    
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use core::any::Any;
    use core::sync::atomic::{AtomicU32, Ordering};
    use osal_rs::os::*;
    use osal_rs::utils::{Result, OsalRsBool};
    use core::time::Duration;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new(
            "test_timer",
            Duration::from_millis(100).to_ticks(),
            false,
            None,
            |_timer, _param| {
                Ok(None)
            }
        );

        assert!(timer.is_ok());
    }

    #[test]
    fn test_timer_one_shot() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        
        let timer = Timer::new(
            "oneshot_timer",
            Duration::from_millis(50).to_ticks(),
            false,
            None,
            |_timer, _param| {
                COUNTER.fetch_add(1, Ordering::SeqCst);
                Ok(None)
            }
        ).unwrap();

        let result = timer.start(Duration::from_millis(10));
        assert_eq!(result, OsalRsBool::True);
        
        // Wait for timer to fire
        Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(200));
        
        assert!(COUNTER.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn test_timer_auto_reload() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        
        let timer = Timer::new(
            "autoreload_timer",
            Duration::from_millis(50).to_ticks(),
            true,
            None,
            |_timer, _param| {
                COUNTER.fetch_add(1, Ordering::SeqCst);
                Ok(None)
            }
        ).unwrap();

        let result = timer.start(Duration::from_millis(10));
        assert_eq!(result, OsalRsBool::True);
        
        Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(300));
        
        assert!(COUNTER.load(Ordering::SeqCst) >= 2);
        
        timer.stop(Duration::from_millis(10));
    }

    #[test]
    fn test_timer_start_stop() {
        let timer = Timer::new(
            "startstop_timer",
            Duration::from_millis(100).to_ticks(),
            false,
            None,
            |_timer, _param| {
                Ok(None)
            }
        ).unwrap();

        let start_result = timer.start(Duration::from_millis(10));
        assert_eq!(start_result, OsalRsBool::True);
        
        let stop_result = timer.stop(Duration::from_millis(10));
        assert_eq!(stop_result, OsalRsBool::True);
    }

    #[test]
    fn test_timer_reset() {
        let timer = Timer::new(
            "reset_timer",
            Duration::from_millis(100).to_ticks(),
            false,
            None,
            |_timer, _param| {
                Ok(None)
            }
        ).unwrap();

        timer.start(Duration::from_millis(10));
        
        let reset_result = timer.reset(Duration::from_millis(10));
        assert_eq!(reset_result, OsalRsBool::True);
        
        timer.stop(Duration::from_millis(10));
    }

    #[test]
    fn test_timer_change_period() {
        let timer = Timer::new(
            "period_timer",
            Duration::from_millis(100).to_ticks(),
            false,
            None,
            |_timer, _param| {
                Ok(None)
            }
        ).unwrap();

        timer.start(Duration::from_millis(10));
        
        let change_result = timer.change_period(
            Duration::from_millis(200).to_ticks(),
            Duration::from_millis(10).to_ticks()
        );
        assert_eq!(change_result, OsalRsBool::True);
        
        timer.stop(Duration::from_millis(10));
    }

    #[test]
    fn test_timer_with_param() {
        let test_value: u32 = 42;
        let param: Arc<dyn Any + Send + Sync> = Arc::new(test_value);
        
        static RECEIVED_VALUE: AtomicU32 = AtomicU32::new(0);
        
        let timer = Timer::new(
            "param_timer",
            Duration::from_millis(50).to_ticks(),
            false,
            Some(param),
            |_timer, param| {
                if let Some(p) = param {
                    if let Some(val) = p.downcast_ref::<u32>() {
                        RECEIVED_VALUE.store(*val, Ordering::SeqCst);
                    }
                }
                Ok(None)
            }
        ).unwrap();

        timer.start(Duration::from_millis(10));
        
        Thread::get_current().wait_notification(0, 0xFFFFFFFF, Duration::from_millis(200));
        
        assert_eq!(RECEIVED_VALUE.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_timer_delete() {
        let mut timer = Timer::new(
            "delete_timer",
            Duration::from_millis(100).to_ticks(),
            false,
            None,
            |_timer, _param| {
                Ok(None)
            }
        ).unwrap();

        let delete_result = timer.delete(Duration::from_millis(10));
        assert_eq!(delete_result, OsalRsBool::True);
    }
}
