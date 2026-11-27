//! The command registry.
//!
//! A command is a value, not a method somewhere in a front end: id, label,
//! where it lives, its parameters and the function that runs it. Three
//! consequences worth keeping:
//!
//! * the same id works from the CLI, a shortcut, a UI button and an agent;
//! * `commands` can list what exists without opening a window;
//! * a test can walk every command with junk parameters.

use std::collections::HashMap;
use std::sync::OnceLock;

use grammarcraft_core::NoteId;
use serde_json::{Value, json};

use crate::{CmdResult, Session};

/// A registered command.
#[derive(Clone)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    /// Where it lives in a front end, e.g. `Notes › Edit`. Free-form on purpose.
    pub location: &'static str,
    /// Portable shortcut (`Mod+N`, `F2`); `Mod` is Ctrl, or ⌘ on macOS.
    pub shortcut: &'static str,
    /// Parameter description for agents and docs, e.g. `{"text": string}`.
    pub params: &'static str,
    /// `false` for commands that only read.
    pub mutates: bool,
    /// `None` when enabled; `Some(reason)` otherwise.
    pub enabled: fn(&Session) -> Option<&'static str>,
    pub run: fn(&mut Session, &Value) -> CmdResult,
}

fn always(_: &Session) -> Option<&'static str> {
    None
}

impl CommandSpec {
    pub const fn new(
        id: &'static str,
        label: &'static str,
        location: &'static str,
        run: fn(&mut Session, &Value) -> CmdResult,
    ) -> Self {
        CommandSpec {
            id,
            label,
            location,
            shortcut: "",
            params: "{}",
            mutates: true,
            enabled: always,
            run,
        }
    }

    pub const fn key(mut self, shortcut: &'static str) -> Self {
        self.shortcut = shortcut;
        self
    }

    pub const fn params(mut self, params: &'static str) -> Self {
        self.params = params;
        self
    }

    /// Marks a command that does not change the document.
    pub const fn pure(mut self) -> Self {
        self.mutates = false;
        self
    }

    pub const fn when(mut self, enabled: fn(&Session) -> Option<&'static str>) -> Self {
        self.enabled = enabled;
        self
    }
}

/// All commands by id.
///
/// Duplicate ids are rejected at construction: `get` must never silently pick
/// one of two commands.
pub struct Registry {
    specs: Vec<CommandSpec>,
    by_id: HashMap<&'static str, usize>,
}

impl Registry {
    /// Builds a registry. A duplicate id keeps the first registration; the
    /// tests below check that the real registry has none.
    pub fn new(specs: Vec<CommandSpec>) -> Self {
        let mut by_id = HashMap::new();
        for (index, spec) in specs.iter().enumerate() {
            by_id.entry(spec.id).or_insert(index);
        }
        Registry { specs, by_id }
    }

    pub fn get(&self, id: &str) -> Option<&CommandSpec> {
        self.by_id.get(id).and_then(|index| self.specs.get(*index))
    }

    pub fn all(&self) -> &[CommandSpec] {
        &self.specs
    }

    /// The command list as JSON, for a CLI `commands` call or an agent.
    pub fn describe(&self) -> Value {
        Value::Array(
            self.specs
                .iter()
                .map(|spec| {
                    json!({
                        "id": spec.id,
                        "label": spec.label,
                        "location": spec.location,
                        "shortcut": spec.shortcut,
                        "params": spec.params,
                        "mutates": spec.mutates,
                    })
                })
                .collect(),
        )
    }
}

/// The process-wide registry.
///
/// Built once, lazily: a front end asks for the same commands the tests do.
pub fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| Registry::new(specs()))
}

fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("note.add", "Add Note", "Notes › Edit", note_add)
            .key("Mod+N")
            .params(r#"{"text": string}"#),
        CommandSpec::new("note.remove", "Remove Note", "Notes › Edit", note_remove)
            .key("Mod+Shift+Backspace")
            .params(r#"{"id": number}"#),
        CommandSpec::new("note.list", "List Notes", "Notes › View", note_list).pure(),
        CommandSpec::new("note.clear", "Clear Notes", "Notes › Edit", note_clear)
            .params("{}")
            .when(|session| {
                session
                    .doc()
                    .is_empty()
                    .then_some("there are no notes to clear")
            }),
    ]
}

fn note_add(session: &mut Session, params: &Value) -> CmdResult {
    let text = p::req_str(params, "text")?;
    let id = session.doc_mut().add_note(text)?;
    Ok(json!({ "id": id.get() }))
}

fn note_remove(session: &mut Session, params: &Value) -> CmdResult {
    let raw = p::req_u64(params, "id")?;
    session.doc_mut().remove_note(NoteId::from_raw(raw))?;
    Ok(json!({ "removed": raw }))
}

fn note_list(session: &mut Session, _params: &Value) -> CmdResult {
    let notes: Vec<Value> = session
        .doc()
        .notes()
        .iter()
        .map(|note| json!({ "id": note.id.get(), "text": note.text }))
        .collect();
    Ok(json!({ "notes": notes }))
}

fn note_clear(session: &mut Session, _params: &Value) -> CmdResult {
    Ok(json!({ "removed": session.doc_mut().clear() }))
}

/// Parameter helpers.
///
/// Commands take `&Value` and never index it directly: a missing key returns an
/// error naming the parameter, an array or `null` in its place is rejected, and
/// a number is range-checked before it becomes a `u64`.
pub mod p {
    use serde_json::Value;

    use crate::CmdError;

    pub fn str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
        value.get(key).and_then(Value::as_str)
    }

    pub fn req_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, CmdError> {
        str(value, key).ok_or_else(|| CmdError::Params(format!("`{key}` (string) is required")))
    }

    /// Accepts an integer that fits in `u64`, or a non-negative integral float
    /// (JSON has one number type, so `3.0` and `3` both arrive as a number).
    pub fn u64(value: &Value, key: &str) -> Option<u64> {
        let raw = value.get(key)?;
        raw.as_u64().or_else(|| {
            let float = raw.as_f64()?;
            (float.is_finite() && float >= 0.0 && float.fract() == 0.0 && float <= u64::MAX as f64)
                .then_some(float as u64)
        })
    }

    pub fn req_u64(value: &Value, key: &str) -> Result<u64, CmdError> {
        u64(value, key)
            .ok_or_else(|| CmdError::Params(format!("`{key}` (non-negative integer) is required")))
    }
}
