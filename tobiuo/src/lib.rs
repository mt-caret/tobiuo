#[macro_use]
extern crate nom;
use nom::types::CompleteStr;
use nom::{digit1, eol};

use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Action {
    Wait,
    Go,
}

#[derive(Debug, PartialEq)]
pub struct State {
    index: u16,
    action: Action,
    go: u16,
    wait: u16,
}

#[derive(Debug, PartialEq)]
pub struct CompressedState {
    action: Action,
    go: u16,
    wait: u16,
}

named!(action<CompleteStr,Action>,
    do_parse!(
        wg: alt!(char!('w') | char!('g')) >>
        (if wg == 'w' { Action::Wait } else { Action::Go })
    )
);

fn parse_number(input: CompleteStr) -> Result<u16, std::num::ParseIntError> {
    input.parse::<u16>()
}

named!(state_number<CompleteStr,u16>, map_res!(digit1, parse_number));

named!(state<CompleteStr,State>,
    do_parse!(
        index: state_number >>
        char!(':') >>
        action: action >>
        char!(',') >>
        go: state_number >>
        char!(',') >>
        wait: state_number >>
        (State { index, action, go, wait })
    )
);

named!(dfa<CompleteStr,Vec<State>>,
       many1!(terminated!(state, alt!(eof!() | eol))));

fn pack(states: Vec<State>) -> Vec<CompressedState> {
    let index_lookup = states
        .iter()
        .enumerate()
        .map(|(n, State { index, .. })| (*index, n as u16))
        .collect::<BTreeMap<_, _>>();
    states
        .into_iter()
        .map(
            |State {
                 index: _,
                 action,
                 go,
                 wait,
             }| CompressedState {
                action: action,
                go: *index_lookup.get(&go).unwrap(),
                wait: *index_lookup.get(&wait).unwrap(),
            },
        )
        .collect()
}

pub fn parse_dfa(input: &str) -> Option<Vec<CompressedState>> {
    dfa(CompleteStr(input)).map(|(_, result)| pack(result)).ok()
}

fn next_state(state: &CompressedState, opponent_action: &Action) -> u16 {
    match opponent_action {
        Action::Go => state.go,
        Action::Wait => state.wait,
    }
}

pub fn simulate(
    player1: &Vec<CompressedState>,
    player2: &Vec<CompressedState>,
    turns: u8,
) -> (i64, i64) {
    let mut scores = (0, 0);
    let mut state: (u16, u16) = (0, 0);
    for _ in 0..turns {
        let player1_state = &player1[state.0 as usize]; // TODO: check is unsafe is faster
        let player2_state = &player2[state.1 as usize];
        let diff = match (&player1_state.action, &player2_state.action) {
            (Action::Go, Action::Go) => (-1, -1),
            (Action::Go, Action::Wait) => (7, -3),
            (Action::Wait, Action::Go) => (-3, 7),
            (Action::Wait, Action::Wait) => (1, 1),
        };
        scores.0 += diff.0;
        scores.1 += diff.1;
        state.0 = next_state(player1_state, &player2_state.action);
        state.1 = next_state(player2_state, &player1_state.action);
    }
    scores
}

pub fn simulate_nvn(states: &Vec<(String, Vec<CompressedState>, u16)>, turns: u8) -> Vec<i64> {
    // points[player1][player2] corresponds to the score of player1 when playing against player2.
    let mut points = vec![vec![0; states.len()]; states.len()];
    for player1 in 0..states.len() {
        for player2 in player1..states.len() {
            let scores = simulate(&states[player1].1, &states[player2].1, turns);
            points[player1][player2] = scores.0;
            points[player2][player1] = scores.1;
        }
    }
    let mut ret = vec![0; states.len()];
    for player in 0..states.len() {
        for opponent in 0..states.len() {
            let mut number_of_opponents = states[opponent].2 as i64;
            if player == opponent {
                number_of_opponents -= 1;
            }
            ret[player] += points[player][opponent] * number_of_opponents;
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
