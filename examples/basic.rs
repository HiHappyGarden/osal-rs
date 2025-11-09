//! Esempio base di utilizzo di OSAL-RS

use osal_rs::{os_version, init, Thread};

fn main() {
    println!("===========================================");
    println!("  OSAL-RS - Operating System Abstraction Layer");
    println!("===========================================");
    println!();
    println!("Sistema Operativo: {}", os_version());
    println!();
    println!("Inizializzazione OSAL...");
    
    init();
    
    println!("✓ OSAL inizializzato con successo!");
    println!();
    
    // Esempio di utilizzo dell'API unificata
    //let _task = Thread::new();
    println!("✓ Task creato usando l'interfaccia unificata");

    #[cfg(feature = "freertos")]
    println!("Compilato con backend FreeRTOS v11.2.0");
    
    #[cfg(feature = "posix")]
    println!("Compilato con backend POSIX");
}
