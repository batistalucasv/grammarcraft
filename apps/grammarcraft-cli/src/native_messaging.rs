use std::io::{self, BufReader, BufWriter, Read, Write};
use std::process::ExitCode;

use grammarcraft_engine::Session;
use serde_json::{Value, json};

/// Standard WebExtension Native Messaging per-message byte limit (1 MiB).
pub const MAX_NATIVE_MESSAGE_BYTES: usize = 1_048_576;

const ALLOWED_COMMANDS: &[&str] = &[
    "doc.get",
    "doc.set_text",
    "goals.set",
    "review.apply",
    "review.apply_all",
    "review.check",
    "review.dismiss",
    "stats.measure",
    "translate.configure",
    "translate.live",
    "translate.status",
];

pub(crate) fn run(args: &[String]) -> Result<ExitCode, String> {
    if !args.is_empty() {
        return Err(format!("unknown native-messaging option `{}`", args[0]));
    }
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    serve_stream(&mut reader, &mut writer)?;
    Ok(ExitCode::SUCCESS)
}

pub fn serve_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<(), String> {
    let mut session = Session::new();
    while let Some(payload) = read_frame(reader)? {
        let reply = match serde_json::from_slice::<Value>(&payload) {
            Ok(msg) => dispatch_message(&mut session, &msg),
            Err(err) => json!({
                "id": Value::Null,
                "ok": false,
                "error": format!("invalid JSON payload: {err}"),
            }),
        };
        write_frame(writer, &reply)?;
    }
    Ok(())
}

pub fn read_frame<R: Read>(reader: &mut R) -> Result<Option<Vec<u8>>, String> {
    let mut len_buf = [0u8; 4];
    let mut total_read = 0usize;
    while total_read < 4 {
        let Some(slice) = len_buf.get_mut(total_read..) else {
            return Err("internal native-messaging header buffer error".to_owned());
        };
        let n = reader
            .read(slice)
            .map_err(|e| format!("native-messaging stdin header read error: {e}"))?;
        if n == 0 {
            if total_read == 0 {
                return Ok(None);
            }
            return Err("unexpected EOF while reading native-messaging 4-byte header".to_owned());
        }
        total_read += n;
    }

    let declared_len = usize::try_from(u32::from_le_bytes(len_buf))
        .map_err(|_| "native-messaging payload length exceeds platform usize".to_owned())?;
    if declared_len == 0 {
        return Err("native-messaging message length must be > 0".to_owned());
    }
    if declared_len > MAX_NATIVE_MESSAGE_BYTES {
        return Err(format!(
            "native-messaging payload length {declared_len} exceeds limit of {MAX_NATIVE_MESSAGE_BYTES} bytes"
        ));
    }

    let mut body = Vec::new();
    body.try_reserve_exact(declared_len)
        .map_err(|e| format!("cannot allocate {declared_len}-byte native-messaging buffer: {e}"))?;
    body.resize(declared_len, 0);
    reader
        .read_exact(&mut body)
        .map_err(|e| format!("native-messaging payload read error: {e}"))?;
    Ok(Some(body))
}

pub fn write_frame<W: Write>(writer: &mut W, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|e| format!("failed to serialize native-messaging reply: {e}"))?;
    if bytes.len() > MAX_NATIVE_MESSAGE_BYTES {
        return Err(format!(
            "native-messaging reply length {} exceeds limit of {MAX_NATIVE_MESSAGE_BYTES} bytes",
            bytes.len()
        ));
    }
    let len_u32 = u32::try_from(bytes.len())
        .map_err(|_| "native-messaging reply exceeds u32 length".to_owned())?;
    writer
        .write_all(&len_u32.to_le_bytes())
        .map_err(|e| format!("native-messaging header write error: {e}"))?;
    writer
        .write_all(&bytes)
        .map_err(|e| format!("native-messaging body write error: {e}"))?;
    writer
        .flush()
        .map_err(|e| format!("native-messaging flush error: {e}"))?;
    Ok(())
}

fn dispatch_message(session: &mut Session, msg: &Value) -> Value {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let Some(command) = msg.get("command").and_then(Value::as_str) else {
        return json!({
            "id": id,
            "ok": false,
            "error": "`command` (string) is required",
        });
    };
    if !ALLOWED_COMMANDS.contains(&command) {
        return json!({
            "id": id,
            "ok": false,
            "error": format!("command `{command}` is not permitted over native-messaging"),
        });
    }
    let params = msg
        .get("params")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
    match session.run(command, &params) {
        Ok(result) => json!({ "id": id, "ok": true, "result": result }),
        Err(err) => json!({ "id": id, "ok": false, "error": err.to_string() }),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::io::Cursor;

    use serde_json::{Value, json};

    use super::{MAX_NATIVE_MESSAGE_BYTES, read_frame, serve_stream, write_frame};

    #[test]
    fn clean_eof_returns_none_and_terminates_loop() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Vec::<u8>::new();
        serve_stream(&mut input, &mut output).expect("clean EOF succeeds");
        assert!(output.is_empty());
    }

    #[test]
    fn rejects_payload_exceeding_max_native_message_bytes_without_allocating() {
        let oversized = u32::try_from(MAX_NATIVE_MESSAGE_BYTES + 1).unwrap();
        let mut input = Cursor::new(oversized.to_le_bytes().to_vec());
        let err = read_frame(&mut input).expect_err("oversized payload must be rejected");
        assert!(err.contains("exceeds limit"));
    }

    #[test]
    fn frames_and_dispatches_safe_commands_and_blocks_file_io() {
        let mut incoming = Vec::new();
        write_frame(
            &mut incoming,
            &json!({ "id": 1, "command": "doc.set_text", "params": { "text": "He recieve it." } }),
        )
        .unwrap();
        write_frame(
            &mut incoming,
            &json!({ "id": 2, "command": "review.check", "params": {} }),
        )
        .unwrap();
        write_frame(
            &mut incoming,
            &json!({ "id": 3, "command": "review.apply_all", "params": {} }),
        )
        .unwrap();
        write_frame(
            &mut incoming,
            &json!({ "id": 4, "command": "stats.measure", "params": {} }),
        )
        .unwrap();
        write_frame(
            &mut incoming,
            &json!({ "id": 5, "command": "doc.open", "params": { "path": "/etc/passwd" } }),
        )
        .unwrap();
        write_frame(
            &mut incoming,
            &json!({ "id": 6, "command": "translate.configure", "params": { "enabled": true, "target_lang": "en" } }),
        )
        .unwrap();
        write_frame(
            &mut incoming,
            &json!({ "id": 7, "command": "translate.live", "params": { "text": "Olá, bom dia.", "apply": false } }),
        )
        .unwrap();

        let mut reader = Cursor::new(incoming);
        let mut out = Vec::new();
        serve_stream(&mut reader, &mut out).expect("serve stream completes cleanly on EOF");

        let mut reply_reader = Cursor::new(out);
        let mut replies = Vec::<Value>::new();
        while let Some(raw) = read_frame(&mut reply_reader).expect("valid outbound frame") {
            replies.push(serde_json::from_slice(&raw).expect("valid JSON"));
        }

        assert_eq!(replies.len(), 7);
        assert_eq!(replies[0]["id"], 1);
        assert_eq!(replies[0]["ok"], true);
        assert_eq!(replies[1]["ok"], true);
        assert!(
            replies[1]["result"]["issues"]
                .as_array()
                .is_some_and(|arr| !arr.is_empty())
        );
        assert_eq!(replies[2]["ok"], true);
        assert!(replies[2]["result"]["applied_count"].as_u64().unwrap_or(0) >= 1);
        assert_eq!(replies[3]["ok"], true);
        assert_eq!(replies[4]["ok"], false);
        assert!(
            replies[4]["error"]
                .as_str()
                .is_some_and(|e| e.contains("not permitted"))
        );
        assert_eq!(replies[5]["ok"], true);
        assert_eq!(replies[5]["result"]["enabled"], true);
        assert_eq!(replies[6]["ok"], true);
        assert_eq!(
            replies[6]["result"]["translated_text"],
            "Hello, good morning."
        );
    }
}
