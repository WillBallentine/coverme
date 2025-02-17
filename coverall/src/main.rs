use std::env;
use std::result::Result::Ok;

use clap::{Arg, ArgMatches};
use cli::get_cli_args;

use anyhow::{Context, Result};
use colored::*;

pub mod cli;

struct Command {
    repo: String,
    pattern: String,
}

fn run() -> Result<()> {
    let cli_args = get_cli_args(env::args_os());

    let command = unwrap_command(cli_args);

    let content =
        std::fs::read_to_string(command.repo).with_context(|| format!("Error reading repo"))?;

    let mut iter = 1;
    for line in content.lines() {
        if line.contains(command.pattern.as_str()) {
            println!("found on line [{}]: {}", iter, line);
        }
        iter += 1;
    }

    Ok(())
}

fn unwrap_command(cli_args: ArgMatches) -> Command {
    Command {
        repo: cli_args.get_one::<String>("repo").unwrap().clone(),
        pattern: cli_args.get_one::<String>("pattern").unwrap().clone(),
    }
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {:#}", "Error".red(), e);
            std::process::exit(1);
        }
    }
}
