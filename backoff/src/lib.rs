use std::cmp;
use std::time::Duration;

use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Config {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f32,
    pub jitter: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(120),
            multiplier: 1.6,
            jitter: 0.2,
        }
    }
}

pub struct Backoff {
    config: Config,
    jitter_range: Uniform<f32>,
    rng: ThreadRng,
    without_jitter: Duration,
    with_jitter: Duration,
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Backoff {
    pub fn new(config: Config) -> Self {
        Self {
            jitter_range: Uniform::new_inclusive(1.0 - config.jitter, 1.0 + config.jitter),
            rng: ThreadRng::default(),
            without_jitter: config.initial_delay,
            with_jitter: config.initial_delay,
            config,
        }
    }

    pub fn current_backoff(&self) -> Duration {
        self.with_jitter
    }

    pub fn next_backoff(&mut self) -> Duration {
        self.without_jitter = cmp::min(
            self.without_jitter.mul_f32(self.config.multiplier),
            self.config.max_delay,
        );
        self.with_jitter = self
            .without_jitter
            .mul_f32(self.jitter_range.sample(&mut self.rng));

        self.with_jitter
    }

    pub fn rest(&mut self) {
        self.without_jitter = self.config.initial_delay;
        self.with_jitter = self.config.initial_delay;
    }
}
