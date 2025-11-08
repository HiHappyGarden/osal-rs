# osal-rs

Operating System Abstraction Layer (OSAL) for Rust, providing a unified interface for OS primitives across POSIX systems and FreeRTOS.

## Features

- **Thread/Task Management**: Create and manage threads with a consistent API
- **Mutex**: Mutual exclusion primitives for protecting shared data
- **Semaphore**: Binary and counting semaphores for synchronization
- **Message Queue**: Thread-safe message passing
- **Timer**: One-shot and periodic timers with callbacks
- **Time**: Duration and instant types for time measurements

## Platform Support

- **POSIX** (default): Linux, macOS, Unix systems using standard POSIX APIs
- **FreeRTOS** (experimental): Real-time operating system support (placeholder implementation)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
osal-rs = "0.1.0"
```

## Usage Examples

### Threads

```rust
use osal_rs::Thread;

// Create and run a thread
let thread = Thread::new("worker", || {
    println!("Hello from thread!");
});

thread.join().unwrap();

// Sleep the current thread
Thread::sleep(Duration::from_millis(100)).unwrap();
```

### Mutex

```rust
use osal_rs::Mutex;
use std::sync::Arc;

let mutex = Arc::new(Mutex::new(0));
let mutex_clone = mutex.clone();

let thread = Thread::new("worker", move || {
    let mut guard = mutex_clone.lock();
    *guard += 1;
});

thread.join().unwrap();
assert_eq!(*mutex.lock(), 1);
```

### Semaphore

```rust
use osal_rs::{Semaphore, BinarySemaphore};

// Counting semaphore
let sem = Semaphore::new(3);
sem.wait().unwrap();  // Acquire
sem.post().unwrap();  // Release

// Binary semaphore
let bin_sem = BinarySemaphore::new(true);
bin_sem.wait().unwrap();
bin_sem.post().unwrap();
```

### Queue

```rust
use osal_rs::Queue;
use std::sync::Arc;

let queue = Arc::new(Queue::new(10));
queue.send(42).unwrap();

let value = queue.recv().unwrap();
assert_eq!(value, 42);
```

### Timer

```rust
use osal_rs::{Timer, time::Duration};
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let counter_clone = counter.clone();

let mut timer = Timer::new("periodic", move || {
    let mut c = counter_clone.lock();
    *c += 1;
});

timer.start_periodic(Duration::from_millis(100)).unwrap();
// Timer will fire every 100ms
std::thread::sleep(std::time::Duration::from_secs(1));
timer.stop().unwrap();
```

### Time

```rust
use osal_rs::time::{Duration, Instant};

let duration = Duration::from_secs(1);
let start = Instant::now();

// Do some work...

let elapsed = start.elapsed();
```

## Feature Flags

- `posix` (default): Enable POSIX implementation
- `freertos`: Enable FreeRTOS implementation (experimental)

**Note:** Only one platform feature can be enabled at a time. The library enforces this at compile-time.

To use with FreeRTOS:

```toml
[dependencies]
osal-rs = { version = "0.1.0", default-features = false, features = ["freertos"] }
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with specific features:

```bash
cargo test --features posix
```

## Design

The library uses Rust's trait system to provide a common interface across different operating systems:

- Core abstractions are defined in the main modules
- Platform-specific implementations are in `posix` and `freertos` submodules
- Feature flags enable conditional compilation for target platforms

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- [x] POSIX implementation for core primitives
- [ ] Complete FreeRTOS implementation with bindings
- [ ] Add timeout support for all blocking operations
- [ ] Add priority support for threads
- [ ] Add condition variables
- [ ] Expand platform support (Windows, Zephyr, etc.)
- [ ] Performance benchmarks
- [ ] Additional examples and documentation