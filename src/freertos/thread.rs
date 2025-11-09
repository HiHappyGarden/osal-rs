#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    use core::ffi::{c_char, c_void};
    use crate::freertos::ffi::{BaseType_t, TickType_t, UBaseType_t};

    pub type TaskHandle_t = *mut c_void;

    pub type TaskFunction_t = unsafe extern "C" fn(*mut c_void);

    unsafe extern "C" {

        // Task Management
        pub fn xTaskCreate(
            pvTaskCode: TaskFunction_t,
            pcName: *const c_char,
            usStackDepth: u16,
            pvParameters: *mut c_void,
            uxPriority: UBaseType_t,
            pxCreatedTask: *mut TaskHandle_t,
        ) -> BaseType_t;

        pub fn vTaskDelete(xTaskToDelete: TaskHandle_t);
        pub fn vTaskSuspend(xTaskToSuspend: TaskHandle_t);
        pub fn vTaskResume(xTaskToResume: TaskHandle_t);
    }
}

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::sync::Arc;
use core::any::Any;
use core::ffi::{c_char, c_void};
use core::fmt::Debug;
use core::ptr::null_mut;
use crate::freertos::ffi::{pdPASS};
use crate::freertos::thread::ffi::{xTaskCreate, TaskHandle_t, vTaskDelete, vTaskResume, vTaskSuspend};
pub use crate::traits::thread::Thread as ThreadTrait;


impl ThreadPriority for ThreadDefaultPriority {
    fn get_priority(&self) -> u32 {
        self.clone() as u32
    }
}

#[derive(Clone)]
pub struct Thread {
    handler: TaskHandle_t,
    callback: Arc<ThreadFunc>,
    param: Option<Arc<dyn Any + Send + Sync>>
}

unsafe extern "C" fn callback(param_ptr: *mut c_void) {
    if param_ptr.is_null() {
        return;
    }

    // Recreate the Box\<Thread\> we passed to the RTOS and run the callback.
    let boxed_thread: Box<Thread> = unsafe { Box::from_raw(param_ptr as *mut Thread) };

    let param_arc: Arc<dyn Any + Send + Sync> = boxed_thread
        .param
        .clone()
        .unwrap_or_else(|| Arc::new(()) as Arc<dyn Any + Send + Sync>);

    (boxed_thread.callback)(param_arc);
}


impl ThreadTrait<Thread> for Thread {
    fn new<F>(
        callback: F,
        name: &str,
        stack: u32,
        param: Option<Arc<dyn Any + Send + Sync>>,
        priority: impl ThreadPriority
    ) -> Result<Self, &'static str>
    where
        F: Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        let name_c = CString::new(name).map_err(|_| "Name not valid")?;
        let name_ptr = name_c.as_ptr() as *const c_char;


        let mut handler = null_mut();
        let callback_arc: Arc<ThreadFunc> = Arc::new(callback);
        let thread =  Thread  {
            handler,
            callback: callback_arc.clone(),
            param: param.clone(),
        };
        let thread_box = Box::new(thread);

        let result = unsafe {
            xTaskCreate(
                crate::freertos::thread::callback,
                name_ptr,
                stack as u16,
                Box::into_raw(thread_box) as *mut c_void,
                priority.get_priority(),
                &mut handler,
            )
        };

        if result == pdPASS {
            Ok(Thread { handler, callback: callback_arc, param })
        } else {
            Err("Impossible create thread")
        }
    }

    fn delete_current() {
        unsafe {
            vTaskDelete(null_mut());
        }
    }

    fn suspend(&self) {
        unsafe {
            vTaskSuspend(self.handler);
        }
    }

    fn resume(&self) {
        unsafe {
            vTaskResume(self.handler);
        }
    }

}

impl Drop for Thread {
    fn drop(&mut self) {
        unsafe {
            vTaskDelete(self.handler);
        }
    }
}

impl Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thread")
            .field("handler", &self.handler)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::sync::Arc;

    // NOTA: Questi test verificano solo la logica di compilazione e validazione.
    // I test che coinvolgono l'esecuzione effettiva di FreeRTOS richiedono
    // un ambiente embedded o un emulatore FreeRTOS e non possono essere
    // eseguiti con `cargo test` standard su Linux/macOS/Windows.

    #[test]
    fn test_thread_priority_values() {
        // Test valori priorità - questi non richiedono FreeRTOS runtime
        assert_eq!(ThreadDefaultPriority::None.get_priority(), 0);
        assert_eq!(ThreadDefaultPriority::Idle.get_priority(), 1);
        assert_eq!(ThreadDefaultPriority::Low.get_priority(), 2);
        assert_eq!(ThreadDefaultPriority::BelowNormal.get_priority(), 3);
        assert_eq!(ThreadDefaultPriority::Normal.get_priority(), 4);
        assert_eq!(ThreadDefaultPriority::AboveNormal.get_priority(), 5);
        assert_eq!(ThreadDefaultPriority::High.get_priority(), 6);
        assert_eq!(ThreadDefaultPriority::Realtime.get_priority(), 7);
        assert_eq!(ThreadDefaultPriority::ISR.get_priority(), 8);
    }

    #[test]
    fn test_thread_priority_trait() {
        // Test trait personalizzato
        struct CustomPriority(u32);

        impl ThreadPriority for CustomPriority {
            fn get_priority(&self) -> u32 {
                self.0
            }
        }

        let custom = CustomPriority(42);
        assert_eq!(custom.get_priority(), 42);
    }

    #[test]
    fn test_thread_priority_clone() {
        // Test clone delle priorità
        let priority = ThreadDefaultPriority::Normal;
        let cloned = priority.clone();
        assert_eq!(priority.get_priority(), cloned.get_priority());
    }

    #[test]
    fn test_arc_callback_type() {
        // Test che il tipo di callback sia corretto
        let callback: Arc<ThreadFunc> = Arc::new(|param| {
            // Questa callback può essere clonata e condivisa
            param
        });

        let callback_clone = callback.clone();
        let test_param = Arc::new(42u32) as Arc<dyn Any + Send + Sync>;
        let result = callback_clone(test_param.clone());

        // Verifica che il parametro sia stato passato correttamente
        assert!(result.downcast_ref::<u32>().is_some());
    }

}

// Test di integrazione che richiedono FreeRTOS runtime
// Questi test sono commentati perché non possono essere eseguiti su host Linux/macOS/Windows
// Per eseguirli, è necessario un ambiente embedded con FreeRTOS o un emulatore



mod integration_tests {
    use super::*;
    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use crate::freertos::thread::Thread;
    use crate::traits::thread::Thread as ThreadTrait;

    #[test]
    fn test_thread_creation() {
        // Test creazione thread di base
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        let thread = Thread::new(
            move |_param| {
                executed_clone.store(true, Ordering::SeqCst);
                Arc::new(())
            },
            "test_thread",
            1024,
            None,
            ThreadDefaultPriority::Normal,
        );

        assert!(thread.is_ok(), "Thread creation should succeed");
    }

    #[test]
    fn test_thread_with_param() {
        // Test thread con parametro
        let result = Arc::new(AtomicU32::new(0));
        let result_clone = result.clone();

        let input_value = Arc::new(42u32);

        let thread = Thread::new(
            move |param| {
                if let Some(value) = param.downcast_ref::<u32>() {
                    result_clone.store(*value, Ordering::SeqCst);
                }
                Arc::new(())
            },
            "test_param",
            1024,
            Some(input_value),
            ThreadDefaultPriority::Normal,
        );

        assert!(thread.is_ok(), "Thread with parameter should be created");
    }

    #[test]
    fn test_thread_priorities() {
        // Test diverse priorità
        let priorities = alloc::vec![
            ThreadDefaultPriority::Idle,
            ThreadDefaultPriority::Low,
            ThreadDefaultPriority::Normal,
            ThreadDefaultPriority::High,
            ThreadDefaultPriority::Realtime,
        ];

        for (idx, priority) in priorities.into_iter().enumerate() {
            let thread = Thread::new(
                |_| Arc::new(()),
                &alloc::format!("priority_test_{}", idx),
                1024,
                None,
                priority,
            );

            assert!(thread.is_ok(), "Thread with priority should be created");
        }
    }

    #[test]
    fn test_thread_name_validation() {
        // Test nome thread con carattere null (dovrebbe fallire)
        let thread = Thread::new(
            |_| Arc::new(()),
            "test\0invalid",
            1024,
            None,
            ThreadDefaultPriority::Normal,
        );

        assert!(thread.is_err(), "Thread with null character in name should fail");
        assert_eq!(thread.err(), Some("Name not valid"));
    }
}


