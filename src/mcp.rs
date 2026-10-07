//! # Model Context Protocol (MCP) Server
//!
//! Provides a standards-compliant Model Context Protocol (MCP) stdio server
//! enabling AI agents (Claude Desktop, Cursor, Antigravity, etc.) to inspect,
//! edit, search-and-replace, and batch-modify static HTML and SVG files directly.

use crate::processor::HtmlProcessor;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Value, code: i64, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(json!({
                "code": code,
                "message": message
            })),
        }
    }
}

/// Launches the MCP stdio server loop
pub fn run_mcp_server() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
            handle_request(&request, &mut writer)?;
        }

        line.clear();
    }

    Ok(())
}

fn handle_request<W: Write>(req: &JsonRpcRequest, writer: &mut W) -> io::Result<()> {
    let id = req.id.clone().unwrap_or(Value::Null);

    match req.method.as_str() {
        "initialize" => {
            let result = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "do-html-editable-cli",
                    "version": "0.1.1"
                }
            });
            send_response(writer, JsonRpcResponse::success(id, result))?;
        }

        "notifications/initialized" => {
            // Client acknowledgment notification; no response needed
        }

        "ping" => {
            send_response(writer, JsonRpcResponse::success(id, json!({})))?;
        }

        "tools/list" => {
            let tools = get_tool_definitions();
            send_response(writer, JsonRpcResponse::success(id, json!({ "tools": tools })))?;
        }

        "tools/call" => {
            let params = req.params.as_ref().unwrap_or(&Value::Null);
            let tool_name = params
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            match execute_tool(tool_name, &args) {
                Ok(text_output) => {
                    let result = json!({
                        "content": [
                            {
                                "type": "text",
                                "text": text_output
                            }
                        ],
                        "isError": false
                    });
                    send_response(writer, JsonRpcResponse::success(id, result))?;
                }
                Err(err_msg) => {
                    let result = json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Error: {}", err_msg)
                            }
                        ],
                        "isError": true
                    });
                    send_response(writer, JsonRpcResponse::success(id, result))?;
                }
            }
        }

        _ => {
            send_response(
                writer,
                JsonRpcResponse::error(id, -32601, &format!("Method '{}' not found", req.method)),
            )?;
        }
    }

    Ok(())
}

fn send_response<W: Write>(writer: &mut W, resp: JsonRpcResponse) -> io::Result<()> {
    let serialized = serde_json::to_string(&resp)?;
    writeln!(writer, "{}", serialized)?;
    writer.flush()?;
    Ok(())
}

/// Returns definitions of tools exposed over MCP
fn get_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "html_list_texts",
            "description": "Extract and enumerate all editable text elements in an HTML or SVG document with unique indexes (#0, #1, ...), CSS selectors, and current text preview.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Relative or absolute path to the target HTML or SVG file."
                    },
                    "filter": {
                        "type": "string",
                        "description": "Optional substring to filter text content."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Optional maximum number of elements to return."
                    }
                },
                "required": ["file_path"]
            }
        }),
        json!({
            "name": "html_get_text",
            "description": "Get the text content of a specific element in an HTML or SVG file by index (#0, #1) or CSS selector.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the target HTML or SVG file."
                    },
                    "target": {
                        "type": "string",
                        "description": "Element index (#0, #1, ...) or CSS selector (h1, p#desc, etc.)."
                    }
                },
                "required": ["file_path", "target"]
            }
        }),
        json!({
            "name": "html_set_text",
            "description": "Set or overwrite the text content of an element in an HTML or SVG file by index or CSS selector, saving changes to disk.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the target HTML or SVG file."
                    },
                    "target": {
                        "type": "string",
                        "description": "Element index (#0, #1, ...) or CSS selector."
                    },
                    "text": {
                        "type": "string",
                        "description": "The new text string to set."
                    },
                    "save_path": {
                        "type": "string",
                        "description": "Optional destination path to save to. If omitted, saves to file_path if in_place is true, or file_path.revised.html/svg if false."
                    },
                    "in_place": {
                        "type": "boolean",
                        "description": "If true, overwrites the source file directly."
                    }
                },
                "required": ["file_path", "target", "text"]
            }
        }),
        json!({
            "name": "html_replace_text",
            "description": "Search and replace text occurrences across the entire HTML or SVG document, or within a specific targeted element.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the target HTML or SVG file."
                    },
                    "old_text": {
                        "type": "string",
                        "description": "The exact substring to find."
                    },
                    "new_text": {
                        "type": "string",
                        "description": "The replacement substring."
                    },
                    "target": {
                        "type": "string",
                        "description": "Optional element index or CSS selector to limit replacement scope."
                    },
                    "save_path": {
                        "type": "string",
                        "description": "Optional destination path to save to."
                    },
                    "in_place": {
                        "type": "boolean",
                        "description": "If true, overwrites the source file directly."
                    }
                },
                "required": ["file_path", "old_text", "new_text"]
            }
        }),
        json!({
            "name": "html_batch_edit",
            "description": "Apply a batch of text edits (set or replace operations) atomically to an HTML or SVG file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the target HTML or SVG file."
                    },
                    "edits": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "target": {
                                    "type": "string",
                                    "description": "Element index (#0) or CSS selector"
                                },
                                "text": {
                                    "type": "string",
                                    "description": "New text to set (for set operation)"
                                },
                                "replace": {
                                    "type": "object",
                                    "properties": {
                                        "old": { "type": "string" },
                                        "new": { "type": "string" }
                                    },
                                    "required": ["old", "new"]
                                }
                            }
                        },
                        "description": "List of edit operations to apply in order."
                    },
                    "save_path": {
                        "type": "string",
                        "description": "Optional destination path."
                    },
                    "in_place": {
                        "type": "boolean",
                        "description": "If true, overwrites source file."
                    }
                },
                "required": ["file_path", "edits"]
            }
        }),
        json!({
            "name": "html_make_editable",
            "description": "Inject the air-gapped in-browser visual editing toolbar into an HTML or SVG file, creating filename.editable.html and optionally launching default browser.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the HTML or SVG file."
                    },
                    "no_open": {
                        "type": "boolean",
                        "description": "If true, skips launching the system browser."
                    }
                },
                "required": ["file_path"]
            }
        })
    ]
}

/// Dispatches execution of MCP tools
fn execute_tool(tool_name: &str, args: &Value) -> Result<String, String> {
    match tool_name {
        "html_list_texts" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'file_path'")?;
            let filter = args.get("filter").and_then(|v| v.as_str());
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);

            let proc = HtmlProcessor::from_file(file_path)?;
            let texts = proc.list_texts(filter, limit);
            serde_json::to_string_pretty(&texts).map_err(|e| e.to_string())
        }

        "html_get_text" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'file_path'")?;
            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'target'")?;

            let proc = HtmlProcessor::from_file(file_path)?;
            let text = proc.get_text(target)?;
            Ok(json!({ "target": target, "text": text }).to_string())
        }

        "html_set_text" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'file_path'")?;
            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'target'")?;
            let new_text = args
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'text'")?;
            let in_place = args
                .get("in_place")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let save_path = args.get("save_path").and_then(|v| v.as_str());

            let proc = HtmlProcessor::from_file(file_path)?;
            proc.set_text(target, new_text)?;

            let dest = resolve_save_dest(file_path, save_path, in_place);
            let saved = proc.save(Some(&dest))?;

            Ok(json!({
                "status": "success",
                "target": target,
                "text": new_text,
                "saved_to": saved.display().to_string()
            })
            .to_string())
        }

        "html_replace_text" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'file_path'")?;
            let old_text = args
                .get("old_text")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'old_text'")?;
            let new_text = args
                .get("new_text")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'new_text'")?;
            let target = args.get("target").and_then(|v| v.as_str());
            let in_place = args
                .get("in_place")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let save_path = args.get("save_path").and_then(|v| v.as_str());

            let proc = HtmlProcessor::from_file(file_path)?;
            let count = proc.replace_text(old_text, new_text, target)?;

            let dest = resolve_save_dest(file_path, save_path, in_place);
            let saved = proc.save(Some(&dest))?;

            Ok(json!({
                "status": "success",
                "replacements": count,
                "old_text": old_text,
                "new_text": new_text,
                "saved_to": saved.display().to_string()
            })
            .to_string())
        }

        "html_batch_edit" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'file_path'")?;
            let edits = args
                .get("edits")
                .and_then(|v| v.as_array())
                .ok_or("Missing required argument 'edits' (must be array)")?;
            let in_place = args
                .get("in_place")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let save_path = args.get("save_path").and_then(|v| v.as_str());

            let proc = HtmlProcessor::from_file(file_path)?;
            let mut applied = 0;

            for edit in edits {
                if let Some(target) = edit.get("target").and_then(|v| v.as_str()) {
                    if let Some(new_text) = edit.get("text").and_then(|v| v.as_str()) {
                        proc.set_text(target, new_text)?;
                        applied += 1;
                    } else if let Some(replace_obj) = edit.get("replace") {
                        if let (Some(old_val), Some(new_val)) = (
                            replace_obj.get("old").and_then(|v| v.as_str()),
                            replace_obj.get("new").and_then(|v| v.as_str()),
                        ) {
                            proc.replace_text(old_val, new_val, Some(target))?;
                            applied += 1;
                        }
                    }
                }
            }

            let dest = resolve_save_dest(file_path, save_path, in_place);
            let saved = proc.save(Some(&dest))?;

            Ok(json!({
                "status": "success",
                "applied_edits": applied,
                "saved_to": saved.display().to_string()
            })
            .to_string())
        }

        "html_make_editable" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required argument 'file_path'")?;
            let no_open = args
                .get("no_open")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let (editable_path, export_name) =
                crate::derive_file_paths(Path::new(file_path));

            let raw_html = std::fs::read_to_string(file_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            let modified = crate::inject_editor_runtime(&raw_html, &export_name);
            std::fs::write(&editable_path, modified)
                .map_err(|e| format!("Failed to write editable file: {}", e))?;

            if !no_open {
                opener::open(&editable_path).ok();
            }

            Ok(json!({
                "status": "success",
                "editable_file": editable_path.display().to_string(),
                "clean_export_name": export_name
            })
            .to_string())
        }

        _ => Err(format!("Unknown tool '{}'", tool_name)),
    }
}

fn resolve_save_dest(source: &str, save_path: Option<&str>, in_place: bool) -> PathBuf {
    if let Some(sp) = save_path {
        PathBuf::from(sp)
    } else if in_place {
        PathBuf::from(source)
    } else {
        let p = Path::new(source);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("document");
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("html");
        let parent = p.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}.revised.{}", stem, ext))
    }
}
