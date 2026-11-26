//! The server speaks MCP over a byte stream, as it does over stdio.

#[cfg(test)]
mod common;

use codeskimr_domain::{Permission, PermissionSet, Principal};
use codeskimr_i18n::Messages;
use codeskimr_mcp::McpServer;
use common::tools;
use rmcp::ServiceExt;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, split};

#[cfg(test)]
struct Client {
    reader: BufReader<tokio::io::ReadHalf<DuplexStream>>,
    writer: tokio::io::WriteHalf<DuplexStream>,
}

#[cfg(test)]
impl Client {
    async fn send(&mut self, message: Value) {
        let line = format!("{message}\n");
        self.writer.write_all(line.as_bytes()).await.unwrap();
    }

    async fn receive(&mut self) -> Value {
        let mut line = String::new();
        self.reader.read_line(&mut line).await.unwrap();
        serde_json::from_str(&line).unwrap()
    }

    async fn call(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await;
        self.receive().await
    }

    /// The names of the tools the server lists for this caller.
    async fn tool_names(&mut self, id: u64) -> Vec<String> {
        let listed = self.call(id, "tools/list", json!({})).await;
        let tools = listed["result"]["tools"].as_array().unwrap();
        tools
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_owned())
            .collect()
    }
}

#[cfg(test)]
async fn connected() -> Client {
    connected_as(Principal::local_owner()).await
}

#[cfg(test)]
async fn connected_as(principal: Principal) -> Client {
    let messages = Messages::load().unwrap();
    let server = McpServer::new(tools(), &messages, principal).unwrap();
    let (near, far) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = server.serve(far).await.unwrap();
        running.waiting().await.unwrap();
    });
    let (read, write) = split(near);
    let mut client = Client {
        reader: BufReader::new(read),
        writer: write,
    };
    let init = client
        .call(
            1,
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0"}
            }),
        )
        .await;
    assert_eq!(init["result"]["serverInfo"]["name"], "codeskimr");
    client
        .send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
        .await;
    client
}

/// The tools in the order of the list.
const TOOL_NAMES: [&str; 16] = [
    "search_symbols",
    "get_symbol",
    "get_project",
    "get_architecture",
    "find_usages",
    "impact_of_change",
    "get_context",
    "get_dependencies",
    "find_similar_code",
    "find_constants",
    "find_unused",
    "preview_rename",
    "check_conventions",
    "list_annotations",
    "get_alerts",
    "rescan",
];

#[tokio::test]
async fn lists_exactly_the_sixteen_tools_with_schemas_and_only_rescan_writes() {
    let mut client = connected().await;
    let listed = client.call(2, "tools/list", json!({})).await;
    let tools = listed["result"]["tools"].as_array().unwrap();
    let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, TOOL_NAMES);
    for tool in tools {
        assert_eq!(
            tool["annotations"]["readOnlyHint"],
            tool["name"] != "rescan"
        );
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert_ne!(tool["description"].as_str().unwrap(), "");
    }
    assert_eq!(tools[0]["inputSchema"]["required"][0], "query");
    let symbol = &tools[1]["inputSchema"]["properties"]["symbol"];
    assert_eq!(symbol["anyOf"].as_array().unwrap().len(), 2);
    let context = tools.iter().find(|t| t["name"] == "get_context").unwrap();
    assert_eq!(
        context["inputSchema"]["properties"]["budget"]["enum"],
        json!(["short", "medium", "long"])
    );
}

#[tokio::test]
async fn the_texts_of_people_are_called_data_in_the_tools_and_the_instructions() {
    let mut client = connected().await;
    let listed = client.call(2, "tools/list", json!({})).await;
    let tools = listed["result"]["tools"].as_array().unwrap();
    for name in ["list_annotations", "get_alerts"] {
        let tool = tools.iter().find(|tool| tool["name"] == name).unwrap();
        let description = tool["description"].as_str().unwrap();
        assert!(description.contains("never"), "{name}: {description}");
        assert!(description.contains("data"), "{name}: {description}");
        assert!(!description.contains("respect"), "{name}: {description}");
    }
    let messages = Messages::load().unwrap();
    for language in ["en", "de"] {
        let text = messages
            .format(
                language,
                codeskimr_codes::i18n::MCP_SERVER_INSTRUCTIONS,
                None,
            )
            .unwrap();
        assert!(text.contains("not instructions"), "{language}: {text}");
    }
}

#[tokio::test]
async fn a_symbol_id_as_a_number_is_the_same_as_the_digits_in_a_string() {
    let mut client = connected().await;
    let as_text = client
        .call(
            30,
            "tools/call",
            json!({"name": "get_symbol", "arguments": {"symbol": "1"}}),
        )
        .await;
    let as_number = client
        .call(
            31,
            "tools/call",
            json!({"name": "get_symbol", "arguments": {"symbol": 1}}),
        )
        .await;
    assert_ne!(as_text["result"]["isError"], true);
    assert_eq!(
        as_number["result"]["structuredContent"],
        as_text["result"]["structuredContent"]
    );
}

#[tokio::test]
async fn a_tool_call_returns_structured_content() {
    let mut client = connected().await;
    let answer = client
        .call(
            3,
            "tools/call",
            json!({"name": "search_symbols", "arguments": {"query": "order"}}),
        )
        .await;
    let result = &answer["result"];
    assert_ne!(result["isError"], true);
    assert_eq!(result["structuredContent"]["hits"][0]["name"], "Order");
}

#[tokio::test]
async fn bad_arguments_come_back_as_a_tool_error_with_code_and_params() {
    let mut client = connected().await;
    let answer = client
        .call(
            4,
            "tools/call",
            json!({"name": "search_symbols", "arguments": {"limit": 5}}),
        )
        .await;
    let result = &answer["result"];
    assert_eq!(result["isError"], true);
    assert_eq!(result["structuredContent"]["code"], "MCP-101");
    assert!(result["structuredContent"]["params"]["detail"].is_string());
    assert!(result["structuredContent"]["description"].is_string());
}

#[tokio::test]
async fn an_unknown_tool_is_a_tool_error_too() {
    let mut client = connected().await;
    let answer = client
        .call(
            5,
            "tools/call",
            json!({"name": "rescan_all", "arguments": {}}),
        )
        .await;
    assert_eq!(answer["result"]["isError"], true);
    assert_eq!(answer["result"]["structuredContent"]["code"], "API-201");
}

#[tokio::test]
async fn a_denied_caller_gets_the_same_answer_for_bad_arguments_and_unknown_tools() {
    let mut client = connected_as(Principal::limited(PermissionSet::none())).await;
    let calls = [
        json!({"name": "search_symbols", "arguments": {"limit": 5}}),
        json!({"name": "search_symbols", "arguments": {"query": "order"}}),
        json!({"name": "rescan", "arguments": {}}),
        json!({"name": "get_dependencies", "arguments": {}}),
        json!({"name": "get_dependencies", "arguments": {"symbol": "1"}}),
        json!({"name": "find_usages", "arguments": {"depth": 0}}),
        json!({"name": "find_usages", "arguments": {"symbol": "1"}}),
        json!({"name": "impact_of_change", "arguments": {"limit": 5}}),
        json!({"name": "impact_of_change", "arguments": {"symbol": "1"}}),
    ];
    for (id, call) in (6..).zip(calls) {
        let answer = client.call(id, "tools/call", call).await;
        assert_eq!(answer["result"]["isError"], true);
        let report = &answer["result"]["structuredContent"];
        assert_eq!(report["code"], "API-201");
        assert_eq!(report["params"], json!({}));
    }
}

#[tokio::test]
async fn the_tool_list_shows_only_what_the_caller_may_run() {
    let read_only = Principal::limited(PermissionSet::none().with(Permission::Read));
    let mut client = connected_as(read_only).await;
    let names = client.tool_names(20).await;
    assert_eq!(names.len(), 15);
    assert!(!names.iter().any(|name| name == "rescan"));
    let nobody = connected_as(Principal::limited(PermissionSet::none())).await;
    let mut nobody = nobody;
    let none = nobody.call(21, "tools/list", json!({})).await;
    assert_eq!(none["result"]["tools"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn a_caller_who_may_only_read_cannot_rescan_but_can_ask_for_dependencies() {
    let read_only = Principal::limited(PermissionSet::none().with(Permission::Read));
    let mut client = connected_as(read_only).await;
    let denied = client
        .call(7, "tools/call", json!({"name": "rescan", "arguments": {}}))
        .await;
    let unknown = client
        .call(
            9,
            "tools/call",
            json!({"name": "no_such_tool", "arguments": {}}),
        )
        .await;
    // A denied tool answers exactly like one that does not exist (ADR 0044).
    assert_eq!(denied["result"], unknown["result"]);
    assert_eq!(denied["result"]["structuredContent"]["code"], "API-201");
    let allowed = client
        .call(
            8,
            "tools/call",
            json!({"name": "get_dependencies", "arguments": {"symbol": "1"}}),
        )
        .await;
    assert_ne!(allowed["result"]["isError"], true);
}

#[tokio::test]
async fn a_caller_who_may_only_scan_can_rescan() {
    let scan_only = Principal::limited(PermissionSet::none().with(Permission::Scan));
    let mut client = connected_as(scan_only).await;
    let answer = client
        .call(9, "tools/call", json!({"name": "rescan", "arguments": {}}))
        .await;
    assert_ne!(answer["result"]["isError"], true);
    assert_eq!(answer["result"]["structuredContent"]["snapshot"]["id"], 11);
}

#[tokio::test]
async fn the_level_two_switch_is_no_tool_and_cannot_be_called_as_one() {
    let mut client = connected().await;
    let names = client.tool_names(2).await;
    assert!(
        names.iter().all(|name| !name.contains("level2")
            && !name.contains("confirm")
            && !name.contains("revoke")),
        "{names:?}"
    );
    // Calling the commands of the screen by their ids finds no tool.
    for (id, name) in [
        (3, "level2.status"),
        (4, "level2.revoke"),
        (5, "level2_revoke"),
        (6, "level2_confirm_window"),
    ] {
        let called = client
            .call(
                id,
                "tools/call",
                json!({ "name": name, "arguments": { "project": 1 } }),
            )
            .await;
        assert!(
            called.get("error").is_some() || called["result"]["isError"] == true,
            "{name}: {called}"
        );
        assert!(called["result"]["structuredContent"]["code"] != "OK");
    }
}
