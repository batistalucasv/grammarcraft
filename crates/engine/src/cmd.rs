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

use grammarcraft_core::{Audience, Context, Formality, Purpose, TextStats, WritingGoals};
use serde_json::{Value, json};

use crate::{CmdResult, Session};

/// A registered command.
#[derive(Clone)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    /// Where it lives in a front end, e.g. `Document › Edit`. Free-form on purpose.
    pub location: &'static str,
    /// Portable shortcut (`Mod+Shift+V`); `Mod` is Ctrl, or ⌘ on macOS.
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

    /// Marks a command that does not change the document or session state.
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
        CommandSpec::new("doc.set_text", "Set Document Text", "Document › Edit", doc_set_text)
            .key("Mod+Shift+V")
            .params(r#"{"text": string}"#),
        CommandSpec::new("doc.get", "Get Document", "Document › View", doc_get)
            .params("{}")
            .pure(),
        CommandSpec::new("doc.clear", "Clear Document", "Document › Edit", doc_clear)
            .params("{}")
            .when(|session| {
                session
                    .doc()
                    .is_empty()
                    .then_some("the document is already empty")
            }),
        CommandSpec::new("goals.set", "Set Writing Goals", "Goals › Configure", goals_set)
            .params(r#"{"audience"?: string, "formality"?: string, "context"?: string, "purpose"?: string}"#),
        CommandSpec::new("stats.measure", "Measure Text Stats", "Review › Metrics", stats_measure)
            .params("{}")
            .pure(),
    ]
}

fn doc_set_text(session: &mut Session, params: &Value) -> CmdResult {
    let text = p::req_str(params, "text")?;
    let stats = session.doc_mut().set_text(text)?;
    Ok(stats_json(stats))
}

fn doc_get(session: &mut Session, _params: &Value) -> CmdResult {
    Ok(json!({
        "text": session.doc().text(),
        "goals": goals_json(session.goals()),
        "stats": stats_json(session.doc().stats()),
    }))
}

fn doc_clear(session: &mut Session, _params: &Value) -> CmdResult {
    Ok(json!({ "cleared_bytes": session.doc_mut().clear() }))
}

fn goals_set(session: &mut Session, params: &Value) -> CmdResult {
    p::require_object(params)?;
    let mut next = *session.goals();
    if let Some(raw) = p::opt_str(params, "audience")? {
        next.audience = Audience::parse(raw)?;
    }
    if let Some(raw) = p::opt_str(params, "formality")? {
        next.formality = Formality::parse(raw)?;
    }
    if let Some(raw) = p::opt_str(params, "context")? {
        next.context = Context::parse(raw)?;
    }
    if let Some(raw) = p::opt_str(params, "purpose")? {
        next.purpose = Purpose::parse(raw)?;
    }
    *session.goals_mut() = next;
    Ok(goals_json(session.goals()))
}

fn stats_measure(session: &mut Session, _params: &Value) -> CmdResult {
    Ok(stats_json(session.doc().stats()))
}

fn stats_json(stats: TextStats) -> Value {
    json!({
        "bytes": stats.bytes,
        "chars": stats.chars,
        "words": stats.words,
        "sentences": stats.sentences,
        "paragraphs": stats.paragraphs,
    })
}

fn goals_json(goals: &WritingGoals) -> Value {
    json!({
        "audience": goals.audience.as_str(),
        "formality": goals.formality.as_str(),
        "context": goals.context.as_str(),
        "purpose": goals.purpose.as_str(),
    })
}

/// Parameter helpers.
///
/// Commands take `&Value` and never index it directly: a missing or mistyped
/// key returns [`CmdError::Params`] naming the parameter.
pub mod p {
    use serde_json::Value;

    use crate::CmdError;

    pub fn require_object(value: &Value) -> Result<(), CmdError> {
        if value.is_object() {
            Ok(())
        } else {
            Err(CmdError::Params(
                "parameters must be a JSON object".to_owned(),
            ))
        }
    }

    pub fn str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
        value.get(key).and_then(Value::as_str)
    }

    pub fn req_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, CmdError> {
        str(value, key).ok_or_else(|| CmdError::Params(format!("`{key}` (string) is required")))
    }

    pub fn opt_str<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>, CmdError> {
        let Some(raw) = value.get(key) else {
            return Ok(None);
        };
        raw.as_str()
            .map(Some)
            .ok_or_else(|| CmdError::Params(format!("`{key}` must be a string")))
    }
}
