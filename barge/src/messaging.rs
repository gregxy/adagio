use async_trait::async_trait;
use errs::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,

    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub leader_id: String,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RequestVoteRequest {
    pub term: u64,
    pub candidate_id: String,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub granted: bool,
}

#[async_trait]
pub trait BargeService {
    async fn append_entries(&self, request: AppendEntriesRequest) -> Result<AppendEntriesResponse>;

    async fn request_vote(&self, request: RequestVoteRequest) -> Result<RequestVoteResponse>;
}
