pub mod messaging;
pub use config::{Config, ConfigError};

mod config;
mod inner;
mod machinery;

use crate::inner::BargeCore;
use crate::machinery::*;

use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Barge {
    core: Arc<BargeCore>,
}

impl Barge {
    pub fn new(config: Config) -> errs::Result<Self> {
        config.validate()?;

        Ok(Self {
            core: Arc::new(BargeCore::new(config)),
        })
    }

    pub async fn run(&self, mut cancel_ch: broadcast::Receiver<()>) {
        tokio::spawn(wait_for_heartbeat(self.core.clone(), 0));

        cancel_ch.recv().await;
    }
}
