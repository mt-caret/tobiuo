#[macro_use]
extern crate nom;
use nom::types::CompleteStr;
use nom::{digit1, eol};

#[macro_use]
extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;

extern crate serde_json;
use serde_json::Value;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, PartialEq)]
enum Action {
    Wait,
    Go,
}

#[derive(Debug, PartialEq)]
struct State {
    index: u16,
    action: Action,
    go: u16,
    wait: u16,
}

#[derive(Debug, PartialEq)]
struct CompressedState {
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

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const USAGE: &'static str = "
tobiuo - yet another osakana simulator

Usage:
  tobiuo 1v1 <player1-file> <player2-file> [--turns=<turns> | --all-turns]
  tobiuo nvn <config-file> [--turns=<turns> | --all-turns]
  tobiuo (-h | --help)
  tobiuo --version

Options:
  -h --help        Show this.
  --version        Show version.
  --turns=<turns>  Number of turns (between 0 and 255) [default: 100].
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_turns: u8,
    flag_all_turns: bool,
    flag_version: bool,
    arg_player1_file: String,
    arg_player2_file: String,
    arg_config_file: String,
    cmd_1v1: bool,
    cmd_nvn: bool,
}

fn read_file(filename: &str) -> String {
    let mut contents = String::new();

    File::open(filename)
        .expect(&format!("Error opening file {}", filename))
        .read_to_string(&mut contents)
        .expect(&format!("Error reading file {}", filename));

    contents
}

fn load_dfa(filename: &str) -> Vec<CompressedState> {
    let contents = read_file(filename);

    let (_, dfa) =
        dfa(CompleteStr(&contents)).expect(&format!("Error parsing readingfile {}", filename));

    pack(dfa)
}

fn next_state(state: &CompressedState, opponent_action: &Action) -> u16 {
    match opponent_action {
        Action::Go => state.go,
        Action::Wait => state.wait,
    }
}

fn simulate(
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

fn run(player1: &Vec<CompressedState>, player2: &Vec<CompressedState>, turns: u8) {
    println!("{} turns:", turns);
    let scores = simulate(player1, player2, turns);
    println!("player1 score: {}", scores.0);
    println!("player2 score: {}", scores.1);
}

fn parse_nvn_config(filename: &str) -> Vec<(String, Vec<CompressedState>, u16)> {
    let contents = read_file(filename);

    let v: Value = serde_json::from_str(&contents).expect("Error parsing config file");

    let keypairs = match v {
        Value::Object(keypairs) => keypairs,
        _ => panic!("Config file is malformed"),
    };

    keypairs
        .into_iter()
        .map(|(filename, value)| {
            let dfa = load_dfa(&filename);
            let number = value
                .as_u64()
                .expect(&format!("Invalid number: {:?}", value)) as u16;
            if number == 0 {
                panic!("There should be at least 1 of each dfa");
            }
            (filename, dfa, number)
        })
        .collect()
}

fn simulate_nvn(states: &Vec<(String, Vec<CompressedState>, u16)>, turns: u8) -> Vec<i64> {
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

fn run_nvn(states: &Vec<(String, Vec<CompressedState>, u16)>, turns: u8) {
    println!("{} turns:", turns);
    let scores = simulate_nvn(states, turns);
    for i in 0..states.len() {
        println!("{} ({}): {}", states[i].0, states[i].2, scores[i]);
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("tobiuo {}", VERSION);
    } else if args.cmd_1v1 {
        let player1 = load_dfa(&args.arg_player1_file);
        let player2 = load_dfa(&args.arg_player2_file);

        if args.flag_all_turns {
            for turns in 95..=105 {
                run(&player1, &player2, turns);
                println!("");
            }
        } else {
            run(&player1, &player2, args.flag_turns);
        }
    } else if args.cmd_nvn {
        let nvn_states = parse_nvn_config(&args.arg_config_file);

        if args.flag_all_turns {
            for turns in 95..=105 {
                run_nvn(&nvn_states, turns);
                println!("");
            }
        } else {
            run_nvn(&nvn_states, args.flag_turns);
        }
    } else {
        panic!("Huh? This message should never be printed. Please contact author.");
    }

    // let test_uo = CompleteStr(include_str!("../test.uo"));
    // println!("{:?}", dfa(test_uo));
}
