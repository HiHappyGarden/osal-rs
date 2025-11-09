use crate::posix::system::ffi::{usleep, useconds_t, clock_gettime, timespec, CLOCK_MONOTONIC};
use core::sync::atomic::{AtomicU64, Ordering};

#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    use core::ffi::{c_uint, c_int, c_long};

    pub type useconds_t = c_uint;
    pub type clockid_t = c_int;
    pub type time_t = c_long;

    pub const CLOCK_MONOTONIC: clockid_t = 1;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct timespec {
        pub tv_sec: time_t,
        pub tv_nsec: c_long,
    }

    unsafe extern "C" {
        pub fn usleep(useconds: useconds_t) -> c_int;
        pub fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int;
    }
}

static mut START_MAIN_LOOP : bool = false;
static START_TIME_NS: AtomicU64 = AtomicU64::new(0);

// Simulated tick rate: 1000 Hz (1 tick = 1 millisecond)
// This matches the typical FreeRTOS configuration
const POSIX_TICK_RATE_HZ: u64 = 1000;

type TickType = u32;

/// Get current time in nanoseconds
fn get_time_ns() -> u64 {
    unsafe {
        let mut ts = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        clock_gettime(CLOCK_MONOTONIC, &mut ts);
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    }
}

/// Initialize start time if not already done
fn init_start_time() {
    if START_TIME_NS.load(Ordering::Relaxed) == 0 {
        START_TIME_NS.store(get_time_ns(), Ordering::Relaxed);
    }
}

pub fn os_version() -> &'static str {
    "POSIX"
}

pub fn us_sleep(us: u64) {
    unsafe { usleep(us as useconds_t); }
}

pub fn ticks_sleep(ticks_to_delay: TickType) {
    // Convert ticks to microseconds
    let us = ticks_to_us(ticks_to_delay);
    us_sleep(us);
}

pub fn tick_current() -> TickType {
    init_start_time();
    let start = START_TIME_NS.load(Ordering::Relaxed);
    let now = get_time_ns();
    let elapsed_ns = now - start;

    // Convert nanoseconds to ticks
    // ticks = (elapsed_ns * TICK_RATE_HZ) / 1_000_000_000
    let ticks = (elapsed_ns * POSIX_TICK_RATE_HZ) / 1_000_000_000;
    ticks as TickType
}

pub fn us_to_ticks(us: u64) -> TickType {
    // Convert microseconds to ticks: ticks = (us * TICK_RATE_HZ) / 1_000_000
    ((us * POSIX_TICK_RATE_HZ) / 1_000_000) as TickType
}

pub fn ticks_to_us(ticks: TickType) -> u64 {
    // Convert ticks to microseconds: us = (ticks * 1_000_000) / TICK_RATE_HZ
    ((ticks as u64) * 1_000_000) / POSIX_TICK_RATE_HZ
}


pub fn start_scheduler() {
    unsafe { START_MAIN_LOOP = true; }
    loop {
        unsafe {
            if !START_MAIN_LOOP {
                break;
            }
        }
    }
}

pub fn stop_scheduler() {
    unsafe { START_MAIN_LOOP = false; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_version() {
        assert_eq!(os_version(), "POSIX");
    }

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
        let original_ticks = 100u32;
        let us = ticks_to_us(original_ticks);
        let back_to_ticks = us_to_ticks(us);

        assert_eq!(
            back_to_ticks,
            original_ticks,
            "Conversione ticks -> us -> ticks dovrebbe mantenere il valore"
        );
    }
}

