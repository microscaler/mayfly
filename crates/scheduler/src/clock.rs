use std::time::{Duration, Instant};

#[allow(unused)]
#[derive(Clone)]
/// Virtual clock used by the scheduler for deterministic time control.
///
/// Rather than relying on the OS clock, the scheduler advances this clock
/// when tasks sleep or when the system is idle. This makes tests run quickly
/// and ensures time-based operations are reproducible.
pub struct TickClock {
    now: Instant,
}

impl TickClock {
    #![allow(unused)]
    /// Create a new clock starting at the given instant.
    pub fn new(start: Instant) -> Self {
        Self { now: start }
    }

    /// Get the current time according to the virtual clock.
    pub fn now(&self) -> Instant {
        self.now
    }

    /// Advance the clock by `dur`.
    pub fn tick(&mut self, dur: Duration) {
        self.now += dur;
    }
}
