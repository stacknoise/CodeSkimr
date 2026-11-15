//! Benchmarks of scans of a Git source (blueprint 7, "Fetch"; issue #36). The origin is a local
//! repository that holds the generated corpus and is served by `git daemon` on the loopback
//! interface (the product accepts no `file://` URLs), so the benchmark needs no network and
//! measures the work of the product, not the speed of a server.
//!
//! - `git_scan/first_100k_loc`: first scan, with an empty cache: clone and scan.
//! - `git_scan/update_100k_loc`: scan after one new commit that changes one file, with the cache
//!   and the database of the earlier scan: update of the clone and scan.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use criterion::Criterion;

mod common;

use common::{SCRATCH, build_binary, build_corpus, measure, root, scan_summary, wipe};

const SAMPLES: usize = 10;
const MEASUREMENT_SECONDS: u64 = 30;
/// The file that gets a line appended for every new commit; it exists in the generated corpus.
const CHANGED_FILE: &str = "part0/thing0.rs";
const APPENDED_LINE: &[u8] = b"// changed\n";
const BRANCH: &str = "main";
/// Folder of the origin's Git data below the scratch folder. The corpus is its work tree.
const ORIGIN_DIR: &str = "git-origin";
/// Folder of the product's data (database and Git cache) below the scratch folder.
const DATA_DIR: &str = "git-data";
/// Identity for the commits of the origin; nothing of the user's Git configuration is used.
const GIT_IDENTITY: [&str; 4] = [
    "-c",
    "user.name=bench",
    "-c",
    "user.email=bench@example.invalid",
];

/// How long the daemon may take to accept connections.
const DAEMON_START_SECONDS: u64 = 10;
const DAEMON_POLL_MS: u64 = 50;

/// A local origin repository whose work tree is the generated corpus.
struct Origin {
    git_dir: PathBuf,
    work_tree: PathBuf,
}

impl Origin {
    fn create(scratch: &Path, corpus: &Path) -> io::Result<Self> {
        let git_dir = scratch.join(ORIGIN_DIR);
        wipe(&git_dir)?;
        let origin = Self {
            git_dir,
            work_tree: corpus.to_path_buf(),
        };
        origin.git(&["init", "-q", "-b", BRANCH])?;
        origin.commit()?;
        Ok(origin)
    }

    fn git(&self, args: &[&str]) -> io::Result<()> {
        let status = Command::new("git")
            .args(GIT_IDENTITY)
            .arg("--git-dir")
            .arg(&self.git_dir)
            .arg("--work-tree")
            .arg(&self.work_tree)
            .args(args)
            .stdout(Stdio::null())
            .status()?;
        status
            .success()
            .then_some(())
            .ok_or_else(|| io::Error::other("git failed"))
    }

    fn commit(&self) -> io::Result<()> {
        self.git(&["add", "-A"])?;
        self.git(&["commit", "-q", "-m", "bench"])
    }

    /// Appends a line to one file and commits it.
    fn commit_change(&self) -> io::Result<()> {
        OpenOptions::new()
            .append(true)
            .open(self.work_tree.join(CHANGED_FILE))?
            .write_all(APPENDED_LINE)?;
        self.commit()
    }
}

/// `git daemon` serving the origin on the loopback interface; stopped when dropped.
struct Daemon {
    child: Child,
    url: String,
}

impl Daemon {
    fn start(scratch: &Path) -> io::Result<Self> {
        // Ask the system for a free port; the small window until the daemon binds it is no risk
        // on a benchmark machine.
        let port = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?
            .local_addr()?
            .port();
        let child = Command::new("git")
            .arg("daemon")
            .arg("--export-all")
            .arg("--reuseaddr")
            .arg(format!("--listen={}", Ipv4Addr::LOCALHOST))
            .arg(format!("--port={port}"))
            .arg(format!("--base-path={}", scratch.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let mut daemon = Self {
            child,
            url: format!("git://{}:{port}/{ORIGIN_DIR}", Ipv4Addr::LOCALHOST),
        };
        let deadline = Instant::now() + Duration::from_secs(DAEMON_START_SECONDS);
        while TcpStream::connect((Ipv4Addr::LOCALHOST, port)).is_err() {
            if daemon.child.try_wait()?.is_some() || Instant::now() > deadline {
                return Err(io::Error::other("git daemon did not start"));
            }
            std::thread::sleep(Duration::from_millis(DAEMON_POLL_MS));
        }
        Ok(daemon)
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn git_scans(c: &mut Criterion) -> io::Result<()> {
    let scratch = root().join(SCRATCH);
    let corpus = build_corpus(&scratch)?;
    let origin = Origin::create(&scratch, &corpus)?;
    let binary = build_binary()?;
    let daemon = Daemon::start(&scratch)?;
    let url = &daemon.url;
    let data = scratch.join(DATA_DIR);
    measure(
        c,
        ("git_scan", "first_100k_loc"),
        (SAMPLES, MEASUREMENT_SECONDS),
        || {
            wipe(&data)?;
            scan_summary(&binary, url, &data, Some(BRANCH)).map(|(wall, _)| wall)
        },
    );
    wipe(&data)?;
    scan_summary(&binary, url, &data, Some(BRANCH))?;
    measure(
        c,
        ("git_scan", "update_100k_loc"),
        (SAMPLES, MEASUREMENT_SECONDS),
        || {
            origin.commit_change()?;
            scan_summary(&binary, url, &data, Some(BRANCH)).map(|(wall, _)| wall)
        },
    );
    Ok(())
}

fn main() -> io::Result<()> {
    let mut criterion = Criterion::default().configure_from_args();
    git_scans(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
