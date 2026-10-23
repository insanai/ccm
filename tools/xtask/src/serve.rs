//! `cargo xtask serve`: builds the whole mounted website and previews it, rebuilding when a
//! source changes.
//!
//! `guidedog serve` previews one project at a time. The home page links to the book and
//! the decisions with `mount://` links, and those resolve only when `guidedog ssg build`
//! assembles every mount. Serving `docs` or `docs/site` directly therefore reports a
//! missing root document or invalid mount links. We build the assembled site instead and
//! serve its output with a small static file server.
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

/// How often we look for changed sources.
const POLL: Duration = Duration::from_secs(1);

/// Runs `cargo xtask serve [--port N] [--pdf]`.
pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let port = match arguments.iter().position(|a| a == "--port") {
        Some(index) => arguments
            .get(index + 1)
            .and_then(|p| p.parse::<u16>().ok())
            .ok_or("Use --port with a number from 1 to 65535")?,
        None => 8000,
    };
    let pdf = arguments.iter().any(|a| a == "--pdf");
    let guidedog = crate::toolchain::require(
        "guidedog",
        "Guidedog builds the mounted website.",
        "Install Guidedog, put it on PATH, and run guidedog --version.",
    )?;
    let site = root.join("docs/_build/site");
    build(&root, &guidedog, &site, pdf);
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    println!(
        "Serving {} at http://127.0.0.1:{port}/ (Ctrl+C to stop)",
        site.display()
    );
    let watched = (root.clone(), site.clone());
    std::thread::spawn(move || watch(&watched.0, &guidedog, &watched.1, pdf));
    for stream in listener.incoming().flatten() {
        if let Err(error) = respond(stream, &site) {
            eprintln!("Request failed: {error}");
        }
    }
    Ok(())
}

/// Builds the site; a failed build is reported and the previous output keeps serving.
fn build(root: &Path, guidedog: &Path, site: &Path, pdf: bool) {
    let mut command = Command::new(guidedog);
    command
        .current_dir(root)
        .args(["ssg", "build", "docs", "--out"])
        .arg(site);
    if !pdf {
        command.arg("--no-pdf");
    }
    match command.status() {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("The site build failed ({status}); serving the previous build."),
        Err(error) => eprintln!("Cannot run Guidedog: {error}"),
    }
}

/// Rebuilds whenever the newest modification time under `docs` or `assets` changes.
fn watch(root: &Path, guidedog: &Path, site: &Path, pdf: bool) {
    let mut seen = newest(root);
    loop {
        std::thread::sleep(POLL);
        let now = newest(root);
        if now != seen {
            println!("Sources changed; rebuilding.");
            build(root, guidedog, site, pdf);
            seen = newest(root);
        }
    }
}

/// Returns the newest modification time and file count of the publication sources,
/// skipping build output so our own builds do not trigger rebuilds.
fn newest(root: &Path) -> (Option<SystemTime>, usize) {
    let mut stack = vec![root.join("docs"), root.join("assets")];
    let (mut latest, mut count) = (None, 0);
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                if entry.file_name() != "_build" {
                    stack.push(entry.path());
                }
            } else if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
                latest = latest.max(Some(modified));
                count += 1;
            }
        }
    }
    (latest, count)
}

/// Answers one HTTP GET request with a file from `site`.
fn respond(mut stream: TcpStream, site: &Path) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    reader.read_line(&mut request)?;
    // Read and ignore the headers up to the blank line.
    let mut header = String::new();
    while reader.read_line(&mut header)? > 2 {
        header.clear();
    }
    let target = request.split_whitespace().nth(1).unwrap_or("/");
    let (status, kind, body) =
        match resolve(site, target).and_then(|p| std::fs::read(&p).ok().map(|b| (p, b))) {
            Some((path, body)) => ("200 OK", content_type(&path), body),
            None => ("404 Not Found", "text/plain", b"Not found\n".to_vec()),
        };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)
}

/// Maps a request target to a file below `site`, refusing paths that leave it.
fn resolve(site: &Path, target: &str) -> Option<PathBuf> {
    let path = target.split(['?', '#']).next().unwrap_or("/");
    let relative = PathBuf::from(decode(path)?.trim_start_matches('/'));
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    let full = site.join(relative);
    if full.is_dir() {
        Some(full.join("index.html"))
    } else {
        Some(full)
    }
}

/// Decodes `%XX` escapes in a URL path.
fn decode(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "woff2" => "font/woff2",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_stay_inside_the_site() {
        let site = Path::new("/site");
        assert_eq!(
            resolve(site, "/book/a%20b.html?x=1"),
            Some(PathBuf::from("/site/book/a b.html"))
        );
        assert_eq!(resolve(site, "/../etc/passwd"), None);
        assert_eq!(resolve(site, "/%2e%2e/x"), None);
        assert_eq!(decode("%zz"), None);
    }
}
