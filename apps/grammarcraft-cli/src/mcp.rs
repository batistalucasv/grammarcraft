use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use grammarcraft_engine::file_access::{self, AutomationRoots};
use grammarcraft_engine::{Session, registry};
use serde_json::{Value, json};

const MAX_LINE_BYTES: u64 = 2 * 1024 * 1024;
const PROTOCOL_VERSION: &str = "2024-11-05";

pub(crate) fn run(args: &[String]) -> Result<ExitCode, String> {
    let (read_root, write_root) = parse_roots(args)?;
    let roots = match AutomationRoots::new(Some(&read_root), Some(&write_root)) {
        Ok(Some(roots)) => roots,
        Ok(None) => return Err("both automation roots are required".to_owned()),
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(ExitCode::FAILURE);
        }
    };
    file_access::confine_process(roots);
    serve_stdio()
}

fn parse_roots(args: &[String]) -> Result<(PathBuf, PathBuf), String> {
    let mut read_root: Option<PathBuf> = None;
    let mut write_root: Option<PathBuf> = None;
    let mut idx = 0usize;
    while let Some(arg) = args.get(idx) {
        if let Some(val) = arg.strip_prefix("--automation-read-root=") {
            read_root = Some(non_empty_path("--automation-read-root", val)?);
        } else if arg == "--automation-read-root" {
            idx += 1;
            let val = args
                .get(idx)
                .ok_or_else(|| "`--automation-read-root` requires a directory".to_owned())?;
            read_root = Some(non_empty_path("--automation-read-root", val)?);
        } else if let Some(val) = arg.strip_prefix("--automation-write-root=") {
            write_root = Some(non_empty_path("--automation-write-root", val)?);
        } else if arg == "--automation-write-root" {
            idx += 1;
            let val = args
                .get(idx)
                .ok_or_else(|| "`--automation-write-root` requires a directory".to_owned())?;
            write_root = Some(non_empty_path("--automation-write-root", val)?);
        } else {
            return Err(format!("unknown mcp option `{arg}`"));
        }
        idx += 1;
    }

    let read =
        read_root.ok_or_else(|| "`mcp` requires `--automation-read-root <dir>`".to_owned())?;
    let write =
        write_root.ok_or_else(|| "`mcp` requires `--automation-write-root <dir>`".to_owned())?;
    Ok((read, write))
}

fn non_empty_path(flag: &str, value: &str) -> Result<PathBuf, String> {
    if value.trim().is_empty() {
        return Err(format!("`{flag}` must not be empty"));
    }
    Ok(PathBuf::from(value))
}

fn serve_stdio() -> Result<ExitCode, String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut writer = io::BufWriter::new(stdout.lock());
    let mut session = Session::new();

    while let Some(line) = read_bounded_line(&mut reader)? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(trimmed) {
            Ok(msg) => handle_message(&mut session, &msg),
            Err(err) => Some(json_rpc_error(
                Value::Null,
                -32700,
                &format!("parse error: {err}"),
            )),
        };
        if let Some(reply) = response {
            let encoded = serde_json::to_string(&reply)
                .map_err(|e| format!("failed to serialize response: {e}"))?;
            writeln!(writer, "{encoded}").map_err(|e| format!("stdout write error: {e}"))?;
            writer
                .flush()
                .map_err(|e| format!("stdout flush error: {e}"))?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn read_bounded_line<R: BufRead>(reader: &mut R) -> Result<Option<String>, String> {
    let mut buf = Vec::new();
    let read = reader
        .take(MAX_LINE_BYTES + 1)
        .read_until(b'\n', &mut buf)
        .map_err(|e| format!("stdin read error: {e}"))?;
    if read == 0 {
        return Ok(None);
    }
    if (read as u64) > MAX_LINE_BYTES {
        return Err(format!("stdin message exceeded {MAX_LINE_BYTES} bytes"));
    }
    let line = String::from_utf8(buf).map_err(|e| format!("stdin is not valid UTF-8: {e}"))?;
    Ok(Some(line))
}

fn handle_message(session: &mut Session, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned();
    let Some(method) = msg.get("method").and_then(Value::as_str) else {
        return id.map(|req_id| json_rpc_error(req_id, -32600, "invalid request: missing method"));
    };

    if method == "notifications/initialized" && id.is_none() {
        return None;
    }

    let req_id = id?;
    let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
    match method {
        "initialize" => Some(json_rpc_ok(
            req_id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "grammarcraft-cli",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }),
        )),
        "ping" => Some(json_rpc_ok(req_id, json!({}))),
        "tools/list" => Some(json_rpc_ok(req_id, tools_list())),
        "tools/call" => Some(json_rpc_ok(req_id, handle_tool_call(session, &params))),
        other => Some(json_rpc_error(
            req_id,
            -32601,
            &format!("method not found: {other}"),
        )),
    }
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "engine_commands",
                "description": "List all registered GrammarCraft engine commands and parameter schemas.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "engine_execute",
                "description": "Execute a GrammarCraft engine command by id under configured automation roots.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string" },
                        "params": { "type": "object" }
                    },
                    "required": ["command"]
                }
            }
        ]
    })
}

fn handle_tool_call(session: &mut Session, params: &Value) -> Value {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return tool_error("missing tool `name`");
    };
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match name {
        "engine_commands" => match serde_json::to_string_pretty(&registry().describe()) {
            Ok(text) => tool_ok(text),
            Err(err) => tool_error(&format!("failed to format commands: {err}")),
        },
        "engine_execute" => {
            let Some(command) = args.get("command").and_then(Value::as_str) else {
                return tool_error("`command` (string) is required");
            };
            let cmd_params = args
                .get("params")
                .cloned()
                .filter(|v| !v.is_null())
                .unwrap_or_else(|| json!({}));
            match session.run(command, &cmd_params) {
                Ok(result) => match serde_json::to_string_pretty(&result) {
                    Ok(text) => tool_ok(text),
                    Err(err) => tool_error(&format!("failed to format result: {err}")),
                },
                Err(err) => tool_error(&err.to_string()),
            }
        }
        other => tool_error(&format!("unknown tool `{other}`")),
    }
}

fn tool_ok(text: String) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
    })
}

fn tool_error(message: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true
    })
}

fn json_rpc_ok(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn json_rpc_error(id: Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}
