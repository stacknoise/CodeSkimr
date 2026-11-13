//! The server speaks MCP over a byte stream, as it does over stdio.

#[cfg(test)]
mod common;

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
}

#[cfg(test)]
async fn connected() -> Client {
    let messages = Messages::load().unwrap();
    let server = McpServer::new(tools(), &messages).unwrap();
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

#[tokio::test]
async fn lists_exactly_the_three_read_only_tools_with_schemas() {
    let mut client = connected().await;
    let listed = client.call(2, "tools/list", json!({})).await;
    let tools = listed["result"]["tools"].as_array().unwrap();
    let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["search_symbols", "get_symbol", "get_architecture"]);
    for tool in tools {
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert_ne!(tool["description"].as_str().unwrap(), "");
    }
    assert_eq!(tools[0]["inputSchema"]["required"][0], "query");
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
        .call(5, "tools/call", json!({"name": "rescan", "arguments": {}}))
        .await;
    assert_eq!(answer["result"]["isError"], true);
    assert_eq!(answer["result"]["structuredContent"]["code"], "MCP-101");
}
