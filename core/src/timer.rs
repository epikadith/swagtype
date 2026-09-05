//! Lightweight timer for measuring elapsed time during games.

use std::time::Instant;

/// A simple stopwatch-style timer.
#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Start a new timer, recording the current instant.
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Elapsed time as a [`std::time::Duration`].
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    /// Elapsed time in fractional seconds.
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn timer_starts_near_zero() {
        let timer = Timer::start();
        // Immediately after creation, elapsed should be very small.
        assert!(timer.elapsed_secs() < 0.1);
    }

    #[test]
    fn timer_measures_elapsed_time() {
        let timer = Timer::start();
        thread::sleep(Duration::from_millis(50));
        let elapsed = timer.elapsed_secs();
        // Should be at least 50ms but allow generous upper bound for CI.
        assert!(elapsed >= 0.04, "elapsed was {elapsed}s, expected >= 0.04s");
        assert!(elapsed < 1.0, "elapsed was {elapsed}s, expected < 1.0s");
    }

    #[test]
    fn timer_elapsed_duration() {
        let timer = Timer::start();
        thread::sleep(Duration::from_millis(20));
        let dur = timer.elapsed();
        assert!(dur.as_millis() >= 15);
    }
}
