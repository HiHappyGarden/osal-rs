//! Esempio base di utilizzo di OSAL-RS

use std::{ffi::c_void, ptr::null_mut};
use std::sync::Arc;
use osal_rs::{os_version, start_scheduler, Thread, ThreadDefaultPriority, ThreadTrait};

fn main() {
    println!("===========================================");
    println!("  OSAL-RS - Operating System Abstraction Layer");
    println!("===========================================");
    println!();
    println!("OS: {}", os_version());
    println!();


    let thread = Thread::new(
        |_| {
            let mut ret = 1;
            for _ in 0..5 {
                ret += 1;
                println!("  Thread is running! Current count: {}", ret);
            }
            Ok(Arc::new(ret))
        },
        "base_task",
        1024 * 16,
        ThreadDefaultPriority::Normal,
    );



    match thread {
        Ok(mut t) => {

            t.create(None).expect("panic message");

            let mut ret_ptr = null_mut::<c_void>();

            t.join(&mut ret_ptr).expect("Failed to join thread");


            if !ret_ptr.is_null() {
                let ret_value = unsafe { *(ret_ptr as *const i32) };
                unsafe { Arc::from_raw(ret_ptr as *mut i32); } // Libera la memoria
                println!("Thread ret value: {}", ret_value);
            } else {
                println!("The thread did not return a value.");
            }

            println!("✓ Thread successfully created: {:?}", t);
        }
        Err(e) => {
            println!("✗ Thread creation error: {:?}", e);
        }
    }
    println!();

    start_scheduler();

    println!("End execution {}", os_version());

}
