//! Esempio base di utilizzo di OSAL-RS


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
            for _ in 0..5 {
                println!("Esecuzione del task di base...");
            }
            stop_scheduler();
            Arc::new(())
        },
        "base_task",
        1024,
        None,
        ThreadDefaultPriority::Normal,
    );

    match thread {
        Ok(t) => {
            println!("✓ Task creato con successo: {:?}", t);
        }
        Err(e) => {
            println!("✗ Errore nella creazione del task: {}", e);
        }
    }
    println!();

    start_scheduler();

    println!("End execution {}", os_version());

}
