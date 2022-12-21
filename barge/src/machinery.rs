use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::time::{sleep, Instant};
use tokio_util::sync::CancellationToken;

use crate::inner::{BargeCore, Role};
use crate::messaging::{
    AppendEntriesRequest, AppendEntriesResponse, RequestVoteRequest, RequestVoteResponse,
};

static EPSILON: Duration = Duration::from_micros(2);

// TODO: single mode
pub(crate) async fn run(abc: Arc<BargeCore>, ct: CancellationToken) {
    let mut sleeptime = abc.config.pick_heartbeat_timeout();
    {
        let mut state = abc.state.lock();
        state.deadline = Instant::now() + sleeptime - EPSILON;
    }

    loop {
        select! {
            _ = ct.cancelled() => return,
            _ = sleep(sleeptime) => sleeptime = act(abc.clone()),
        }
    }
}

fn act(abc: Arc<BargeCore>) -> Duration {
    let mut to_trigger_election = false;
    let mut to_send_heartbeat = false;
    let term: u64;
    let sleeptime: Duration;

    {
        let mut state = abc.state.lock();
        let now = Instant::now();
        if now < state.deadline {
            return state.deadline - now + EPSILON;
        }

        // exceeded deadline
        match state.role {
            Role::Follower | Role::Candidate => {
                to_trigger_election = true;

                state.role = Role::Candidate;
                state.term += 1;
                state.vote_count = 0;

                term = state.term;
                sleeptime = abc.config.pick_election_timeout();
                state.deadline = Instant::now() + sleeptime - EPSILON;
            }
            Role::Leader => {
                to_send_heartbeat = true;

                term = state.term;
                sleeptime = abc.config.send_heartbeat_period;
                state.deadline = Instant::now() + sleeptime - EPSILON;
            }
        }
    }

    if to_trigger_election {
        trigger_election(abc.clone(), term);
    }

    if to_send_heartbeat {
        send_heartbeat(abc, term);
    }

    sleeptime
}

fn trigger_election(abc: Arc<BargeCore>, term: u64) {
    for idx in 0..abc.peers.len() {
        tokio::spawn(request_vote(abc.clone(), term, idx));
    }
}

async fn request_vote(abc: Arc<BargeCore>, term: u64, index: usize) {
    let request = RequestVoteRequest {
        term,
        candidate_id: abc.id.clone(),
    };

    let result = abc.peers[index].request_vote(request).await;

    // TODO: handle error and retry
    if result.is_err() {
        return;
    }

    let response = result.unwrap();
    receive_request_vote_response(abc, response);
}

fn receive_request_vote_response(abc: Arc<BargeCore>, response: RequestVoteResponse) {
    let mut won = false;
    let mut term = 0;

    {
        let mut state = abc.state.lock();

        if state.role != Role::Candidate {
            return;
        }

        if response.granted {
            state.vote_count += 1;
            if state.vote_count >= state.vote_threshold {
                won = true;
                term = state.term;
                state.role = Role::Leader;
                state.deadline = Instant::now() + abc.config.send_heartbeat_period;
            }
        } else if response.term > state.term {
            state.role = Role::Follower;
            state.term = response.term;
            state.deadline = Instant::now() + abc.config.pick_heartbeat_timeout();
        }
    }

    if won {
        send_heartbeat(abc, term);
    }
}

fn send_heartbeat(abc: Arc<BargeCore>, term: u64) {
    for idx in 0..abc.peers.len() {
        tokio::spawn(send_heartbeat_request(abc.clone(), term, idx));
    }
}

async fn send_heartbeat_request(abc: Arc<BargeCore>, term: u64, index: usize) {
    let request = AppendEntriesRequest {
        term,
        leader_id: abc.id.clone(),
    };

    let result = abc.peers[index].append_entries(request).await;

    if result.is_err() {
        return;
    }

    let response = result.unwrap();

    recieve_append_entries_response(abc, response);
}

fn recieve_append_entries_response(abc: Arc<BargeCore>, response: AppendEntriesResponse) {
    {
        let mut state = abc.state.lock();

        if state.role != Role::Leader {
            return;
        }

        if !response.success && response.term >= state.term {
            state.role = Role::Follower;
            state.term = response.term;
            state.deadline = Instant::now() + abc.config.pick_heartbeat_timeout();
        }
    }
}

pub(crate) async fn receive_request_vote_request(
    abc: Arc<BargeCore>,
    request: RequestVoteRequest,
) -> RequestVoteResponse {
    let mut response = RequestVoteResponse::default();
    {
        let mut state = abc.state.lock();
        if request.term > state.term {
            response.granted = true;
            state.role = Role::Follower;
            state.term = request.term;
            state.deadline = Instant::now() + abc.config.pick_election_timeout();
        } else {
            response.granted = false;
            response.term = state.term;
        }
    }

    response
}

pub(crate) async fn recieve_append_entries_request(
    abc: Arc<BargeCore>,
    request: AppendEntriesRequest,
) -> AppendEntriesResponse {
    let mut response = AppendEntriesResponse::default();
    {
        let mut state = abc.state.lock();

        if request.term < state.term {
            response.term = state.term;
            response.success = false;

            return response;
        }

        if request.term == state.term && state.role == Role::Leader {
            state.role = Role::Follower;
            state.deadline = Instant::now() + abc.config.pick_election_timeout();
        } else {
            state.deadline = Instant::now() + abc.config.pick_heartbeat_timeout();
        }
    }

    response
}
