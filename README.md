# coverall
An open source local code coverage analyzer tool written in Rust.


## CLI Options
- --repo <PATH>
    - pass the directory you want to run the analysis on
- --language <Language>
    - pass the language that the code being tested is written in


## Current Language Support
- Rust
- C# (still in verification testing)

## Example output
Currently, we are only reporting at the method level. Here is an example output from running coverall on its own repo with only one test written.

![alt text](image.png)