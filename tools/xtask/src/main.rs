//! Repository checks without external dependencies.
mod bench;
mod corpus;
mod docs;
mod limits;
mod serve;
mod sha256;
mod showcase;
mod toolchain;

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "docs" || s == "assets") {
        return match docs::run(&args[1..], args[0] == "assets") {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    if args.first().is_some_and(|s| s == "bench") {
        return match bench::run(&args[1..]) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                std::process::ExitCode::from(1)
            }
        };
    }
    if args.first().is_some_and(|s| s == "serve") {
        return match serve::run(&args[1..]) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                std::process::ExitCode::from(1)
            }
        };
    }
    if args.first().is_some_and(|s| s == "corpus") {
        return match corpus::run(&args[1..]) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                std::process::ExitCode::from(1)
            }
        };
    }
    if args.first().is_some_and(|s| s == "showcase") {
        return match showcase::run(&args[1..]) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                std::process::ExitCode::from(1)
            }
        };
    }
    if args.first().is_none_or(|s| s != "check") {
        eprintln!(concat!(
            "Usage: cargo xtask check [--soft-width] | bench [PAIRS] [PATH]",
            " | corpus SOURCE DESTINATION | docs [--no-pdf] [--no-screenshots]",
            " | serve [--port N] [--no-pdf] | assets",
            " | showcase [--terminal NAME] NAME=PATH..."
        ));
        return std::process::ExitCode::from(1);
    }
    match limits::check(args.iter().any(|s| s == "--soft-width")) {
        Ok(()) => {
            println!("Repository limits passed.");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(1)
        }
    }
}
