#[macro_use]
extern crate nom;
use nom::{digit1, eol};
use nom::types::CompleteStr;

use std::collections::BTreeMap;

#[derive(Debug,PartialEq)]
enum Action {
    Wait,
    Go,
}

#[derive(Debug,PartialEq)]
struct State {
    index : u16,
    action : Action,
    go : u16,
    wait : u16
}

#[derive(Debug,PartialEq)]
struct CompressedState {
    action : Action,
    go : u16,
    wait : u16
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
    let index_lookup =
        states
        .iter()
        .enumerate()
        .map(|(n, State { index, .. })| (n as u16, *index))
        .collect::<BTreeMap<_,_>>();
    states
        .into_iter()
        .map(|State { index: _, action, go, wait }| {
            CompressedState {
                action: action,
                go: *index_lookup.get(&go).unwrap(),
                wait: *index_lookup.get(&wait).unwrap(),
            }
        })
        .collect()
}

fn main() {
    let test_uo = CompleteStr(include_str!("../test.uo"));
    println!("{:?}", dfa(test_uo));
}
