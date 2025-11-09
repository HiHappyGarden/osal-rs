//! Test example for POSIX pthread support

use std::sync::Arc;
use osal_rs::{os_version, Thread, ThreadDefaultPriority, ThreadTrait};

fn main() {
    println!("===========================================");
    println!("  OSAL-RS - POSIX pthread Test");
    println!("===========================================");
    println!();
    println!("OS: {}", os_version());
    println!();

    println!("Creating thread with pthread...");

    let thread = Thread::new(
        |_| {
            println!("Thread is running!");
            for i in 0..5 {
                println!("  Iteration {}/5", i + 1);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            println!("Thread completed!");
            Arc::new(())
        },
        "test_thread",
        1024,
        None,
        ThreadDefaultPriority::Normal,
    );

    match thread {
        Ok(_t) => {
            println!("✓ Thread created successfully");
            // Give the thread time to run
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
        Err(e) => {
            println!("✗ Error creating thread: {}", e);
        }
    }

    println!();
    println!("Main thread exiting...");
}

