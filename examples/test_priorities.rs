//! Esempio che dimostra i test delle priorità dei thread
//! Questo esempio può essere compilato ed eseguito su host senza FreeRTOS

#[cfg(feature = "freertos")]
use osal_rs::{ThreadDefaultPriority, ThreadPriority};

fn main() {
    #[cfg(feature = "freertos")]
    {
        println!("===========================================");
        println!("  Test Priorità Thread - OSAL-RS");
        println!("===========================================\n");

        // Test dei valori delle priorità
        println!("Test valori priorità:");
        assert_eq!(ThreadDefaultPriority::None.get_priority(), 0);
        println!("  ✓ None = {}", ThreadDefaultPriority::None.get_priority());

        assert_eq!(ThreadDefaultPriority::Idle.get_priority(), 1);
        println!("  ✓ Idle = {}", ThreadDefaultPriority::Idle.get_priority());

        assert_eq!(ThreadDefaultPriority::Low.get_priority(), 2);
        println!("  ✓ Low = {}", ThreadDefaultPriority::Low.get_priority());

        assert_eq!(ThreadDefaultPriority::BelowNormal.get_priority(), 3);
        println!("  ✓ BelowNormal = {}", ThreadDefaultPriority::BelowNormal.get_priority());

        assert_eq!(ThreadDefaultPriority::Normal.get_priority(), 4);
        println!("  ✓ Normal = {}", ThreadDefaultPriority::Normal.get_priority());

        assert_eq!(ThreadDefaultPriority::AboveNormal.get_priority(), 5);
        println!("  ✓ AboveNormal = {}", ThreadDefaultPriority::AboveNormal.get_priority());

        assert_eq!(ThreadDefaultPriority::High.get_priority(), 6);
        println!("  ✓ High = {}", ThreadDefaultPriority::High.get_priority());

        assert_eq!(ThreadDefaultPriority::Realtime.get_priority(), 7);
        println!("  ✓ Realtime = {}", ThreadDefaultPriority::Realtime.get_priority());

        assert_eq!(ThreadDefaultPriority::ISR.get_priority(), 8);
        println!("  ✓ ISR = {}", ThreadDefaultPriority::ISR.get_priority());

        // Test clone delle priorità
        println!("\nTest clonazione priorità:");
        let priority = ThreadDefaultPriority::Normal;
        let cloned = priority.clone();
        assert_eq!(priority.get_priority(), cloned.get_priority());
        println!("  ✓ Priorità clonata correttamente: {} == {}",
                 priority.get_priority(), cloned.get_priority());

        // Test priorità personalizzata
        println!("\nTest priorità personalizzata:");
        struct CustomPriority(u32);

        impl ThreadPriority for CustomPriority {
            fn get_priority(&self) -> u32 {
                self.0
            }
        }

        let custom = CustomPriority(42);
        assert_eq!(custom.get_priority(), 42);
        println!("  ✓ Priorità personalizzata = {}", custom.get_priority());

        println!("\n===========================================");
        println!("  Tutti i test sono passati con successo! ✓");
        println!("===========================================");
    }

    #[cfg(not(feature = "freertos"))]
    {
        println!("Questo esempio richiede il feature 'freertos'");
        println!("Esegui con: cargo run --example test_priorities --features freertos");
    }
}

