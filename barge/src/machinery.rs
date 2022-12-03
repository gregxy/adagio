use std::sync::Arc;
use tokio::time::sleep;

use crate::inner::{BargeCore, Role};
use crate::messaging::{
    AppendEntriesRequest, AppendEntriesResponse, RequestVoteRequest, RequestVoteResponse,
};

pub(crate) async fn wait_for_heartbeat(abc: Arc<BargeCore>, tick: u64) {
    sleep(abc.config.pick_heartbeat_timeout()).await;

    let mut trigger_election: bool = false;
    let mut expecting_tick: u64 = 0;
    {
        let mut state = abc.state.lock();

        if state.tick != tick || state.role != Role::Follower {
            return;
        }

        state.tick += 1;
        state.role = Role::Candidate;
        state.term += 1;
        state.vote_count = 0;
        expecting_tick = state.tick;
        trigger_election = true;
    }

    if !trigger_election {
        return;
    }

    tokio::spawn(wait_for_election(abc.clone(), tick + 1));

    // TODO: send vote requests
}

async fn wait_for_election(abc: Arc<BargeCore>, tick: u64) {
    let mut expect_tick: u64 = tick;

    loop {
        sleep(abc.config.pick_election_timeout()).await;

        let mut trigger_election_again: bool = false;
        {
            let mut state = abc.state.lock();

            if state.tick != expect_tick || state.role != Role::Candidate {
                return;
            }

            state.tick += 1;
            expect_tick += 1;
            state.vote_count = 0;
            trigger_election_again = true;
        }

        if !trigger_election_again {
            return;
        }

        // TODO: send vote requests
    }
}

#[derive(PartialEq)]
enum ElectionResult {
    Unknown,
    Win,
    Loose,
}

pub(crate) async fn receive_request_vote_response(
    abc: Arc<BargeCore>,
    response: RequestVoteResponse,
) {
    let mut result = ElectionResult::Unknown;
    let mut expecting_tick: u64 = 0;

    {
        let mut state = abc.state.lock();

        if state.role != Role::Candidate {
            return;
        }

        if response.granted {
            state.vote_count += 1;
            if state.vote_count >= state.vote_threshold {
                result = ElectionResult::Win;
            }
        } else {
            if response.term > state.term {
                result = ElectionResult::Loose;
                state.role = Role::Candidate;
                state.term = response.term;
                state.tick += 1;
                expecting_tick = state.tick;
            }
        }
    }

    if result == ElectionResult::Loose {
        tokio::spawn(wait_for_election(abc.clone(), expecting_tick));
    }
    // TODO: send heartbeat if won
}

pub(crate) async fn receive_request_vote_request(
    abc: Arc<BargeCore>,
    request: RequestVoteRequest,
) -> RequestVoteResponse {
    let mut response = RequestVoteResponse::default();
    let mut expecting_tick: u64 = 0;
    {
        let mut state = abc.state.lock();
        if request.term > state.term {
            response.granted = true;
            state.role = Role::Follower;
            state.term = request.term;
            state.tick += 1;
            expecting_tick = state.tick;
        } else {
            response.granted = false;
            response.term = state.term;
        }
    }

    if response.granted {
        tokio::spawn(wait_for_election(abc.clone(), expecting_tick));
    }

    return response;
}

pub(crate) async fn recieve_append_entries_request(
    abc: Arc<BargeCore>,
    request: AppendEntriesRequest,
) -> AppendEntriesResponse {
    let mut response = AppendEntriesResponse::default();
    let mut expecting_tick: u64 = 0;
    let mut trigger_election = false;
    let mut need_heartbeat = false;
    {
        let mut state = abc.state.lock();

        if request.term < state.term {
            response.term = state.term;
            response.success = false;

            return response;
        }

        if request.term == state.term && state.role == Role::Leader {
            response.term = state.term;
            response.success = false;

            state.role = Role::Candidate;
            state.term += 1;
            state.tick += 1;
            state.vote_count = 0;
            expecting_tick = state.tick;
            trigger_election = true;
        } else {
            state.term = request.term;
            state.tick += 1;
            expecting_tick = state.tick;
            need_heartbeat = true;
        }
    }

    if trigger_election {
        // TODO: Send vote request;
        tokio::spawn(wait_for_election(abc.clone(), expecting_tick));
    } else if need_heartbeat {
        tokio::spawn(wait_for_heartbeat(abc.clone(), expecting_tick));
    }

    return response;
}

pub(crate) async fn recieve_append_entries_response(
    abc: Arc<BargeCore>,
    response: AppendEntriesResponse,
) {
    let mut expecting_tick: u64 = 0;
    let mut need_heartbeat = false;

    {
        let mut state = abc.state.lock();

        if state.role != Role::Leader {
            return;
        }

        if response.success == false {
            if response.term > state.term {
                state.role = Role::Follower;
                state.tick += 1;
                expecting_tick = state.tick;
                state.term = response.term;
                need_heartbeat = true;
            }
        }
    }

    if need_heartbeat {
        tokio::spawn(wait_for_heartbeat(abc.clone(), expecting_tick));
    }
}
