//! `codeskimr scan` and `codeskimr mcp` end to end, through the real binary.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;

const SOURCE: &str = "namespace Demo.Core;\n\n/// <summary>Adds numbers.</summary>\npublic class Calculator\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

/// A scanned project in a temporary folder with its own data folder.
#[cfg(test)]
struct Workspace {
    root: TempDir,
}

#[cfg(test)]
impl Workspace {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("project/src")).unwrap();
        fs::write(root.path().join("project/src/Calculator.cs"), SOURCE).unwrap();
        fs::write(root.path().join("empty-config.toml"), "").unwrap();
        Self { root }
    }

    fn project(&self) -> PathBuf {
        self.root.path().join("project")
    }

    fn command(&self) -> Command {
        let languages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages");
        let mut command = Command::new(env!("CARGO_BIN_EXE_codeskimr"));
        command
            .env("CODESKIMR_LANG", "en")
            .env("CODESKIMR_DATA_DIR", self.root.path().join("data"))
            .env("CODESKIMR_LANGUAGES_DIR", languages)
            .env(
                "CODESKIMR_CONFIG",
                self.root.path().join("empty-config.toml"),
            );
        command
    }

    fn scan(&self, extra: &[&str]) -> Output {
        self.command()
            .args(["scan", "--json"])
            .arg(self.project())
            .args(extra)
            .output()
            .unwrap()
    }
}

#[cfg(test)]
fn json_of(output: &Output) -> Value {
    assert!(output.status.success(), "{}", stderr(output));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[cfg(test)]
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[cfg(test)]
fn with_user_config(workspace: &Workspace, text: &str) -> PathBuf {
    let path = workspace.root.path().join("config.toml");
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn scan_finds_files_and_symbols_and_writes_a_snapshot() {
    let workspace = Workspace::new();
    let summary = json_of(&workspace.scan(&[]));
    assert_eq!(summary["project"], "project");
    assert_eq!(summary["files"], 1);
    assert!(summary["symbols"].as_u64().unwrap() >= 3);
    assert!(summary["snapshot"].as_u64().unwrap() >= 1);
}

/// A secret in this shape is found by the redaction, so it must never be printed.
const TOKEN_CANARY: &str = "ghp_CANARY7f3a9c1e5b2d4f60aa11bb22";

#[test]
fn the_log_level_never_moves_log_lines_to_standard_output() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .env("CODESKIMR_LOG", "trace")
        .args(["scan", "--json"])
        .arg(workspace.project())
        .output()
        .unwrap();
    assert_eq!(json_of(&output)["files"], 1);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("DEBUG"));
}

#[test]
fn canaries_in_arguments_reach_neither_the_log_nor_the_error_output() {
    let workspace = Workspace::new();
    let url = format!("https://user:{TOKEN_CANARY}@host.example/repo.git");
    let flag = format!("--json={TOKEN_CANARY}");
    for extra in [url, flag] {
        let output = workspace
            .command()
            .env("CODESKIMR_LOG", "debug")
            .env("CODESKIMR_CANARY", TOKEN_CANARY)
            .arg("scan")
            .arg(workspace.project())
            .arg(extra)
            .output()
            .unwrap();
        let (out, err) = (String::from_utf8_lossy(&output.stdout), stderr(&output));
        assert!(!output.status.success());
        assert!(err.contains("DEBUG"), "the log line is missing: {err}");
        assert!(
            !out.contains("CANARY") && !err.contains("CANARY"),
            "leaked: {out}{err}"
        );
    }
}

#[test]
fn scanning_again_reuses_the_project_and_adds_a_snapshot() {
    let workspace = Workspace::new();
    let first = json_of(&workspace.scan(&[]));
    let second = json_of(&workspace.scan(&[]));
    assert_eq!(second["project"], first["project"]);
    assert!(second["snapshot"].as_u64() > first["snapshot"].as_u64());
}

#[test]
fn the_text_summary_follows_the_language() {
    let workspace = Workspace::new();
    let english = workspace
        .command()
        .arg("scan")
        .arg(workspace.project())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&english.stdout).contains("Scanned 1 files"));
    let german = workspace
        .command()
        .env("CODESKIMR_LANG", "de_DE.UTF-8")
        .arg("scan")
        .arg(workspace.project())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&german.stdout).contains("1 Dateien"));
}

#[test]
fn the_command_line_overrides_the_project_file_which_overrides_the_user_file() {
    let workspace = Workspace::new();
    let user = with_user_config(&workspace, "[scan]\nexcluded_folders = [\"src\"]\n");
    let from_user = workspace.scan(&["--config", user.to_str().unwrap()]);
    assert_eq!(json_of(&from_user)["files"], 0);

    fs::write(
        workspace.project().join(".codeskimr.toml"),
        "[scan]\nexcluded_folders = [\"other\"]\n",
    )
    .unwrap();
    let from_project = workspace.scan(&["--config", user.to_str().unwrap()]);
    assert_eq!(json_of(&from_project)["files"], 1);

    let from_flag = workspace.scan(&["--set", "scan.excluded_folders=[\"src\"]"]);
    assert_eq!(json_of(&from_flag)["files"], 0);
}

#[test]
fn an_unknown_setting_fails_with_cfg_103() {
    let workspace = Workspace::new();
    let output = workspace.scan(&["--set", "scan.no_such_key=1"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("CFG-103"));
}

#[test]
fn serve_refuses_port_zero_with_cfg_101_before_it_starts_anything() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .args(["serve", "--set", "server.port=0"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(stderr(&output).contains("CFG-101"), "{}", stderr(&output));
}

#[test]
fn serve_has_no_flag_for_another_address() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .args(["serve", "--bind", "0.0.0.0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_missing_named_config_file_fails() {
    let workspace = Workspace::new();
    let output = workspace.scan(&["--config", "/no/such/file.toml"]);
    assert!(stderr(&output).contains("CFG-102"));
}

#[test]
fn a_source_that_is_nothing_fails_with_prj_101() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .args(["scan", "/no/such/folder"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("PRJ-101"));
}

#[test]
fn a_git_url_with_credentials_is_refused_and_never_printed() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .args(["scan", "https://user:secret-token@example.com/repo.git"])
        .output()
        .unwrap();
    let all = format!(
        "{}{}",
        stderr(&output),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(all.contains("PRJ-101"));
    assert!(all.contains("credential helper"));
    assert!(!all.contains("secret-token"));
}

#[test]
fn missing_arguments_exit_with_the_usage_code() {
    let workspace = Workspace::new();
    let output = workspace.command().arg("scan").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("CLI-101"));
}

#[test]
fn help_is_shown_in_the_chosen_language() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .env("CODESKIMR_LANG", "de")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Ordner oder eine Git-URL"));
}

/// Starts `command` and forwards every line of its output that is JSON, as it arrives.
#[cfg(test)]
fn spawn_json_lines(
    mut command: Command,
    stdin: Stdio,
) -> (std::process::Child, mpsc::Receiver<Value>) {
    let mut child = command
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, lines) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                let _ = sender.send(value);
            }
        }
    });
    (child, lines)
}

/// A running `codeskimr mcp` process that answers line by line.
#[cfg(test)]
struct McpSession {
    child: std::process::Child,
    answers: mpsc::Receiver<Value>,
}

#[cfg(test)]
impl McpSession {
    fn start(workspace: &Workspace) -> Self {
        let mut command = workspace.command();
        command.arg("mcp").arg("--project").arg(workspace.project());
        let (child, answers) = spawn_json_lines(command, Stdio::piped());
        Self { child, answers }
    }

    fn send(&mut self, message: &Value) {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{message}").unwrap();
    }

    fn call(&mut self, id: u64, method: &str, params: &Value) -> Value {
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let answer = self
                .answers
                .recv_timeout(Duration::from_secs(30))
                .expect("the server answered within 30 seconds");
            if answer["id"] == id {
                return answer;
            }
        }
    }
}

#[cfg(test)]
impl Drop for McpSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
fn initialized(workspace: &Workspace) -> McpSession {
    let mut session = McpSession::start(workspace);
    let init = session.call(
        1,
        "initialize",
        &json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0"}
        }),
    );
    assert_eq!(init["result"]["serverInfo"]["name"], "codeskimr");
    session.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    session
}

#[test]
fn mcp_answers_from_the_scanned_project() {
    let workspace = Workspace::new();
    json_of(&workspace.scan(&[]));
    let mut session = initialized(&workspace);
    let answer = session.call(
        2,
        "tools/call",
        &json!({"name": "search_symbols", "arguments": {"query": "Calculator"}}),
    );
    let result = &answer["result"];
    assert_ne!(result["isError"], true, "{result}");
    assert_eq!(result["structuredContent"]["hits"][0]["name"], "Calculator");
    let hit = result["structuredContent"]["hits"][0]["qualified_name"]
        .as_str()
        .unwrap();
    assert!(hit.ends_with("Calculator"));
}

#[cfg(test)]
fn mcp_without_input(workspace: &Workspace) -> Output {
    workspace
        .command()
        .arg("mcp")
        .arg("--project")
        .arg(workspace.project())
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn mcp_for_a_folder_that_was_never_scanned_fails_with_prj_201() {
    let workspace = Workspace::new();
    let output = mcp_without_input(&workspace);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("PRJ-201"));
    assert_eq!(output.stdout, Vec::<u8>::new());
}

#[test]
fn mcp_for_a_registered_project_without_snapshot_fails_with_mcp_203() {
    let workspace = Workspace::new();
    let folder = fs::canonicalize(workspace.project()).unwrap();
    let store = codeskimr_storage::SqliteStore::open(
        &workspace.root.path().join("data/codeskimr.db"),
        std::num::NonZeroU32::new(5).unwrap(),
    )
    .unwrap();
    let project = codeskimr_domain::Project {
        id: codeskimr_domain::ProjectId::new(1),
        name: codeskimr_domain::ProjectName::new("project").unwrap(),
        source: codeskimr_domain::ProjectSource::Folder(
            codeskimr_domain::FolderPath::new(folder.to_string_lossy().into_owned()).unwrap(),
        ),
        layers: Vec::new(),
    };
    store.save_project(&project).unwrap();
    drop(store);
    let output = mcp_without_input(&workspace);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("MCP-203"));
}

/// A running `codeskimr scan --watch --json` process; every scan is one line of JSON.
#[cfg(test)]
struct WatchSession {
    child: std::process::Child,
    summaries: mpsc::Receiver<Value>,
}

#[cfg(test)]
impl WatchSession {
    fn start(workspace: &Workspace) -> Self {
        let mut command = workspace.command();
        command
            .args(["scan", "--watch", "--json"])
            .arg(workspace.project());
        let (child, summaries) = spawn_json_lines(command, Stdio::null());
        Self { child, summaries }
    }

    fn next_summary(&self) -> Value {
        self.summaries
            .recv_timeout(Duration::from_secs(30))
            .expect("a scan was reported within 30 seconds")
    }
}

#[cfg(test)]
impl Drop for WatchSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn watch_scans_again_when_a_file_is_added() {
    let workspace = Workspace::new();
    let session = WatchSession::start(&workspace);
    let first = session.next_summary();
    assert_eq!(first["files"], 1);
    fs::write(
        workspace.project().join("src/Other.cs"),
        "namespace Demo.Core;\n\npublic class Other\n{\n}\n",
    )
    .unwrap();
    let second = session.next_summary();
    assert_eq!(second["files"], 2);
    assert!(second["snapshot"].as_u64() > first["snapshot"].as_u64());
}

#[test]
fn watch_of_a_git_url_fails_with_cli_102_before_anything_is_registered() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .args(["scan", "--watch", "https://example.com/team/repo.git"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("CLI-102"));
    assert!(!workspace.root.path().join("data/codeskimr.db").exists());
}
