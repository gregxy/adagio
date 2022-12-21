use parking_lot::Mutex;
use tokio::time::Instant;

use crate::config::Config;
use crate::messaging::BargeService;

#[derive(strum::Display, Debug, PartialEq)]
pub(crate) enum Role {
    Follower,
    Candidate,
    Leader,
}

pub(crate) struct State {
    pub role: Role,
    pub term: u64,
    pub deadline: Instant,
    pub vote_count: u32,
    pub vote_threshold: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            role: Role::Follower,
            term: 0,
            deadline: Instant::now(),
            vote_count: 0,
            vote_threshold: 0,
        }
    }
}

pub(crate) struct BargeCore {
    pub id: String,
    pub state: Mutex<State>,
    pub config: Config,
    pub peers: Vec<Box<dyn BargeService + Send + Sync>>,
}

unsafe impl Send for BargeCore {}
unsafe impl Sync for BargeCore {}

impl BargeCore {
    pub(crate) fn new(
        id: String,
        config: Config,
        peers: Vec<Box<dyn BargeService + Send + Sync>>,
    ) -> Self {
        let state = State {
            vote_threshold: (config.peer_uris.len() as u32) / 2,
            ..Default::default()
        };

        Self {
            id,
            state: Mutex::new(state),
            config,
            peers,
        }
    }
}
