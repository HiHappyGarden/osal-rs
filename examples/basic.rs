//! Esempio base di utilizzo di OSAL-RS

use std::ffi::c_void;
use std::sync::Arc;
use osal_rs::{os_version, start_scheduler, stop_scheduler, Thread, ThreadDefaultPriority, ThreadTrait};

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
                println!("Esecuzione del task di base...");
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

            let mut ret_ptr = std::ptr::null_mut::<c_void>();
            //let mut ret_ptr: *mut std::os::raw::c_void = null_mut();
            t.join(&mut ret_ptr).unwrap();


            if !ret_ptr.is_null() {
                let ret_value = unsafe { *(ret_ptr as *const i32) };
                unsafe { Arc::from_raw(ret_ptr as *mut i32); } // Libera la memoria
                println!("Valore di ritorno del thread: {}", ret_value);
            } else {
                println!("Il thread non ha restituito alcun valore");
            }

            println!("✓ Task creato con successo: {:?}", t);
        }
        Err(e) => {
            println!("✗ Errore nella creazione del task: {:?}", e);
        }
    }
    println!();

    start_scheduler();

    println!("End execution {}", os_version());

}
