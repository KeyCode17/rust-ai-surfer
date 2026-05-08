use ras_mcp::infrastructure::stdio_protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

#[test]
fn parses_jsonrpc_request() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
    let req: JsonRpcRequest = serde_json::from_str(raw).expect("parse");
    assert_eq!(req.method, "tools/list");
    assert_eq!(req.id, json!(1));
}

#[test]
fn ok_response_has_result_no_error() {
    let r = JsonRpcResponse::ok(json!(1), json!({"tools": []}));
    let s = serde_json::to_string(&r).expect("ser");
    assert!(s.contains(r#""result""#));
    assert!(!s.contains(r#""error""#));
}

#[test]
fn error_response_has_error_no_result() {
    let r = JsonRpcResponse::err(json!(1), -32601, "method not found");
    let s = serde_json::to_string(&r).expect("ser");
    assert!(s.contains(r#""error""#));
    assert!(!s.contains(r#""result""#));
    assert!(s.contains("-32601"));
}
