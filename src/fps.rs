use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Counter<const N: usize> {
    accumulated: Duration,
    last: Instant,
    samples: [Duration; N],
    index: usize,
}

impl<const N: usize> Counter<N> {
    pub fn tick(&mut self) {
        let sample = std::mem::replace(&mut self.last, Instant::now()).elapsed();
        self.accumulated -= self.samples[self.index];
        self.accumulated += sample;
        self.samples[self.index] = sample;
        self.index = (self.index + 1) % N;
    }

    pub fn value(&self) -> f64 {
        self.accumulated.as_secs_f64().recip() * (N as f64)
    }
}

impl<const N: usize> Default for Counter<N> {
    fn default() -> Self {
        Self {
            accumulated: Duration::ZERO,
            last: Instant::now(),
            samples: [Duration::ZERO; N],
            index: 0,
        }
    }
}
