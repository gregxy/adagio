use parking_lot::Mutex;

use crate::config::Config;

#[derive(strum::Display, Debug, PartialEq)]
pub(crate) enum Role {
    Follower,
    Candidate,
    Leader,
}

pub(crate) struct State {
    pub role: Role,
    pub tick: u64,
    pub term: u64,
    pub vote_count: u32,
    pub vote_threshold: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            role: Role::Follower,
            tick: 0,
            term: 0,
            vote_count: 0,
            vote_threshold: 0,
        }
    }
}

pub(crate) struct BargeCore {
    pub state: Mutex<State>,
    pub config: Config,
}

unsafe impl Send for BargeCore {}
unsafe impl Sync for BargeCore {}

impl BargeCore {
    pub(crate) fn new(config: Config) -> Self {
        let mut state = State::default();
        state.vote_threshold = (config.peer_uris.len() as u32) / 2;

        Self {
            state: Mutex::new(State::default()),
            config,
        }
    }
}
