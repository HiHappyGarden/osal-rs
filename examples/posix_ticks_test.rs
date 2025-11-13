//! Test example for POSIX ticks_sleep implementation

use osal_rs::{ticks_sleep, us_to_ticks, ticks_to_us, tick_current, us_sleep};
use std::time::Instant;

fn main() {
    println!("===========================================");
    println!("  OSAL-RS - POSIX ticks_sleep() Test");
    println!("===========================================");
    println!();

    // Test 1: Conversion functions
    println!("Test 1: Tick conversion functions");
    println!("  TICK_RATE_HZ = 1000 (1 tick = 1 ms = 1000 us)");
    println!();

    // 1 tick should equal 1000 microseconds
    let us = ticks_to_us(1);
    println!("  ticks_to_us(1) = {} us (expected: 1000 us)", us);
    assert_eq!(us, 1000, "1 tick should be 1000 us");

    // 1000 us should equal 1 tick
    let ticks = us_to_ticks(1000);
    println!("  us_to_ticks(1000) = {} ticks (expected: 1 tick)", ticks);
    assert_eq!(ticks, 1, "1000 us should be 1 tick");

    // 1 second = 1,000,000 us = 1000 ticks
    let ticks = us_to_ticks(1_000_000);
    println!("  us_to_ticks(1_000_000) = {} ticks (expected: 1000 ticks)", ticks);
    assert_eq!(ticks, 1000, "1 second should be 1000 ticks");

    println!("  ✓ All conversion tests passed");
    println!();

    // Test 2: tick_current()
    println!("Test 2: tick_current() monotonic clock");
    let t1 = tick_current();
    us_sleep(10_000); // Sleep 10ms
    let t2 = tick_current();
    println!("  tick_current() before sleep: {}", t1);
    println!("  tick_current() after 10ms sleep: {}", t2);
    println!("  Elapsed ticks: {} (expected: ~10)", t2 - t1);
    assert!(t2 > t1, "tick_current should increase");
    assert!((t2 - t1) >= 9 && (t2 - t1) <= 12, "Should be approximately 10 ticks");
    println!("  ✓ tick_current() test passed");
    println!();

    // Test 3: ticks_sleep() accuracy
    println!("Test 3: ticks_sleep() accuracy");

    // Sleep for 50 ticks (50 ms)
    let sleep_ticks = 50;
    println!("  Sleeping for {} ticks ({} ms)...", sleep_ticks, sleep_ticks);

    let start = Instant::now();
    ticks_sleep(sleep_ticks);
    let elapsed = start.elapsed();

    println!("  Actual sleep time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("  Expected: ~{} ms", sleep_ticks);

    // Allow some tolerance (±5ms)
    let elapsed_ms = elapsed.as_millis() as u32;
    assert!(
        elapsed_ms >= (sleep_ticks - 5) && elapsed_ms <= (sleep_ticks + 10),
        "Sleep duration should be approximately {} ms (got {} ms)",
        sleep_ticks,
        elapsed_ms
    );
    println!("  ✓ ticks_sleep() accuracy test passed");
    println!();

    // Test 4: Multiple short sleeps
    println!("Test 4: Multiple short ticks_sleep() calls");
    let iterations = 10;
    let ticks_per_sleep = 5; // 5 ticks = 5 ms each

    let start = Instant::now();
    for i in 0..iterations {
        ticks_sleep(ticks_per_sleep);
        if i % 3 == 0 {
            print!(".");
        }
    }
    println!();
    let elapsed = start.elapsed();

    let total_expected_ms = iterations * ticks_per_sleep;
    let elapsed_ms = elapsed.as_millis() as u32;

    println!("  {} iterations of {} ticks each", iterations, ticks_per_sleep);
    println!("  Total sleep time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("  Expected: ~{} ms", total_expected_ms);

    assert!(
        elapsed_ms >= (total_expected_ms - 10) && elapsed_ms <= (total_expected_ms + 20),
        "Total sleep should be approximately {} ms (got {} ms)",
        total_expected_ms,
        elapsed_ms
    );
    println!("  ✓ Multiple sleep test passed");
    println!();

    println!("===========================================");
    println!("  All tests passed! ✓");
    println!("===========================================");
}

