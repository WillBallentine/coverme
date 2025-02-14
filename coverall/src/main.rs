use clap::Parser;
use anyhow::{Context, Result};
use indicatif::ProgressBar;

#[derive(Parser)]
struct Cli {
    pattern: String,
    repo: std::path::PathBuf,
}

fn main() -> Result<()>{
    let args = Cli::parse();
    let pb = ProgressBar::new(100);

    let content = std::fs::read_to_string(&args.repo)
        .with_context(|| format!("Error reading `{}`", args.repo.display()))?;

    let mut iter = 1;
    for line in content.lines() {
        pb.println(format!("[+] reading line  #{}", iter));
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
        iter += 1;
    }
    pb.finish_with_message("done");

    Ok(())
}
