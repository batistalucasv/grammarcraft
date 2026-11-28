use std::io::{self, Read};
use std::process::ExitCode;

use grammarcraft_engine::{MAX_DOCUMENT_BYTES, Session};
use serde_json::{Map, Value, json};

const QUICK_REVIEW_USAGE: &str = "\
grammarcraft-cli quick-review — quick review text, a file, or stdin

USAGE:
  grammarcraft-cli quick-review (--text <TEXT> | --file <PATH> | --stdin) [OPTIONS]

OPTIONS:
  --text <TEXT>            inline text to review
  --file <PATH>            path to .txt/.md/.docx/.grammarcraft.json file
  --stdin                  read UTF-8 text from stdin (max MAX_DOCUMENT_BYTES)
  --lang <TAG>             override document language tag (e.g. en-US, pt-BR, es-ES)
  --audience <AUDIENCE>    writing goal audience (general, specialized, technical_executive)
  --formality <FORMALITY>  writing goal formality (informal, neutral, formal)
  --apply-all              apply top suggestions and print corrected text
  -h, --help               print this help
";

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputSource {
    Text(String),
    File(String),
    Stdin,
}

#[derive(Debug, Default)]
struct QuickReviewArgs {
    source: Option<InputSource>,
    lang: Option<String>,
    audience: Option<String>,
    formality: Option<String>,
    apply_all: bool,
    help: bool,
}

pub(crate) fn run(args: &[String]) -> Result<ExitCode, String> {
    let mut parsed = parse_args(args)?;
    if parsed.help {
        print!("{QUICK_REVIEW_USAGE}");
        return Ok(ExitCode::SUCCESS);
    }

    let source = parsed.source.take().ok_or_else(|| {
        format!(
            "`quick-review` requires one of --text <TEXT>, --file <PATH>, or --stdin\n\n{QUICK_REVIEW_USAGE}"
        )
    })?;

    let mut session = Session::new();
    match source {
        InputSource::Text(text) => {
            if !text.is_empty()
                && let Err(err) = session.run("doc.set_text", &json!({ "text": text }))
            {
                eprintln!("error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        }
        InputSource::File(path) => {
            if let Err(err) = session.run("doc.open", &json!({ "path": path })) {
                eprintln!("error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        }
        InputSource::Stdin => {
            let text = match read_bounded_stdin() {
                Ok(text) => text,
                Err(err) => {
                    eprintln!("error: {err}");
                    return Ok(ExitCode::FAILURE);
                }
            };
            if !text.is_empty()
                && let Err(err) = session.run("doc.set_text", &json!({ "text": text }))
            {
                eprintln!("error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        }
    }

    if let Some(goals_payload) = build_goals_payload(&parsed)
        && let Err(err) = session.run("goals.set", &goals_payload)
    {
        eprintln!("error: {err}");
        return Ok(ExitCode::FAILURE);
    }

    if parsed.apply_all {
        if !session.doc().is_empty()
            && let Err(err) = session.run("review.apply_all", &json!({}))
        {
            eprintln!("error: {err}");
            return Ok(ExitCode::FAILURE);
        }
        let doc = match session.run("doc.get", &json!({})) {
            Ok(doc) => doc,
            Err(err) => {
                eprintln!("error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        };
        let corrected = doc.get("text").and_then(Value::as_str).unwrap_or("");
        if corrected.ends_with('\n') {
            print!("{corrected}");
        } else {
            println!("{corrected}");
        }
        return Ok(ExitCode::SUCCESS);
    }

    let report = match session.run("review.check", &json!({})) {
        Ok(report) => report,
        Err(err) => {
            eprintln!("error: {err}");
            return Ok(ExitCode::FAILURE);
        }
    };
    let compact = json!({
        "language": report.get("language").cloned().unwrap_or(Value::Null),
        "stats": report.get("stats").cloned().unwrap_or(Value::Null),
        "issues": report.get("issues").cloned().unwrap_or_else(|| json!([])),
        "score": report.get("score").cloned().unwrap_or(Value::Null),
        "tone": report.get("tone").cloned().unwrap_or(Value::Null),
    });
    let line = serde_json::to_string(&compact)
        .map_err(|error| format!("could not print compact JSON: {error}"))?;
    println!("{line}");
    Ok(ExitCode::SUCCESS)
}

fn build_goals_payload(parsed: &QuickReviewArgs) -> Option<Value> {
    let mut map = Map::new();
    if let Some(lang) = &parsed.lang {
        map.insert("language".to_owned(), Value::String(lang.clone()));
    }
    if let Some(audience) = &parsed.audience {
        map.insert("audience".to_owned(), Value::String(audience.clone()));
    }
    if let Some(formality) = &parsed.formality {
        map.insert("formality".to_owned(), Value::String(formality.clone()));
    }
    (!map.is_empty()).then_some(Value::Object(map))
}

fn read_bounded_stdin() -> Result<String, String> {
    let limit = u64::try_from(MAX_DOCUMENT_BYTES)
        .map_err(|_| "document byte limit exceeds u64".to_owned())?;
    let mut buf = Vec::new();
    io::stdin()
        .take(limit.saturating_add(1))
        .read_to_end(&mut buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;
    if buf.len() > MAX_DOCUMENT_BYTES {
        return Err(format!(
            "stdin input exceeds maximum size of {MAX_DOCUMENT_BYTES} bytes"
        ));
    }
    String::from_utf8(buf).map_err(|e| format!("stdin is not valid UTF-8: {e}"))
}

fn parse_args(args: &[String]) -> Result<QuickReviewArgs, String> {
    let mut parsed = QuickReviewArgs::default();
    let mut i = 0usize;
    while let Some(arg) = args.get(i) {
        match arg.as_str() {
            "-h" | "--help" => {
                parsed.help = true;
                return Ok(parsed);
            }
            "--stdin" => set_source(&mut parsed.source, InputSource::Stdin)?,
            "--apply-all" => parsed.apply_all = true,
            "--text" => {
                let val = next_val(args, &mut i, "--text")?;
                set_source(&mut parsed.source, InputSource::Text(val.to_owned()))?;
            }
            other if other.starts_with("--text=") => {
                let val = other.trim_start_matches("--text=");
                set_source(&mut parsed.source, InputSource::Text(val.to_owned()))?;
            }
            "--file" => {
                let val = next_val(args, &mut i, "--file")?;
                set_source(&mut parsed.source, InputSource::File(val.to_owned()))?;
            }
            other if other.starts_with("--file=") => {
                let val = other.trim_start_matches("--file=");
                set_source(&mut parsed.source, InputSource::File(val.to_owned()))?;
            }
            "--lang" => {
                let val = next_val(args, &mut i, "--lang")?;
                parsed.lang = Some(val.to_owned());
            }
            other if other.starts_with("--lang=") => {
                parsed.lang = Some(other.trim_start_matches("--lang=").to_owned());
            }
            "--audience" => {
                let val = next_val(args, &mut i, "--audience")?;
                parsed.audience = Some(val.to_owned());
            }
            other if other.starts_with("--audience=") => {
                parsed.audience = Some(other.trim_start_matches("--audience=").to_owned());
            }
            "--formality" => {
                let val = next_val(args, &mut i, "--formality")?;
                parsed.formality = Some(val.to_owned());
            }
            other if other.starts_with("--formality=") => {
                parsed.formality = Some(other.trim_start_matches("--formality=").to_owned());
            }
            other => {
                return Err(format!(
                    "unknown quick-review option `{other}`\n\n{QUICK_REVIEW_USAGE}"
                ));
            }
        }
        i += 1;
    }
    Ok(parsed)
}

fn set_source(slot: &mut Option<InputSource>, next: InputSource) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!(
            "specify only one of --text, --file, or --stdin\n\n{QUICK_REVIEW_USAGE}"
        ));
    }
    *slot = Some(next);
    Ok(())
}

fn next_val<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    *i += 1;
    args.get(*i)
        .map(String::as_str)
        .ok_or_else(|| format!("`{flag}` requires a value\n\n{QUICK_REVIEW_USAGE}"))
}
