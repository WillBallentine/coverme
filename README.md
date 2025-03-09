# coverme

An open source local code coverage analyzer tool written in Rust.

# Crates.io

You can always find the latest version of the crate here: https://crates.io/crates/coverme

# Install

To install, you can run `cargo install coverme`

You are also welcome to pull the repo and build/run directly.

## CLI Arguments

- --repo <PATH>
  - pass the directory you want to run the analysis on
- --language <Language>
  - pass the language_id that the code being tested is written in
    - "rust"
    - "csharp"
    - "js"

## How To Use

### If you installed via cargo install

In your cli, run `coverme --repo path/to/repo --language language_id`

### If you pulled the repo from GitHub and are running directly

In your cli, run `cargo run -- --repo path/to/repo --language language_id`

## Current Language Support

- Rust
- C#
- JavaScript

### Languages In The Works

- Python
- Go

## Language Specific Notes

### JavaScript

The way JS tests are currently being detected is via checking for testing framework keywords which then call the function being tested. A more robust implementation is in the pipeline. Please open an issue if you have a specific use case where tests exist but the functions are not being marked as covered.

### C#

Currently, the C# implementation is looking specifically for `[Test]`, `[TestMethod]`, `[Fact]` or `[Theory]` in the attribute list of the method in the syntax tree. If you are using a testing framework that denotes tests in a different way, your test will not currently be detected. Please open an issue if you have a specific case where tests exist but the functions are not being marked as covered.

### Rust

Rust tests are only being detected currently by looking for the `[test]` attribute. If tests exist within another entity such as within a `mod`, they may not be detected at this time. A fix for this is in the pipeline. Please open an issue if you have a specific case where tests exist but the functions are not being marked as covered.

## Example output

Currently, we are only reporting at the method level. Here is an example output from running coverall on its own repo with a few tests written.

![alt text](https://github.com/WillBallentine/coverme/blob/main/coverage_example.PNG)

# Contributing and Ideas

Feel free to open an issue on GitHub with any ideas you have on how we can improve coverme. These are all subject to review and approval before being allowed.

If you would like to help build coverme, create a feature branch off of `main` and submit a PR. If the PR meets the standards and goals of this project, I will approve it and merge it in.
