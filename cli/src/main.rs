#[macro_use]
extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;

extern crate serde_json;
use serde_json::Value;

extern crate tobiuo;
use tobiuo::{parse_dfa, simulate, simulate_nvn, CompressedState};

use std::fs::File;
use std::io::prelude::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "
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
        .unwrap_or_else(|_| panic!("Error opening file {}", filename))
        .read_to_string(&mut contents)
        .unwrap_or_else(|_| panic!("Error reading file {}", filename));

    contents
}

fn load_dfa(filename: &str) -> Vec<CompressedState> {
    let contents = read_file(filename);

    parse_dfa(&contents).expect(&format!("Error parsing readingfile {}", filename))
}

fn run(player1: &[CompressedState], player2: &[CompressedState], turns: u8) {
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
                .unwrap_or_else(|| panic!("Invalid number: {:?}", value))
                as u16;
            if number == 0 {
                panic!("There should be at least 1 of each dfa");
            }
            (filename, dfa, number)
        })
        .collect()
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
                println!();
            }
        } else {
            run(&player1, &player2, args.flag_turns);
        }
    } else if args.cmd_nvn {
        let nvn_states = parse_nvn_config(&args.arg_config_file);

        if args.flag_all_turns {
            for turns in 95..=105 {
                run_nvn(&nvn_states, turns);
                println!();
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
