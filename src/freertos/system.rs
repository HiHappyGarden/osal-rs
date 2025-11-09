use super::ffi::TickType_t;
use super::constants::CONFIG_TICK_RATE_HZ;
use super::system::ffi::{vTaskDelay, vTaskEndScheduler, vTaskStartScheduler, xTaskGetTickCount};

#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    use crate::freertos::ffi::TickType_t;
    unsafe extern "C" {
        pub fn vTaskDelayUntil(pxPreviousWakeTime: *mut TickType_t, xTimeIncrement: TickType_t);
        pub fn xTaskGetTickCount() -> TickType_t;
        pub fn vTaskStartScheduler();
        pub fn vTaskEndScheduler();
        pub fn vTaskDelay( xTicksToDelay :  TickType_t );
    }
}

type TickType = TickType_t;

pub fn os_version() -> &'static str {
    "FreeRTOS V11.2.0"
}

pub fn us_sleep(us: u64) {
    unsafe {
        vTaskDelay( ( us / (CONFIG_TICK_RATE_HZ as u64) / 1_000 )as TickType_t);
    }
}

pub fn ticks_sleep(ticks_to_delay: TickType) {
    unsafe {
        vTaskDelay(ticks_to_delay);
    }
}

pub fn tick_current () -> TickType {
    unsafe {
        xTaskGetTickCount()
    }
}

pub fn us_to_ticks(us: u64) -> TickType {
    // Converti microsecondi in ticks: ticks = (us * CONFIG_TICK_RATE_HZ) / 1_000_000
    ((us * CONFIG_TICK_RATE_HZ as u64) / 1_000_000) as TickType_t
}

pub fn ticks_to_us(ticks: TickType) -> u64 {
    // Converti ticks in microsecondi: us = (ticks * 1_000_000) / CONFIG_TICK_RATE_HZ
    ((ticks as u64) * 1_000_000) / (CONFIG_TICK_RATE_HZ as u64)
}

pub fn start_scheduler() {
    unsafe {
        vTaskStartScheduler();
    }
}

pub fn stop_scheduler() {
    unsafe {
        vTaskEndScheduler();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_version() {
        assert_eq!(os_version(), "FreeRTOS V11.2.0");
    }

    #[test]
    fn test_us_to_ticks_conversion() {
        // Con CONFIG_TICK_RATE_HZ = 1000 Hz
        // 1 tick = 1 ms = 1000 us

        // 1000 us = 1 ms = 1 tick
        let ticks = us_to_ticks(1_000);
        assert_eq!(ticks, 1, "1000 us dovrebbe essere 1 tick con 1000 Hz");

        // 500 us = 0.5 ms = 0 ticks (arrotondato per difetto)
        let ticks = us_to_ticks(500);
        assert_eq!(ticks, 0, "500 us dovrebbe essere 0 ticks con 1000 Hz (arrotondato)");

        // 1 secondo = 1_000_000 us = 1000 ticks
        let ticks = us_to_ticks(1_000_000);
        assert_eq!(ticks, 1000, "1 secondo dovrebbe essere 1000 ticks con 1000 Hz");
    }

    #[test]
    fn test_ticks_to_us_conversion() {
        // Con CONFIG_TICK_RATE_HZ = 1000 Hz
        // 1 tick = 1 ms = 1000 us

        // 1 tick = 1 ms = 1000 us
        let us = ticks_to_us(1);
        assert_eq!(us, 1_000, "1 tick dovrebbe essere 1000 us con 1000 Hz");

        // 2 ticks = 2 ms = 2000 us
        let us = ticks_to_us(2);
        assert_eq!(us, 2_000, "2 ticks dovrebbero essere 2000 us con 1000 Hz");

        // 1000 ticks = 1 secondo = 1_000_000 us
        let us = ticks_to_us(1000);
        assert_eq!(us, 1_000_000, "1000 ticks dovrebbero essere 1 secondo");
    }

    #[test]
    fn test_us_to_ticks_round_trip() {
        // Test conversione andata e ritorno
        let original_us = 10_000; // 10 ms
        let ticks = us_to_ticks(original_us);
        let converted_us = ticks_to_us(ticks);

        // Dovrebbe essere uguale o molto vicino (tolleranza per arrotondamento)
        assert!(
            (converted_us as i64 - original_us as i64).abs() <= 1000,
            "Conversione round-trip dovrebbe essere precisa entro 1ms"
        );
    }

    #[test]
    fn test_ms_to_us_macro() {
        assert_eq!(ms_to_us!(1), 1_000);
        assert_eq!(ms_to_us!(10), 10_000);
        assert_eq!(ms_to_us!(1000), 1_000_000);
        assert_eq!(ms_to_us!(0), 0);
    }

    #[test]
    fn test_sec_to_us_macro() {
        assert_eq!(sec_to_us!(1), 1_000_000);
        assert_eq!(sec_to_us!(10), 10_000_000);
        assert_eq!(sec_to_us!(60), 60_000_000);
        assert_eq!(sec_to_us!(0), 0);
    }

    #[test]
    fn test_macro_with_expressions() {
        // Test che le macro funzionino con espressioni
        let ms = 5;
        assert_eq!(ms_to_us!(ms + 5), 10_000);

        let sec = 2;
        assert_eq!(sec_to_us!(sec * 3), 6_000_000);
    }

    #[test]
    fn test_conversion_edge_cases() {
        // Test casi limite

        // Zero
        assert_eq!(us_to_ticks(0), 0);
        assert_eq!(ticks_to_us(0), 0);

        // Valori piccoli
        assert_eq!(us_to_ticks(1), 0); // < 1 tick
        assert_eq!(ticks_to_us(1), 1_000); // 1 tick = 1000 us con 1000 Hz

        // Valori grandi
        let large_us = 3600_000_000; // 1 ora in us
        let ticks = us_to_ticks(large_us);
        assert!(ticks > 0, "Valori grandi dovrebbero essere convertiti correttamente");
    }

    #[test]
    fn test_tick_rate_consistency() {
        // Verifica che le conversioni siano consistenti con CONFIG_TICK_RATE_HZ

        // 1 secondo = CONFIG_TICK_RATE_HZ ticks
        let one_second_us = 1_000_000;
        let ticks = us_to_ticks(one_second_us);

        // Con qualsiasi valore di Hz, 1 secondo dovrebbe essere CONFIG_TICK_RATE_HZ ticks
        assert_eq!(
            ticks, CONFIG_TICK_RATE_HZ,
            "1 secondo dovrebbe essere esattamente CONFIG_TICK_RATE_HZ ticks"
        );
    }

    #[test]
    fn test_conversion_precision() {
        // Test precisione delle conversioni

        // Test millisecondi esatti con 1000 Hz
        let test_cases = alloc::vec![
            (1_000, 1),      // 1 ms = 1 tick
            (5_000, 5),      // 5 ms = 5 ticks
            (10_000, 10),    // 10 ms = 10 ticks
            (100_000, 100),  // 100 ms = 100 ticks
        ];

        for (us, expected_ticks) in test_cases {
            let ticks = us_to_ticks(us);
            assert_eq!(
                ticks, expected_ticks,
                "{} us dovrebbero essere {} ticks", us, expected_ticks
            );
        }
    }
}

// Test di integrazione commentati - richiedono ambiente FreeRTOS reale
/*
#[cfg(all(test, target_os = "none"))]
mod integration_tests {
    use super::*;

    #[test]
    fn test_us_sleep() {
        // Test sleep effettivo
        let start = tick_current();
        us_sleep(10_000); // 10 ms
        let end = tick_current();

        let elapsed_ticks = end - start;
        assert!(elapsed_ticks >= 19 && elapsed_ticks <= 21,
                "Sleep di 10ms dovrebbe durare circa 20 ticks");
    }

    #[test]
    fn test_ticks_sleep() {
        let start = tick_current();
        ticks_sleep(100);
        let end = tick_current();

        let elapsed = end - start;
        assert!(elapsed >= 99 && elapsed <= 101,
                "Sleep di 100 ticks dovrebbe durare circa 100 ticks");
    }

    #[test]
    fn test_tick_current() {
        let tick1 = tick_current();
        ticks_sleep(10);
        let tick2 = tick_current();

        assert!(tick2 > tick1, "Il tick count dovrebbe aumentare");
        assert_eq!(tick2 - tick1, 10, "Dovrebbero essere passati 10 ticks");
    }

    #[test]
    fn test_scheduler_lifecycle() {
        // Nota: questo test è molto specifico e potrebbe non funzionare
        // in tutti gli ambienti. Usare con cautela.

        // Lo scheduler dovrebbe essere avviabile
        start_scheduler();

        // Dopo lo start, end_scheduler dovrebbe fermare tutto
        // (in realtà start_scheduler non ritorna mai in FreeRTOS normale)
        end_scheduler();
    }
}
*/
