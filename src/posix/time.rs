
use core::{ffi::c_int, ptr::null_mut, sync::atomic::{AtomicU64, Ordering}};
use crate::posix::ffi::{timespec, clock_gettime, nanosleep, CLOCK_MONOTONIC};

type TickType_t = u64;

static START_TIME_NS: AtomicU64 = AtomicU64::new(0);

const POSIX_TICK_RATE_HZ: u64 = 1000;



#[macro_export]
macro_rules! ms_to_us {
    ($ms:expr) => {
        { ($ms as u64) * 1_000 }
    };
}

#[macro_export]
macro_rules! sec_to_us {
    ($sec:expr) => {
        { ($sec as u64) * 1_000_000 }
    };
}

/// Converts microseconds to POSIX ticks
#[macro_export]
macro_rules! tick_from_us {
    ($us:expr) => {
        (($us as u64) * POSIX_TICK_RATE_HZ) / 1_000_000
    };
}

/// Convert microseconds to ticks
#[macro_export]
macro_rules! us_to_ticks {
    ($us:expr) => {
        (($us as u64) * POSIX_TICK_RATE_HZ) / 1_000_000
    };
}



/// Convert ticks to microseconds
#[macro_export]
macro_rules! ticks_to_us {
    ($ticks:expr) => {
        (($ticks as u64) * 1_000_000) / POSIX_TICK_RATE_HZ
    };
}

pub fn us_sleep(us: u64) {
    unsafe {
        let ts = timespec {
            tv_sec: (us / 1_000_000) as i64,
            tv_nsec: ((us % 1_000_000) * 1000) as i64,
        };

        nanosleep(&ts, null_mut());
    }
}


/// Get current time in nanoseconds
fn get_time_ns() -> u64 {
    unsafe {
        let mut ts = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        clock_gettime(CLOCK_MONOTONIC as c_int, &mut ts);
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    }
}

/// Initialize start time if not already done
fn init_start_time() {
    if START_TIME_NS.load(Ordering::Relaxed) == 0 {
        START_TIME_NS.store(get_time_ns(), Ordering::Relaxed);
    }
}



pub fn ticks_sleep(ticks_to_delay: TickType_t) {
    // Convert ticks to microseconds
    let us = ticks_to_us(ticks_to_delay);
    us_sleep(us);
}

pub fn tick_current() -> TickType_t {
    init_start_time();
    let start = START_TIME_NS.load(Ordering::Relaxed);
    let now = get_time_ns();
    let elapsed_ns = now - start;

    // Convert nanoseconds to ticks
    // ticks = (elapsed_ns * TICK_RATE_HZ) / 1_000_000_000
    let ticks = (elapsed_ns * POSIX_TICK_RATE_HZ) / 1_000_000_000;
    ticks as TickType_t
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us_to_ticks_conversion() {
        // Con POSIX_TICK_RATE_HZ = 1000 Hz
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
        // Con POSIX_TICK_RATE_HZ = 1000 Hz
        // 1 tick = 1 ms = 1000 us

        // 1 tick = 1000 us
        let us = ticks_to_us(1);
        assert_eq!(us, 1_000, "1 tick dovrebbe essere 1000 us");

        // 10 ticks = 10_000 us = 10 ms
        let us = ticks_to_us(10);
        assert_eq!(us, 10_000, "10 ticks dovrebbero essere 10_000 us");

        // 1000 ticks = 1_000_000 us = 1 secondo
        let us = ticks_to_us(1000);
        assert_eq!(us, 1_000_000, "1000 ticks dovrebbero essere 1_000_000 us (1 secondo)");
    }

    #[test]
    fn test_tick_rate_constant() {
        // Verifica che il tick rate sia 1000 Hz come FreeRTOS standard
        assert_eq!(POSIX_TICK_RATE_HZ, 1000, "Tick rate dovrebbe essere 1000 Hz");
    }

    #[test]
    fn test_tick_current_monotonic() {
        // Test che tick_current() sia monotono (sempre crescente)
        let t1 = tick_current();
        us_sleep(5_000); // Sleep 5ms
        let t2 = tick_current();

        assert!(t2 > t1, "tick_current() dovrebbe essere monotono");
        assert!(t2 >= t1 + 4, "Dovrebbero essere trascorsi almeno 4 ticks (5ms con tolleranza)");
    }

    #[test]
    fn test_round_trip_conversion() {
        // Test conversione andata e ritorno
        let original_ticks = 100u64;
        let us = ticks_to_us(original_ticks);
        let back_to_ticks = us_to_ticks(us);

        assert_eq!(
            back_to_ticks,
            original_ticks,
            "Conversione ticks -> us -> ticks dovrebbe mantenere il valore"
        );
    }
}

