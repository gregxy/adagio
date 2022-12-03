use errs::create_msg_err;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

create_msg_err!(ConfigError);

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // How often the leader should send heartbeat request.
    pub send_heartbeat_period: Duration,

    // Minimal interval of back-to-back heartbeat requests
    pub send_heartbeat_interval_min: Duration,

    // Lower bound of timeout value when waiting for a heartbeat.
    pub heartbeat_timeout_min: Duration,

    // Upper bound of timeout value when waiting for a heartbeat.
    pub heartbeat_timeout_max: Duration,

    // Lower bound of timeout value when waiting for election to complete.
    pub election_timeout_min: Duration,

    // Upper bound of timeout value when waiting for election to complete.
    pub election_timeout_max: Duration,

    // URIs of peers.
    pub peer_uris: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            send_heartbeat_period: Duration::from_millis(100),
            send_heartbeat_interval_min: Duration::from_millis(500),
            heartbeat_timeout_min: Duration::from_millis(900),
            heartbeat_timeout_max: Duration::from_millis(1100),
            election_timeout_min: Duration::from_millis(900),
            election_timeout_max: Duration::from_millis(1100),
            peer_uris: Vec::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errs: Vec<String> = Vec::new();

        if self.send_heartbeat_period.as_micros() == 0 {
            errs.push(String::from("send_heartbeat_period cannot be 0"));
        }

        if self.send_heartbeat_interval_min.as_micros() == 0 {
            errs.push(String::from("send_heartbeat_interval_min cannot be 0"));
        }

        if self.send_heartbeat_period <= self.send_heartbeat_interval_min {
            errs.push(format!(
                "Expect send_heartbeat_period ({:?}) > send_heartbeat_interval_min ({:?})",
                self.send_heartbeat_period, self.send_heartbeat_interval_min
            ));
        }

        if self.heartbeat_timeout_min <= self.send_heartbeat_period {
            errs.push(format!(
                "Expect heartbeat_timeout_min ({:?}) > heartbeat_period ({:?})",
                self.heartbeat_timeout_min, self.send_heartbeat_period
            ));
        }

        if self.heartbeat_timeout_min <= self.send_heartbeat_period {
            errs.push(format!(
                "Expect heartbeat_timeout_min ({:?}) > send_heartbeat_period ({:?})",
                self.heartbeat_timeout_min, self.send_heartbeat_period
            ));
        }

        if self.heartbeat_timeout_max <= self.heartbeat_timeout_min {
            errs.push(format!(
                "Expect heartbeat_timeout_max ({:?}) > heartbeat_timeout_min ({:?})",
                self.heartbeat_timeout_max, self.heartbeat_timeout_min
            ));
        }

        if self.election_timeout_max <= self.election_timeout_min {
            errs.push(format!(
                "Expect election_timeout_max ({:?}) > election_timeout_min ({:?})",
                self.election_timeout_max, self.election_timeout_min
            ));
        }

        if self.peer_uris.len() % 2 != 0 {
            errs.push(format!(
                "Expect even number of peers (= {})",
                self.peer_uris.len()
            ));
        }

        if errs.is_empty() {
            return Ok(());
        }

        Err(ConfigError::new(errs.join("; ")))
    }

    pub fn pick_election_timeout(&self) -> Duration {
        Duration::from_micros(rand::thread_rng().gen_range(
            self.election_timeout_min.as_micros() as u64
                ..=self.election_timeout_max.as_micros() as u64,
        ))
    }

    pub fn pick_heartbeat_timeout(&self) -> Duration {
        Duration::from_micros(rand::thread_rng().gen_range(
            self.heartbeat_timeout_min.as_micros() as u64
                ..=self.heartbeat_timeout_max.as_micros() as u64,
        ))
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    fn test_instance() -> Config {
        Config {
            send_heartbeat_period: Duration::from_millis(400),
            send_heartbeat_interval_min: Duration::from_millis(200),
            heartbeat_timeout_min: Duration::from_millis(450),
            heartbeat_timeout_max: Duration::from_millis(500),
            election_timeout_min: Duration::from_millis(420),
            election_timeout_max: Duration::from_millis(460),
        }
    }

    #[test]
    fn validate_test_instance() {
        assert!(test_instance().validate().is_ok());
    }

    #[test]
    fn zero_heartbeat_period() {
        let mut c = test_instance();
        c.send_heartbeat_period = Duration::from_millis(0);
        assert!(c.validate().is_err());
    }

    #[test]
    fn bad_heartbeat_timeout_min() {
        let mut c = test_instance();
        c.heartbeat_timeout_min = c.send_heartbeat_period;
        assert!(c.validate().is_err());
    }

    #[test]
    fn multiple_timeout_error() {
        let mut c = test_instance();

        c.heartbeat_timeout_min = c.heartbeat_timeout_max + Duration::from_millis(1);
        c.election_timeout_min = c.election_timeout_max;

        assert!(c.validate().is_err());
    }
}
