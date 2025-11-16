#![cfg(all(test, feature = "posix"))]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use osal_rs::{Thread, ThreadDefaultPriority, ThreadTrait};

#[test]
fn test_thread_creation() {
    let mut result = Thread::new(
        |_| {
            Ok(Arc::new(()))
        },
        "test_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    let thread = result.as_mut();

    thread.unwrap().create(None).expect("TODO: panic message");

    assert!(result.is_ok(), "Failed to create thread");
}

#[test]
fn test_thread_execution() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let thread = Thread::new(
        move |_| {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            Ok(Arc::new(()))
        },
        "exec_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    let thread = thread.unwrap().create(None);

    assert!(thread.is_ok(), "Failed to create thread");

    // Give thread time to execute
    std::thread::sleep(Duration::from_millis(100));

    let final_count = *counter.lock().unwrap();
    assert_eq!(final_count, 1, "Thread did not execute");
}

#[test]
fn test_thread_with_parameter() {
    let result = Arc::new(Mutex::new(0));
    let result_clone = result.clone();
    let param_value = 42;

    let thread = Thread::new(
        move |param| {
            if let Some(value) = param.unwrap().downcast_ref::<i32>() {
                let mut res = result_clone.lock().unwrap();
                *res = *value;
            }
            Ok(Arc::new(()))
        },
        "param_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    let thread = thread.unwrap().create(Some(Arc::new(param_value)));

    assert!(thread.is_ok(), "Failed to create thread with parameter");

    // Give thread time to execute
    thread::sleep(Duration::from_millis(100));

    let final_value = *result.lock().unwrap();
    assert_eq!(final_value, 42, "Thread did not receive parameter correctly");
}

#[test]
fn test_thread_with_name() {
    let thread = Thread::new(
        |_| {
            Ok(Arc::new(()))
        },
        "named_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    let thread = thread.unwrap().create(None);

    assert!(thread.is_ok(), "Failed to create named thread");

    // Give thread time to start
    std::thread::sleep(Duration::from_millis(50));
}

#[test]
fn test_thread_with_stack_size() {
    let thread = Thread::new(
        |_| {
            Ok(Arc::new(()))
        },
        "stack_thread",
        16384, // 16KB stack
        ThreadDefaultPriority::Normal,
    );

    let thread = thread.unwrap().create(None);

    assert!(thread.is_ok(), "Failed to create thread with custom stack size");
}

#[test]
fn test_thread_with_priority() {
    let thread_low = Thread::new(
        |_| Ok(Arc::new(())),
        "low_priority",
        0,
        ThreadDefaultPriority::Low,
    );

    let thread_low = thread_low.unwrap().create(None);

    let thread_normal = Thread::new(
        |_| Ok(Arc::new(())),
        "normal_priority",
        0,
        ThreadDefaultPriority::Normal,
    );

    let thread_normal = thread_normal.unwrap().create(None);

    let thread_high = Thread::new(
        |_| Ok(Arc::new(())),
        "high_priority",
        0,
        ThreadDefaultPriority::High,
    );

    let thread_high = thread_high.unwrap().create(None);

    assert!(thread_low.is_ok(), "Failed to create low priority thread");
    assert!(thread_normal.is_ok(), "Failed to create normal priority thread");
    assert!(thread_high.is_ok(), "Failed to create high priority thread");
}

#[test]
fn test_multiple_threads() {
    let counter = Arc::new(Mutex::new(0));
    let mut threads = Vec::new();

    for i in 0..5 {
        let counter_clone = counter.clone();
        let thread = Thread::new(
            move |_| {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                Ok(Arc::new(()))
            },
            &format!("thread_{}", i),
            0,
            ThreadDefaultPriority::Normal,
        );

        let thread = thread.unwrap().create(None);

        assert!(thread.is_ok(), "Failed to create thread {}", i);
        threads.push(thread.unwrap());
    }

    // Give threads time to execute
    std::thread::sleep(Duration::from_millis(200));

    let final_count = *counter.lock().unwrap();
    assert_eq!(final_count, 5, "Not all threads executed");
}

#[test]
fn test_thread_with_return_value() {
    let thread = Thread::new(
        |_| {
            Ok(Arc::new(100))
        },
        "return_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    let thread = thread.unwrap().create(None);

    assert!(thread.is_ok(), "Failed to create thread");
}

#[test]
fn test_thread_join() {
    let thread = Thread::new(
        |_| {
            std::thread::sleep(Duration::from_millis(100));
            Ok(Arc::new(()))
        },
        "join_thread",
        0,
        ThreadDefaultPriority::Normal,
    );



    assert!(thread.is_ok(), "Failed to create thread");

    let mut thread = thread.unwrap();

    let _ = thread.create(None);

    let result = thread.join(std::ptr::null_mut());
    assert!(result.is_ok(), "Failed to join thread");
}

#[test]
fn test_thread_with_empty_name() {
    let thread = Thread::new(
        |_| Ok(Arc::new(())),
        "",
        0,
        ThreadDefaultPriority::Normal,
    );

    let thread = thread.unwrap().create(None);

    assert!(thread.is_ok(), "Failed to create thread with empty name");
}

#[test]
fn test_thread_suspend_resume() {
    let thread = Thread::new(
        |_| {
            std::thread::sleep(Duration::from_millis(100));
            Ok(Arc::new(()))
        },
        "suspend_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    assert!(thread.is_ok(), "Failed to create thread");


    let mut thread = thread.unwrap();

    let _ = thread.create(None);

    // Note: POSIX threads don't have direct suspend/resume
    // These are no-ops in the current implementation
    thread.suspend();
    thread.resume();
}

#[test]
fn test_thread_debug_format() {
    let thread = Thread::new(
        |_| {
            std::thread::sleep(Duration::from_millis(100));
            Ok(Arc::new(()))
        },
        "debug_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    assert!(thread.is_ok(), "Failed to create thread");

    let mut thread = thread.unwrap();

    thread.create(None).expect("TODO: panic message");

    let debug_string = format!("{:?}", thread);
    assert!(!debug_string.is_empty(), "Debug format returned empty string");
}

#[test]
fn test_thread_with_complex_parameter() {
    #[derive(Clone)]
    struct ComplexParam {
        value: i32,
        text: String,
    }

    let result = Arc::new(Mutex::new(ComplexParam {
        value: 0,
        text: String::new(),
    }));
    let result_clone = result.clone();

    let param = ComplexParam {
        value: 123,
        text: "test".to_string(),
    };

    let thread = Thread::new(
        move |param| {
            if let Some(complex) = param.unwrap().downcast_ref::<ComplexParam>() {
                let mut res = result_clone.lock().unwrap();
                res.value = complex.value;
                res.text = complex.text.clone();
            }
            Ok(Arc::new(()))
        },
        "complex_thread",
        0,
        ThreadDefaultPriority::Normal,
    );

    assert!(thread.is_ok(), "Failed to create thread with complex parameter");

    let _ = thread.unwrap().create(Some(Arc::new(param)));

    // Give thread time to execute
    std::thread::sleep(Duration::from_millis(100));

    let final_result = result.lock().unwrap();
    assert_eq!(final_result.value, 123, "Complex parameter value not passed correctly");
    assert_eq!(final_result.text, "test", "Complex parameter text not passed correctly");
}

#[test]
fn test_thread_send_sync_traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<Thread>();
    assert_sync::<Thread>();
}

#[test]
fn test_concurrent_thread_creation() {
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let thread = Thread::new(
                    |_| Ok(Arc::new(())),
                    &format!("concurrent_{}", i),
                    0,
                    ThreadDefaultPriority::Normal,
                );

                let _ = thread.unwrap().create(None);
            })
        })
        .collect();

    for handle in handles {
        let result = handle.join();
        assert!(result.is_ok(), "Failed to create thread concurrently");
    }
}

