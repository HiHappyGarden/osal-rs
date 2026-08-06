# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

All crates in the workspace (`osal-rs`, `osal-rs-serde`, `osal-rs-serde-derive`,
`osal-rs-build`, `osal-rs-tests`) share a single version number and are released
together.

## [Unreleased]

### Documentation

- The trait-level doc examples are compiled and run as doctests instead of being
  marked `ignore`. They were stale pseudo-code calling APIs that no longer exist
  (`Semaphore::new_with_count`, `Thread::current()`, the four-argument
  `Thread::new`), and several would not have compiled. `--features posix` now
  runs 283 doctests instead of 195.
- `QueueStreamed`'s examples are skipped under the `serde` feature, which
  replaces the `Serialize`/`Deserialize` traits they implement by hand.

### Fixed

- `EventGroupFn::set` is documented as returning the bits *after* the update.
  Both backends return the new value (POSIX reads the mask after the OR,
  FreeRTOS returns `xEventGroupSetBits`'s result); the trait documented the
  value from before the call.

## [1.2.0] - 2026-08-05

Timer ownership and thread joining now behave the same on both backends. See
[Breaking Changes](README.md#breaking-changes) in the README for migration notes.

### Changed

- **BREAKING** — `Timer` clones now share one underlying timer, destroyed exactly
  once when the last handle is dropped. On FreeRTOS a clone previously copied the
  raw handle, so `delete()` through one handle left the others dangling and the
  *first* clone dropped destroyed the timer for everyone; on POSIX the timer and
  its background thread leaked unless `delete()` was called explicitly. Both
  backends now keep the timer in shared state (`TimerShared`), with FreeRTOS timer
  IDs used to reach it from the daemon callback (`vTimerSetTimerID` added to the
  FFI bindings).
- **BREAKING** — a timer callback's return value is handed to the next firing on
  FreeRTOS too; it used to be discarded and the original parameter re-passed.
- **BREAKING** — the handle passed to a timer callback is now non-owning. It
  previously owned the timer and destroyed it on return, so an auto-reload
  FreeRTOS timer deleted itself at its first firing.
- **BREAKING** — `ThreadFn::join` blocks until the thread's closure returns on
  FreeRTOS, matching the trait documentation and the POSIX (`pthread_join`)
  behaviour. It used to call `vTaskDelete` on the target task, killing it
  mid-work — and, since the task wrappers already delete themselves once their
  callback returns, that was a use-after-free for threads that had already
  finished. Every spawned thread now carries an exit latch, so `join` waits for
  the thread and collects its return value on both backends. Code that used
  `join()` to *kill* a FreeRTOS task must call `delete()` instead.

### Fixed

- Null-handle checks added across the FreeRTOS synchronization primitives
  (event group, mutex, queue, semaphore, thread, timer), so operating on a
  deleted or never-created object returns an error instead of dereferencing a
  null handle.
- POSIX `System::elapsed` resolves the lazily-captured start epoch *before*
  sampling the monotonic clock. Sampling first subtracted a later epoch from an
  earlier reading, saturating the first elapsed time to zero.

### Added

- Extensive test coverage on both backends: error/failure paths
  (`std_error_paths_tests.rs`, `error_paths_tests.rs`), system lifecycle,
  trait-level tests, logging, async, timers and utils — roughly 6,000 lines of
  new tests, including deletion-guard failure paths and a test verifying that a
  timer callback dropping the last handle tears the timer down safely.
- `osal-rs-serde` as a dev-dependency of `osal-rs`, so the crate's own tests can
  exercise the `Serialize`/`Deserialize` impls on `Bytes<SIZE>` without changing
  what downstream users link.
- CHANGELOG.md 

## [1.1.0] - 2026-08-02

### Changed

- **BREAKING** — `EventGroupFn::wait` and `wait_with_to_tick` take an explicit
  `wait_for_all_bits: bool`, mirroring FreeRTOS's `xEventGroupWaitBits`:

  ```rust
  let bits = event_group.wait(mask, true, timeout_ticks);   // AND
  let bits = event_group.wait(mask, false, timeout_ticks);  // OR
  ```

  The backends previously disagreed in silence: POSIX only implemented AND-wait,
  while FreeRTOS hardcoded OR-wait (`xWaitForAllBits = pdFALSE`) regardless of
  what the trait documented. Waiting on multiple independent bits could hang
  forever on POSIX while working by accident on FreeRTOS.
- Internal refactor of `MutexFn`/`Mutex` locking and of the `thread_extract_param!`
  macro and `Bytes::fill_str` to `let ... else`; no behaviour change.
- `osal-rs-serde-derive` error reporting consolidated into a single
  `syn::Error::new_spanned(...).to_compile_error()` path (~100 lines removed).

### Fixed

- The POSIX scheduler run loop no longer busy-waits: a delay was added to the
  loop body.

## [1.0.4] - 2026-07-30

### Changed

- `System::get_current_time_us` renamed to `System::get_current_time`, and
  `System::get_ms_from_tick` to `System::get_from_tick`. Deprecated aliases
  (`get_current_time_ms`, `get_ms_from_tick`) are kept for source compatibility
  and marked `#[deprecated(since = "1.0.4")]`.
- `get_from_tick` reimplemented on top of `to_ticks` instead of duplicating the
  conversion.
- `Display` implementations simplified and the `Write` trait impl for `Bytes`
  reworked.
- `osal-rs-serde-derive` moved to `syn` 3 (from 2.0); `quote` and `proc-macro2`
  requirements relaxed to `1`.

### Removed

- Unused `clock_nanosleep` declaration from the POSIX FFI bindings.

### Documentation

- CI test badge (and branch badge) added to the README.

## [1.0.3] - 2026-07-25

### Fixed

- The `posix` feature documentation no longer claims a static library must be
  linked — the POSIX backend links against the system libc only.

### Changed

- The test workflow now declares `pull-requests` permissions, so it can run and
  report on pull requests.

## [1.0.2] - 2026-07-23

### Changed

- `[package.metadata.docs.rs]` added to `osal-rs`, so docs.rs builds the crate
  with `--no-default-features --features posix,async,serde` on
  `x86_64-unknown-linux-gnu` — necessary since 1.0.0 removed the default
  backend, leaving docs.rs with no buildable feature set of its own.

## [1.0.1] - 2026-07-22

### Removed

- The POSIX C porting layer (`osal-rs-porting/posix/`). The POSIX backend now
  binds directly to libc from Rust, so no C sources are compiled and no static
  library is produced for it; the build script and `osal-rs-build` were
  simplified accordingly (the porting layer is still required for FreeRTOS).

### Fixed

- License header placement in the code emitted by `osal-rs-build`.
- POSIX FFI and thread fallout from dropping the C shim.

## [1.0.0] - 2026-07-22

First stable release. The POSIX backend is complete and tested, making it
possible to build, run and test OSAL-RS applications on a host without embedded
hardware.

### Added

- **POSIX backend**, fully implemented on glibc/Linux: threads with a thread
  registry, metadata and state tracking, signal handling and notifications;
  POSIX timers; event groups and semaphores built on `pthread` condition
  variables with `CLOCK_MONOTONIC` support; mutexes with attributes; queues;
  system services. Includes hand-written FFI bindings (~680 lines) and
  build-time type generation.
- **`async` feature** (experimental, backend-agnostic): `async_executor`
  (executor + waker) and `async_primitives` (async `Mutex`, `Queue`,
  `Semaphore`, `WakerSlot`).
- **`real_time` feature**: schedules spawned POSIX threads with `SCHED_FIFO`
  instead of inheriting the creating thread's policy and priority.
- `is_null()` on the synchronization primitives, for checking a handle without
  touching the underlying object.
- `ThreadState` and `ThreadMetadata` in the public trait surface, plus thread
  state tracking.
- FreeRTOS critical-section functions in the porting layer and FFI bindings.
- A host-side test suite (`osal-rs/tests/std_*.rs`) covering threads, timers,
  mutexes, queues, semaphores, event groups, system, utils, duration and async,
  alongside an expanded FreeRTOS suite in `osal-rs-tests`.
- GitHub Actions workflow running the test suite with the POSIX feature.

### Changed

- **BREAKING** — no default backend. `default = ["freertos"]` was removed:
  exactly one of `freertos` / `posix` must be enabled explicitly, or the build
  fails with a `compile_error!`. The features now also forward to
  `osal-rs-build` (`freertos`/`posix`), which is pulled in with
  `default-features = false`.
- **BREAKING** — `System::get_us_from_tick` renamed to `get_ms_from_tick`.
- **BREAKING** — critical-section methods renamed for consistency across the
  trait and both backends.
- **BREAKING** — `Semaphore` is initialized with `UBaseType::MAX` as its count,
  for consistency between backends.
- Thread-related types reorganized; unused definitions dropped.
- POSIX start time initialization moved from `OnceLock` to `pthread_once`.
- `#[inline]` annotations added across hot paths.
- Workspace-internal dependencies bumped to the `1.0` line.

### Removed

- **BREAKING** — `System::get_state` and its tests.
- **BREAKING** — the `alloc` feature dependency; `POSIXAllocator`.
- The `osal_rs_use_sched_fifo` function, replaced by the `real_time` feature
  flag.

### Fixed

- `get_all_thread` no longer double-counts registered threads.
- POSIX monotonic clock and `nanosleep`-based sleeping.
- Waker pointer casting for `Arc<Semaphore>` in the async executor.
- `Bytes` methods made consistent (`from_str` / `from_as_sync_str`).

### Documentation

- Doc examples throughout the crate are now compiled and run as doctests
  (the `ignore` attributes were removed), and examples were added for most
  components.
- README rewritten around backend selection, feature flags and POSIX support.

[Unreleased]: https://github.com/HiHappyGarden/osal-rs/compare/1.2.0...HEAD
[1.2.0]: https://github.com/HiHappyGarden/osal-rs/compare/1.1.0...1.2.0
[1.1.0]: https://github.com/HiHappyGarden/osal-rs/compare/1.0.4...1.1.0
[1.0.4]: https://github.com/HiHappyGarden/osal-rs/compare/1.0.3...1.0.4
[1.0.3]: https://github.com/HiHappyGarden/osal-rs/compare/1.0.2...1.0.3
[1.0.2]: https://github.com/HiHappyGarden/osal-rs/compare/1.0.1...1.0.2
[1.0.1]: https://github.com/HiHappyGarden/osal-rs/compare/1.0.0...1.0.1
[1.0.0]: https://github.com/HiHappyGarden/osal-rs/compare/0.5.1...1.0.0
