//! Esempio che mostra come leggere le costanti di configurazione FreeRTOS
//! Incluso CONFIG_TICK_RATE_HZ

#[cfg(feature = "freertos")]
use osal_rs::constants::*;

fn main() {
    #[cfg(feature = "freertos")]
    {
        println!("===========================================");
        println!("  Configurazione FreeRTOS - OSAL-RS");
        println!("===========================================\n");

        // Lettura di CONFIG_TICK_RATE_HZ
        println!("Configurazione Tick:");
        println!("  CONFIG_TICK_RATE_HZ: {} Hz", CONFIG_TICK_RATE_HZ);
        println!("  Periodo tick: {} ms", get_tick_period_ms());
        println!();

        // Altre costanti di configurazione
        println!("Configurazione Task:");
        println!("  CONFIG_MAX_PRIORITIES: {}", CONFIG_MAX_PRIORITIES);
        println!("  CONFIG_MINIMAL_STACK_SIZE: {} words", CONFIG_MINIMAL_STACK_SIZE);
        println!("  CONFIG_MAX_TASK_NAME_LEN: {} chars", CONFIG_MAX_TASK_NAME_LEN);
        println!();

        println!("Configurazione Memoria:");
        println!("  CONFIG_TOTAL_HEAP_SIZE: {} bytes ({} KB)",
                 CONFIG_TOTAL_HEAP_SIZE,
                 CONFIG_TOTAL_HEAP_SIZE / 1024);
        println!();

        println!("Configurazione Sistema:");
        println!("  CONFIG_CPU_CLOCK_HZ: {} Hz ({} MHz)",
                 CONFIG_CPU_CLOCK_HZ,
                 CONFIG_CPU_CLOCK_HZ / 1_000_000);
        println!();

        // Esempi di conversione
        println!("===========================================");
        println!("  Esempi di Conversione Tempo");
        println!("===========================================\n");

        let test_ms = [1, 10, 100, 1000, 5000];
        println!("Millisecondi -> Ticks:");
        for ms in &test_ms {
            println!("  {} ms = {} ticks", ms, ms_to_ticks(*ms));
        }
        println!();

        let test_ticks = [1, 10, 100, 1000, 5000];
        println!("Ticks -> Millisecondi:");
        for ticks in &test_ticks {
            println!("  {} ticks = {} ms", ticks, ticks_to_ms(*ticks));
        }
        println!();

        // Calcoli pratici
        println!("===========================================");
        println!("  Calcoli Pratici");
        println!("===========================================\n");

        println!("Per attendere 1 secondo:");
        println!("  Delay: {} ticks", ms_to_ticks(1000));
        println!();

        println!("Per attendere 250 millisecondi:");
        println!("  Delay: {} ticks", ms_to_ticks(250));
        println!();

        println!("Numero massimo di tick in un secondo:");
        println!("  {} ticks/secondo", CONFIG_TICK_RATE_HZ);
        println!();

        println!("===========================================");
        println!("  ✓ Configurazione letta con successo!");
        println!("===========================================");
    }

    #[cfg(not(feature = "freertos"))]
    {
        println!("Questo esempio richiede il feature 'freertos'");
        println!("Esegui con: cargo run --example freertos_config --features freertos");
    }
}

