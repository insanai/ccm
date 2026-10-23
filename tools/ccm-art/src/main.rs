//! Reproducible CCM vector identity. SVG is emitted with the standard library.
mod diagrams;
mod logo;
use std::path::Path;

fn main() -> std::process::ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let result = std::fs::create_dir_all(root.join("assets")).and_then(|_| {
        std::fs::write(root.join("assets/ccm-logo.svg"), logo::render(false))?;
        std::fs::write(root.join("assets/ccm-logo-mono.svg"), logo::render(true))?;
        for kind in ["architecture", "sequence", "change", "economics"] {
            std::fs::write(
                root.join(format!("assets/{kind}.svg")),
                diagrams::render(kind),
            )?;
        }
        Ok(())
    });
    match result {
        Ok(()) => {
            println!("Generated assets/ccm-logo.svg and assets/ccm-logo-mono.svg");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Cannot generate CCM identity: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
