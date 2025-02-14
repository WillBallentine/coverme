use clap::Parser;

#[derive(Parser)]
struct Cli {
    pattern: String,
    repo: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    let content = std::fs::read_to_string(&args.repo).expect("could not read file");
}
