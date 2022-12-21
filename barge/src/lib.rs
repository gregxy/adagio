use async_trait::async_trait;
use errs::Result;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;

pub mod messaging;
pub use config::{Config, ConfigError};

mod config;
mod inner;
mod machinery;

use crate::inner::BargeCore;
use crate::machinery::*;
use crate::messaging::*;

pub struct Barge {
    core: Arc<BargeCore>,
}

impl Barge {
    pub fn new(id: String, config: Config) -> errs::Result<Self> {
        config.validate()?;

        // TODO: construction API
        Ok(Self {
            core: Arc::new(BargeCore::new(id, config, Vec::new())),
        })
    }

    pub async fn run(&self, ct: CancellationToken) {
        let child = ct.child_token();

        select! {
            _ = ct.cancelled() => (),
            _ = crate::machinery::run(self.core.clone(), child) => (),
        }
    }
}

#[async_trait]
impl BargeService for Barge {
    // TODO: error handling (e.g., check sngle mode)
    async fn append_entries(&self, request: AppendEntriesRequest) -> Result<AppendEntriesResponse> {
        Ok(recieve_append_entries_request(self.core.clone(), request).await)
    }

    async fn request_vote(&self, request: RequestVoteRequest) -> Result<RequestVoteResponse> {
        Ok(receive_request_vote_request(self.core.clone(), request).await)
    }
}
