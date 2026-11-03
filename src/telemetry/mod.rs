use std::time::Instant;

pub struct LatencyStopwatch {
    start: Instant,
}

impl LatencyStopwatch {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_micros(&self) -> f64 {
        self.start.elapsed().as_nanos() as f64 / 1_000.0
    }

    pub fn elapsed_millis(&self) -> f64 {
        self.start.elapsed().as_nanos() as f64 / 1_000_000.0
    }
}
