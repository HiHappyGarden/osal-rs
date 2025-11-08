//! Time and duration types

use std::time::Duration as StdDuration;

/// A duration of time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    inner: StdDuration,
}

impl Duration {
    /// Creates a new duration from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self {
            inner: StdDuration::from_secs(secs),
        }
    }

    /// Creates a new duration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self {
            inner: StdDuration::from_millis(millis),
        }
    }

    /// Creates a new duration from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self {
            inner: StdDuration::from_micros(micros),
        }
    }

    /// Creates a new duration from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self {
            inner: StdDuration::from_nanos(nanos),
        }
    }

    /// Returns the total number of whole seconds
    pub fn as_secs(&self) -> u64 {
        self.inner.as_secs()
    }

    /// Returns the total number of milliseconds
    pub fn as_millis(&self) -> u128 {
        self.inner.as_millis()
    }

    /// Returns the total number of microseconds
    pub fn as_micros(&self) -> u128 {
        self.inner.as_micros()
    }

    /// Returns the total number of nanoseconds
    pub fn as_nanos(&self) -> u128 {
        self.inner.as_nanos()
    }
}

impl From<StdDuration> for Duration {
    fn from(d: StdDuration) -> Self {
        Self { inner: d }
    }
}

impl From<Duration> for StdDuration {
    fn from(d: Duration) -> Self {
        d.inner
    }
}

/// A measurement of a monotonically increasing clock
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    inner: std::time::Instant,
}

impl Instant {
    /// Returns the current instant
    pub fn now() -> Self {
        Self {
            inner: std::time::Instant::now(),
        }
    }

    /// Returns the amount of time elapsed since this instant
    pub fn elapsed(&self) -> Duration {
        Duration::from(self.inner.elapsed())
    }

    /// Returns the duration between this instant and another
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration::from(self.inner.duration_since(earlier.inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_creation() {
        let d1 = Duration::from_secs(1);
        assert_eq!(d1.as_secs(), 1);
        assert_eq!(d1.as_millis(), 1000);

        let d2 = Duration::from_millis(500);
        assert_eq!(d2.as_millis(), 500);
        assert_eq!(d2.as_micros(), 500_000);
    }

    #[test]
    fn test_instant() {
        let start = Instant::now();
        std::thread::sleep(StdDuration::from_millis(10));
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_duration_ordering() {
        let d1 = Duration::from_millis(100);
        let d2 = Duration::from_millis(200);
        assert!(d1 < d2);
        assert!(d2 > d1);
    }
}
