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

#[test]
fn layers_of_the_project_file_reach_get_architecture() {
    let workspace = Workspace::new();
    fs::write(
        workspace.project().join(".codeskimr.toml"),
        "[[layers]]\nname = \"core\"\nnamespaces = [\"Demo.Core\"]\n",
    )
    .unwrap();
    json_of(&workspace.scan(&[]));
    let mut session = initialized(&workspace);
    let answer = session.call(
        2,
        "tools/call",
        &json!({"name": "get_architecture", "arguments": {}}),
    );
    let result = &answer["result"];
    assert_ne!(result["isError"], true, "{result}");
    assert_eq!(result["structuredContent"]["layers"][0]["name"], "core");
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

#[cfg(test)]
const EXPORT_PAGE: &str = "<html><script id=\"codeskimr-data\" type=\"application/json\">__CODESKIMR_EXPORT_DATA__</script></html>";

#[cfg(test)]
fn export_ui(workspace: &Workspace) -> String {
    let ui = workspace.root.path().join("ui");
    fs::create_dir_all(ui.join("export")).unwrap();
    fs::write(ui.join("export/export.html"), EXPORT_PAGE).unwrap();
    format!("ui.dir='{}'", ui.display())
}

#[test]
fn export_writes_one_page_with_the_scan_and_no_source_text() {
    let workspace = Workspace::new();
    json_of(&workspace.scan(&[]));
    let file = workspace.root.path().join("out.html");
    let output = workspace
        .command()
        .args(["--set", &export_ui(&workspace), "export"])
        .arg(workspace.project())
        .arg("--html")
        .arg(&file)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Export written to"));
    let page = fs::read_to_string(&file).unwrap();
    assert!(page.contains("Calculator"));
    assert!(!page.contains("return a + b"));
}

#[test]
fn export_by_project_id_and_the_failures_with_their_codes() {
    let workspace = Workspace::new();
    json_of(&workspace.scan(&[]));
    let file = workspace.root.path().join("out.html");
    let run = |project: &str, target: &Path, ui: &str| {
        workspace
            .command()
            .args(["--set", ui, "export", project, "--html"])
            .arg(target)
            .output()
            .unwrap()
    };
    let ui = export_ui(&workspace);
    assert!(run("1", &file, &ui).status.success());
    let unknown = run("99", &file, &ui);
    assert!(stderr(&unknown).contains("PRJ-201"), "{}", stderr(&unknown));
    let nowhere = workspace.root.path().join("no/such/folder/out.html");
    assert!(stderr(&run("1", &nowhere, &ui)).contains("EXP-601"));
    let no_page = run("1", &file, "ui.dir='/does/not/exist'");
    assert!(stderr(&no_page).contains("EXP-201"));
}

#[test]
fn a_credential_for_a_folder_is_refused() {
    let workspace = Workspace::new();
    let output = workspace.scan(&["--credential", "work"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("PRJ-101"));
}

#[test]
fn an_empty_standard_input_stores_no_secret() {
    let workspace = Workspace::new();
    let mut child = workspace
        .command()
        .args(["credential", "set", "work", "--user", "anton"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("CLI-103"));
}

#[test]
fn the_installation_token_is_not_a_credential_to_manage() {
    let workspace = Workspace::new();
    let output = workspace
        .command()
        .args(["credential", "delete", "installation-token"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("CLI-101"));
}

// --- Security suite (WP-74, blueprint S-02 and S-05) through the real binary ---------------

/// A class name that is found if a scan or a check reads the file that holds it.
const OUTSIDE_CLASS: &str = "CanaryOutside6b1d";

/// Writes a C# file with a very long method at `path`.
#[cfg(test)]
fn write_long_file(path: &Path, class: &str) {
    let body = "        var x = 1;\n".repeat(60);
    let source = format!(
        "namespace Out;\npublic class {class}\n{{\n    public void Long()\n    {{\n{body}    }}\n}}\n"
    );
    fs::write(path, source).unwrap();
}

/// The same file next to the project, outside of it.
fn write_outside_file(workspace: &Workspace) -> PathBuf {
    let path = workspace.root.path().join("outside.cs");
    write_long_file(&path, OUTSIDE_CLASS);
    path
}

/// The whole text of `output`, both streams.
fn all_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        stderr(output)
    )
}

/// `src/link.cs` inside the project, pointing at `outside`; its path relative to the project.
/// Only where symlinks can be made without privileges: on Windows the path stays a plain
/// missing file, which the test also has to refuse.
#[cfg(test)]
fn link_outside(workspace: &Workspace, outside: &Path) -> &'static str {
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside, workspace.project().join("src/link.cs")).unwrap();
    #[cfg(not(unix))]
    let _ = (workspace, outside);
    "src/link.cs"
}

#[test]
fn check_conventions_never_reads_a_file_outside_the_project() {
    let workspace = Workspace::new();
    let outside = write_outside_file(&workspace);
    write_long_file(&workspace.project().join("src/Inside.cs"), "Inside");
    json_of(&workspace.scan(&[]));
    let mut session = initialized(&workspace);
    // The control: the same file inside the project is checked and found too long.
    let inside = session.call(
        2,
        "tools/call",
        &json!({"name": "check_conventions", "arguments": {"paths": ["src/Inside.cs"]}}),
    );
    assert!(inside.to_string().contains("CHK-104"), "{inside}");
    let absolute = outside.display().to_string();
    let link = link_outside(&workspace, &outside);
    for path in [
        "../outside.cs",
        absolute.as_str(),
        "src/../../outside.cs",
        link,
    ] {
        let answer = session.call(
            2,
            "tools/call",
            &json!({"name": "check_conventions", "arguments": {"paths": [path]}}),
        );
        let text = answer.to_string();
        // A controlled refusal: nothing was checked and the path is reported as unmatched.
        let result = &answer["result"]["structuredContent"];
        assert_eq!(result["files_checked"], 0, "{path}: {text}");
        assert_eq!(result["unmatched"], json!([path]), "{path}: {text}");
        assert!(!text.contains(OUTSIDE_CLASS), "{path}: {text}");
        assert!(!text.contains("CHK-104"), "{path}: {text}");
    }
}

#[test]
fn the_cli_check_never_reads_a_file_outside_the_current_folder() {
    let workspace = Workspace::new();
    let outside = write_outside_file(&workspace);
    write_long_file(&workspace.project().join("src/Inside.cs"), "Inside");
    let check = |path: &str| {
        workspace
            .command()
            .current_dir(workspace.project())
            .args(["check", path])
            .output()
            .unwrap()
    };
    assert!(all_text(&check("src/Inside.cs")).contains("CHK-104"));
    let absolute = outside.display().to_string();
    let link = link_outside(&workspace, &outside);
    for path in ["../outside.cs", absolute.as_str(), link] {
        let output = check(path);
        let text = all_text(&output);
        // A controlled refusal: exit code 1 and the "no file matches" message, no crash.
        assert_eq!(output.status.code(), Some(1), "{path}: {text}");
        assert!(text.contains("No file matches"), "{path}: {text}");
        assert!(!text.contains("CHK-104"), "{path}: {text}");
        assert!(!text.contains(OUTSIDE_CLASS), "{path}: {text}");
    }
}

// --- Canaries in logs and process output (WP-74, blueprint S-02) ---------------------------

/// A credential-shaped secret that appears only as input, never as something the app stores.
const SECRET_CANARY: &str = "ghp_CANARY91e4b7c2d0a845f3b6c1d9e2";

/// Every file below `dir` that holds `needle`.
#[cfg(test)]
fn files_with(dir: &Path, needle: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_with(&path, needle));
        } else if fs::read(&path).is_ok_and(|bytes| {
            bytes
                .windows(needle.len())
                .any(|window| window == needle.as_bytes())
        }) {
            found.push(path);
        }
    }
    found
}

/// No canary in the text and none in a file of the data folder. The messages name the stream,
/// never the secret.
#[cfg(test)]
fn assert_clean(workspace: &Workspace, streams: &[(&str, String)]) {
    for (name, text) in streams {
        assert!(!text.contains(SECRET_CANARY), "canary in {name}");
    }
    let data = workspace.root.path().join("data");
    assert!(
        files_with(&data, SECRET_CANARY).is_empty(),
        "canary in the data folder"
    );
}

#[test]
fn a_secret_in_a_git_address_stays_out_of_the_output_and_the_data_folder() {
    let workspace = Workspace::new();
    let address = format!("https://user:{SECRET_CANARY}@127.0.0.1:9/repo.git");
    let output = workspace
        .command()
        .env("CODESKIMR_LOG", "trace")
        .args(["scan", "--json", &address])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "the unreachable address must fail"
    );
    // The control: the failure is reported on standard error.
    assert!(!stderr(&output).is_empty());
    let streams = [
        (
            "stdout",
            String::from_utf8_lossy(&output.stdout).into_owned(),
        ),
        ("stderr", stderr(&output)),
    ];
    assert_clean(&workspace, &streams);
}

/// The requests of a stdio session that carry the secret in every place a client can put one.
#[cfg(test)]
fn stdio_requests() -> [Value; 6] {
    let rpc = |id: u64, method: &str, params: Value| json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
    let call = |id: u64, name: &str, arguments: Value| {
        rpc(
            id,
            "tools/call",
            json!({"name": name, "arguments": arguments}),
        )
    };
    [
        rpc(
            1,
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": SECRET_CANARY, "version": "0"}}),
        ),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        call(2, "search_symbols", json!({"query": SECRET_CANARY})),
        call(3, "check_conventions", json!({"paths": [SECRET_CANARY]})),
        call(4, "get_architecture", json!({"project": 99})),
        call(5, "no_such_tool", json!({"token": SECRET_CANARY})),
    ]
}

#[test]
fn a_secret_sent_to_the_stdio_server_stays_out_of_its_log_and_the_data_folder() {
    let workspace = Workspace::new();
    json_of(&workspace.scan(&[]));
    let mut child = workspace
        .command()
        .env("CODESKIMR_LOG", "trace")
        .arg("mcp")
        .arg("--project")
        .arg(workspace.project())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let requests = stdio_requests();
    let mut stdin = child.stdin.take().unwrap();
    for request in &requests {
        writeln!(stdin, "{request}").unwrap();
    }
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    // The control: every call was answered, so the secret really went through the server.
    let answers = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(answers.lines().count(), 5, "{answers}");
    assert!(!stderr(&output).is_empty(), "the trace level logs");
    assert_clean(&workspace, &[("stderr", stderr(&output))]);
}
