// SPDX-License-Identifier: MPL-2.0

mod args;
mod commands;

use crate::args::{Cli, Command};
use clap::Parser;
use std::error::Error;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        Command::Probe(args) => commands::probe::run(args),
        Command::Discover(args) => commands::discover::run(args),
        Command::Query(args) => commands::query::run(args),
    }
}
