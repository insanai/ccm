//! Repository checks without external dependencies.
mod bench;
mod corpus;
mod dist;
mod docs;
mod homebrew;
mod limits;
mod serve;
mod sha256;
mod showcase;
mod toolchain;

const USAGE: &str = concat!(
    "Usage: cargo xtask check [--soft-width] | bench [PAIRS] [PATH]",
    " | corpus SOURCE DESTINATION | docs [--no-pdf] [--no-screenshots]",
    " | serve [--port N] [--no-pdf] | assets",
    " | showcase [--terminal NAME] NAME=PATH...",
    " | dist TARGET [DIRECTORY] | checksums DIRECTORY | homebrew SHA256SUMS OUTPUT"
);

fn main() -> std::process::ExitCode {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[String]) -> Result<(), String> {
    let (command, rest) = arguments.split_first().ok_or(USAGE)?;
    match command.as_str() {
        "docs" | "assets" => docs::run(rest, command == "assets"),
        "bench" => bench::run(rest),
        "serve" => serve::run(rest),
        "corpus" => corpus::run(rest),
        "showcase" => showcase::run(rest),
        "dist" => dist::run(rest),
        "checksums" => dist::checksums(rest),
        "homebrew" => homebrew::run(rest),
        "check" => {
            limits::check(rest.iter().any(|s| s == "--soft-width"))?;
            println!("Repository limits passed.");
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}
