//! A stand-in for a real indexer, started by the tests of `codeskimr_indexer`.
//!
//! `fake_indexer <mode> <output file>`; `--version` prints a version. The modes act like the
//! behaviours a real indexer can show: write an index, hang, start a child that hangs, write too
//! much, use too much memory, fail with a message, tell what environment it got.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::thread;
use std::time::Duration;

const HANG: Duration = Duration::from_secs(120);

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mode = args.first().map_or("", String::as_str);
    let output = args.get(1).map_or("", String::as_str);
    let pid_file = args.get(2).map_or("", String::as_str);
    match run(mode, Path::new(output), Path::new(pid_file)) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(io::stderr(), "fake_indexer: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(mode: &str, output: &Path, pid_file: &Path) -> io::Result<ExitCode> {
    match mode {
        "--version" => writeln!(io::stdout(), "fake-indexer 1.2.3")?,
        "ok" => fs::write(output, b"index")?,
        "no-index" => {}
        "hang" | "child" => thread::sleep(HANG),
        "hang-with-child" => hang_with_child(pid_file)?,
        "flood" => flood()?,
        "big-file" => big_file(output)?,
        "memory" => use_memory(),
        "dump-env" => dump_env(output)?,
        "fail" => return fail(),
        "fail-long" => return fail_long(),
        "deep-dir" => deep_dir(output)?,
        "link-index" => link_index(output, pid_file)?,
        _ => return Ok(ExitCode::from(64)),
    }
    Ok(ExitCode::SUCCESS)
}

/// Starts a child that outlives it unless the whole group is killed, and writes the child's id.
fn hang_with_child(pid_file: &Path) -> io::Result<()> {
    let me = env::current_exe()?;
    let child = Command::new(me).arg("child").spawn()?;
    fs::write(pid_file, child.id().to_string())?;
    thread::sleep(HANG);
    Ok(())
}

fn big_file(output: &Path) -> io::Result<()> {
    fs::write(output, vec![7_u8; 4 * 1024 * 1024])?;
    thread::sleep(HANG);
    Ok(())
}

fn flood() -> io::Result<()> {
    let line = vec![b'x'; 1024];
    let mut stdout = io::stdout().lock();
    for _ in 0..16 * 1024 {
        stdout.write_all(&line)?;
    }
    stdout.flush()?;
    thread::sleep(HANG);
    Ok(())
}

fn use_memory() {
    let mut block = vec![0_u8; 300 * 1024 * 1024];
    for page in block.chunks_mut(4096) {
        page.fill(1);
    }
    thread::sleep(HANG);
    drop(block);
}

fn dump_env(output: &Path) -> io::Result<()> {
    let mut file = fs::File::create(output)?;
    for (name, value) in env::vars() {
        writeln!(file, "{name}={value}")?;
    }
    Ok(())
}

fn fail() -> io::Result<ExitCode> {
    writeln!(
        io::stderr(),
        "restore failed: Authorization: Bearer CANARY-SECRET-VALUE for feed"
    )?;
    Ok(ExitCode::from(3))
}

/// A secret that sits across the end of the console output, so that a cut before redaction would
/// split it.
fn fail_long() -> io::Result<ExitCode> {
    let filler = "x".repeat(2040);
    writeln!(io::stderr(), "CANARY-SECRET-VALUE{filler}")?;
    Ok(ExitCode::from(3))
}

/// Nests folders deeper than the runner measures and puts data at the bottom.
fn deep_dir(output: &Path) -> io::Result<()> {
    let mut dir = output.parent().map(Path::to_path_buf).unwrap_or_default();
    for _ in 0..64 {
        dir.push("d");
    }
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("data"), vec![7_u8; 1024 * 1024])?;
    thread::sleep(HANG);
    Ok(())
}

#[cfg(unix)]
fn link_index(output: &Path, target: &Path) -> io::Result<()> {
    fs::write(target, b"not the index")?;
    std::os::unix::fs::symlink(target, output)
}

#[cfg(not(unix))]
fn link_index(output: &Path, _target: &Path) -> io::Result<()> {
    fs::write(output, b"index")
}
