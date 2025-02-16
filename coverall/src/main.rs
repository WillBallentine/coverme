use std::env;
use std::result::Result::Ok;

use cli::get_cli_args;

use anyhow::{Context, Result};
use colored::*;

pub mod cli;

fn run() -> Result<()> {
    let cli_args = get_cli_args(env::args_os());
    let repo = cli_args.get_one::<String>("repo").unwrap();

    let content = std::fs::read_to_string(cli_args.get_one::<String>("repo").unwrap())
        .with_context(|| format!("Error reading `{}`", repo.red()))?;

    let mut iter = 1;
    for line in content.lines() {
        if line.contains("found") {
            println!("found on line [{}]: {}", iter, line);
        }
        iter += 1;
    }

    Ok(())
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
