//! Basic example demonstrating the OSAL library usage

use osal_rs::{Thread, Mutex, Semaphore, Queue, Timer, time::Duration};
use std::sync::Arc;

fn main() {
    println!("=== OSAL-RS Examples ===\n");

    // Example 1: Threads
    println!("1. Thread Example:");
    let thread = Thread::new("example_thread", || {
        println!("   Hello from a thread!");
    });
    thread.join().unwrap();
    println!();

    // Example 2: Mutex
    println!("2. Mutex Example:");
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let thread = Thread::new("counter_thread", move || {
        for _ in 0..5 {
            let mut guard = counter_clone.lock();
            *guard += 1;
        }
    });

    thread.join().unwrap();
    println!("   Counter value: {}", *counter.lock());
    println!();

    // Example 3: Semaphore
    println!("3. Semaphore Example:");
    let sem = Arc::new(Semaphore::new(0));
    let sem_clone = sem.clone();

    let thread = Thread::new("sem_poster", move || {
        println!("   Thread waiting 100ms before posting...");
        Thread::sleep(Duration::from_millis(100)).unwrap();
        sem_clone.post().unwrap();
        println!("   Semaphore posted!");
    });

    println!("   Main waiting on semaphore...");
    sem.wait().unwrap();
    println!("   Semaphore received!");
    thread.join().unwrap();
    println!();

    // Example 4: Queue
    println!("4. Queue Example:");
    let queue = Arc::new(Queue::new(10));
    let queue_clone = queue.clone();

    let thread = Thread::new("producer", move || {
        for i in 0..5 {
            queue_clone.send(i).unwrap();
            println!("   Sent: {}", i);
            Thread::sleep(Duration::from_millis(50)).unwrap();
        }
    });

    for _ in 0..5 {
        let value = queue.recv().unwrap();
        println!("   Received: {}", value);
    }
    thread.join().unwrap();
    println!();

    // Example 5: Timer
    println!("5. Timer Example:");
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let mut timer = Timer::new("example_timer", move || {
        let mut c = counter_clone.lock();
        *c += 1;
        println!("   Timer fired! Count: {}", *c);
    });

    println!("   Starting periodic timer (every 100ms)...");
    timer.start_periodic(Duration::from_millis(100)).unwrap();
    Thread::sleep(Duration::from_millis(350)).unwrap();
    timer.stop().unwrap();
    println!("   Timer stopped.");
    println!();

    // Example 6: Time measurement
    println!("6. Time Measurement Example:");
    let start = osal_rs::time::Instant::now();
    Thread::sleep(Duration::from_millis(100)).unwrap();
    let elapsed = start.elapsed();
    println!("   Operation took: {}ms", elapsed.as_millis());
    println!();

    println!("All examples completed successfully!");
}
