//! Lightweight stdio MCP server for local agent play.
//!
//! Implements the MCP protocol (JSON-RPC over stdin/stdout) with 5 game tools:
//! - get_game_state: returns the current game state snapshot
//! - spawn_unit: spawn a unit at the player's base
//! - move_unit: move a unit to a target hex
//! - attack_target: order a unit to attack a target
//! - harvest: order a worker to harvest a resource

use bf_types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

use crate::GameEngine;

/// JSON-RPC request.
#[derive(Deserialize, Debug)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC response.
#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// MCP tool definition.
#[derive(Serialize)]
struct ToolDef {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// Run the MCP stdio server loop. Blocks until stdin is closed.
pub fn run_stdio_server(engine: &mut GameEngine, player_slot: u8) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
                let _ = stdout.flush();
                continue;
            }
        };

        let id = request.id.clone().unwrap_or(Value::Null);
        let response = handle_request(engine, player_slot, &request);

        let resp = JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: response.result,
            error: response.error,
        };
        let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
        let _ = stdout.flush();
    }
}

struct HandlerResult {
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

fn handle_request(engine: &mut GameEngine, player_slot: u8, req: &JsonRpcRequest) -> HandlerResult {
    match req.method.as_str() {
        "initialize" => HandlerResult {
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "battleform-engine",
                    "version": "0.1.0"
                }
            })),
            error: None,
        },
        "tools/list" => {
            let tools = get_tool_definitions();
            HandlerResult {
                result: Some(serde_json::json!({ "tools": tools })),
                error: None,
            }
        }
        "tools/call" => {
            let tool_name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = req.params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
            handle_tool_call(engine, player_slot, tool_name, &arguments)
        }
        "notifications/initialized" | "ping" => HandlerResult {
            result: Some(Value::Object(Default::default())),
            error: None,
        },
        _ => HandlerResult {
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
        },
    }
}

fn get_tool_definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "get_game_state".into(),
            description: "Get the current game state snapshot including map, units, buildings, and resources.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDef {
            name: "spawn_unit".into(),
            description: "Spawn a new unit at your base. Costs energy.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "unit_type": {
                        "type": "string",
                        "enum": ["worker", "soldier", "scout"],
                        "description": "Type of unit to spawn"
                    }
                },
                "required": ["unit_type"]
            }),
        },
        ToolDef {
            name: "move_unit".into(),
            description: "Move a unit to a target hex position.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "unit_id": { "type": "string", "description": "ID of the unit to move" },
                    "x": { "type": "integer", "description": "Target hex column" },
                    "y": { "type": "integer", "description": "Target hex row" }
                },
                "required": ["unit_id", "x", "y"]
            }),
        },
        ToolDef {
            name: "attack_target".into(),
            description: "Order a unit to attack a target unit or building.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "unit_id": { "type": "string", "description": "ID of the attacking unit" },
                    "target_id": { "type": "string", "description": "ID of the target unit or building" }
                },
                "required": ["unit_id", "target_id"]
            }),
        },
        ToolDef {
            name: "harvest".into(),
            description: "Order a worker to harvest energy from a resource node.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "unit_id": { "type": "string", "description": "ID of the worker unit" },
                    "resource_id": { "type": "string", "description": "ID of the resource node" }
                },
                "required": ["unit_id", "resource_id"]
            }),
        },
    ]
}

fn handle_tool_call(engine: &mut GameEngine, player_slot: u8, tool_name: &str, args: &Value) -> HandlerResult {
    match tool_name {
        "get_game_state" => {
            let snapshot = engine.full_snapshot();
            HandlerResult {
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string(&snapshot).unwrap_or_default()
                    }]
                })),
                error: None,
            }
        }
        "spawn_unit" => {
            let unit_type_str = args.get("unit_type").and_then(|v| v.as_str()).unwrap_or("worker");
            let unit_type = match unit_type_str {
                "worker" => UnitType::Worker,
                "soldier" => UnitType::Soldier,
                "scout" => UnitType::Scout,
                _ => {
                    return HandlerResult {
                        result: Some(mcp_error(&format!("Unknown unit type: {}", unit_type_str))),
                        error: None,
                    };
                }
            };
            engine.queue_command(Command::SpawnUnit { player_slot, unit_type });
            HandlerResult {
                result: Some(mcp_text("Unit spawn queued")),
                error: None,
            }
        }
        "move_unit" => {
            let unit_id = args.get("unit_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let x = args.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = args.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            engine.queue_command(Command::MoveUnit { player_slot, unit_id, target_x: x, target_y: y });
            HandlerResult {
                result: Some(mcp_text("Move command queued")),
                error: None,
            }
        }
        "attack_target" => {
            let unit_id = args.get("unit_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let target_id = args.get("target_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            engine.queue_command(Command::AttackTarget { player_slot, unit_id, target_id });
            HandlerResult {
                result: Some(mcp_text("Attack command queued")),
                error: None,
            }
        }
        "harvest" => {
            let unit_id = args.get("unit_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let resource_id = args.get("resource_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            engine.queue_command(Command::Harvest { player_slot, unit_id, resource_id });
            HandlerResult {
                result: Some(mcp_text("Harvest command queued")),
                error: None,
            }
        }
        _ => HandlerResult {
            result: Some(mcp_error(&format!("Unknown tool: {}", tool_name))),
            error: None,
        },
    }
}

fn mcp_text(msg: &str) -> Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": msg }]
    })
}

fn mcp_error(msg: &str) -> Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": msg }],
        "isError": true
    })
}
