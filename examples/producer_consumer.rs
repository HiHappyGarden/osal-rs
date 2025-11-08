//! Producer-Consumer pattern example using OSAL primitives

use osal_rs::{Thread, Queue, time::Duration};
use std::sync::Arc;

fn main() {
    println!("=== Producer-Consumer Example ===\n");

    let queue = Arc::new(Queue::new(5));
    
    // Create multiple producers
    let mut producers = vec![];
    for id in 0..3 {
        let queue_clone = queue.clone();
        let producer = Thread::new(&format!("producer-{}", id), move || {
            for i in 0..5 {
                let value = id * 100 + i;
                queue_clone.send(value).unwrap();
                println!("Producer {} sent: {}", id, value);
                Thread::sleep(Duration::from_millis(50 + id * 20)).unwrap();
            }
            println!("Producer {} finished", id);
        });
        producers.push(producer);
    }

    // Create multiple consumers
    let mut consumers = vec![];
    for id in 0..2 {
        let queue_clone = queue.clone();
        let consumer = Thread::new(&format!("consumer-{}", id), move || {
            for _ in 0..7 {
                match queue_clone.recv_timeout(Duration::from_millis(500)) {
                    Ok(value) => {
                        println!("Consumer {} received: {}", id, value);
                        Thread::sleep(Duration::from_millis(30)).unwrap();
                    }
                    Err(_) => {
                        println!("Consumer {} timed out", id);
                        break;
                    }
                }
            }
            println!("Consumer {} finished", id);
        });
        consumers.push(consumer);
    }

    // Wait for all threads to complete
    for producer in producers {
        producer.join().unwrap();
    }

    // Give consumers time to process remaining items
    Thread::sleep(Duration::from_millis(200)).unwrap();

    for consumer in consumers {
        consumer.join().unwrap();
    }

    println!("\n=== Example completed ===");
}
