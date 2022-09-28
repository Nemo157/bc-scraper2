use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Counter<const N: usize> {
    last: Instant,
    accumulated: Duration,
    samples: [Duration; N],
    inner_accumulated: Duration,
    inner_samples: [Duration; N],
    index: usize,
}

impl<const N: usize> Counter<N> {
    pub fn new(baseline: f64) -> Self {
        let accumulated = Duration::from_secs_f64((baseline / (N as f64)).recip());
        Self {
            last: Instant::now(),
            accumulated,
            samples: [accumulated / (N as u32); N],
            inner_accumulated: accumulated,
            inner_samples: [accumulated / (N as u32); N],
            index: 0,
        }
    }

    pub fn reset_start(&mut self) {
        self.last = Instant::now();
    }

    pub fn tick(&mut self, inner_sample: Duration) {
        self.inner_accumulated -= self.inner_samples[self.index];
        self.inner_accumulated += inner_sample;
        self.inner_samples[self.index] = inner_sample;

        let sample = std::mem::replace(&mut self.last, Instant::now()).elapsed();

        self.accumulated -= self.samples[self.index];
        self.accumulated += sample;
        self.samples[self.index] = sample;

        self.index = (self.index + 1) % N;
    }

    pub fn record<R>(&mut self, f: impl FnOnce() -> R) -> R {
        let start = Instant::now();
        let result = f();
        self.tick(start.elapsed());
        result
    }

    pub fn per_second(&self) -> f64 {
        self.accumulated.as_secs_f64().recip() * (N as f64)
    }

    pub fn inner_duration(&self) -> Duration {
        self.inner_accumulated / (N as u32)
    }
}
