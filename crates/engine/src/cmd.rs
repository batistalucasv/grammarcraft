//! The command registry.

use std::collections::HashMap;
use std::sync::OnceLock;

use grammarcraft_core::{
    Audience, Context, CoreError, Formality, IgnoredIssueKey, Issue, IssueId, Purpose, TextStats,
    WritingGoals,
};
use grammarcraft_rules::AnalysisReport;
use serde_json::{Value, json};

use crate::{CmdError, CmdResult, Session, io};

/// A registered command.
#[derive(Clone)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub location: &'static str,
    pub shortcut: &'static str,
    pub params: &'static str,
    pub mutates: bool,
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
pub struct Registry {
    specs: Vec<CommandSpec>,
    by_id: HashMap<&'static str, usize>,
}

impl Registry {
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
        CommandSpec::new("doc.save", "Save Document", "Document › File", doc_save)
            .key("Mod+S")
            .params(r#"{"path": string}"#)
            .pure(),
        CommandSpec::new("doc.open", "Open Document", "Document › File", doc_open)
            .key("Mod+O")
            .params(r#"{"path": string}"#),
        CommandSpec::new("history.undo", "Undo", "Document › Edit", history_undo)
            .key("Mod+Z")
            .params("{}")
            .when(|session| (!session.can_undo()).then_some("nothing to undo")),
        CommandSpec::new("history.redo", "Redo", "Document › Edit", history_redo)
            .key("Mod+Shift+Z")
            .params("{}")
            .when(|session| (!session.can_redo()).then_some("nothing to redo")),
        CommandSpec::new("goals.set", "Set Writing Goals", "Goals › Configure", goals_set)
            .params(r#"{"audience"?: string, "formality"?: string, "context"?: string, "purpose"?: string, "language"?: string}"#),
        CommandSpec::new("stats.measure", "Measure Text Stats", "Review › Metrics", stats_measure)
            .params("{}")
            .pure(),
        CommandSpec::new("review.check", "Check Document", "Review › Proofing", review_check)
            .key("F7")
            .params(r#"{"language"?: string}"#)
            .pure(),
        CommandSpec::new("review.apply", "Apply Suggestion", "Review › Proofing", review_apply)
            .params(r#"{"issue_id": number, "replacement_index"?: number, "text"?: string}"#),
        CommandSpec::new("review.dismiss", "Dismiss Issue", "Review › Proofing", review_dismiss)
            .params(r#"{"issue_id": number}"#),
        CommandSpec::new("dict.add", "Add to Dictionary", "Review › Dictionary", dict_add)
            .params(r#"{"word": string}"#),
        CommandSpec::new("dict.remove", "Remove from Dictionary", "Review › Dictionary", dict_remove)
            .params(r#"{"word": string}"#),
    ]
}

fn doc_set_text(session: &mut Session, params: &Value) -> CmdResult {
    let text = p::req_str(params, "text")?;
    let mut candidate = session.doc.clone();
    let stats = candidate.set_text(text)?;
    session.push_undo();
    session.doc = candidate;
    session.invalidate_report();
    Ok(stats_json(stats))
}

fn doc_get(session: &mut Session, _params: &Value) -> CmdResult {
    Ok(json!({
        "text": session.doc().text(),
        "language": session.language_tag(),
        "goals": goals_json(session.goals()),
        "stats": stats_json(session.doc().stats()),
        "undo_depth": session.undo_len(),
        "redo_depth": session.redo_len(),
    }))
}

fn doc_clear(session: &mut Session, _params: &Value) -> CmdResult {
    session.push_undo();
    let cleared = session.doc.clear();
    session.invalidate_report();
    Ok(json!({ "cleared_bytes": cleared }))
}

fn doc_save(session: &mut Session, params: &Value) -> CmdResult {
    let path = p::req_str(params, "path")?;
    let format = io::save_document(
        path,
        &session.doc,
        &session.language_tag,
        session.goals,
        &session.personal_dict,
    )?;
    Ok(json!({ "path": path, "format": format, "bytes": session.doc.len_bytes() }))
}

fn doc_open(session: &mut Session, params: &Value) -> CmdResult {
    let path = p::req_str(params, "path")?;
    let loaded = io::open_document(path)?;
    session.push_undo();
    session.doc = loaded.doc;
    session.language_tag = loaded.language;
    session.goals = loaded.goals;
    session.personal_dict = loaded.personal_dict;
    session.ignored.clear();
    session.invalidate_report();
    Ok(json!({
        "path": path,
        "format": loaded.format,
        "language": session.language_tag(),
        "stats": stats_json(session.doc.stats()),
    }))
}

fn history_undo(session: &mut Session, _params: &Value) -> CmdResult {
    session.undo()?;
    Ok(json!({
        "text": session.doc.text(),
        "stats": stats_json(session.doc.stats()),
        "undo_depth": session.undo_len(),
        "redo_depth": session.redo_len(),
    }))
}

fn history_redo(session: &mut Session, _params: &Value) -> CmdResult {
    session.redo()?;
    Ok(json!({
        "text": session.doc.text(),
        "stats": stats_json(session.doc.stats()),
        "undo_depth": session.undo_len(),
        "redo_depth": session.redo_len(),
    }))
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
    let next_lang = match p::opt_str(params, "language")? {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.len() > 32 {
                return Err(CmdError::Params(
                    "`language` must be 1..=32 bytes".to_owned(),
                ));
            }
            Some(trimmed.to_owned())
        }
        None => None,
    };
    session.push_undo();
    session.goals = next;
    if let Some(lang) = next_lang {
        session.language_tag = lang;
    }
    session.invalidate_report();
    Ok(goals_json_with_lang(
        session.goals(),
        session.language_tag(),
    ))
}

fn stats_measure(session: &mut Session, _params: &Value) -> CmdResult {
    Ok(stats_json(session.doc().stats()))
}

fn review_check(session: &mut Session, params: &Value) -> CmdResult {
    p::require_object_or_empty(params)?;
    let lang_override = p::opt_str(params, "language")?;
    let (report, doc_text) = session.analyze(lang_override)?;
    Ok(report_json(report, doc_text))
}

fn review_apply(session: &mut Session, params: &Value) -> CmdResult {
    let raw_id = p::req_u64(params, "issue_id")?;
    let target_id = IssueId::from_raw(raw_id);
    let explicit_text = p::opt_str(params, "text")?.map(ToOwned::to_owned);
    let rep_index = p::opt_u64(params, "replacement_index")?;

    let issue = {
        let (report, _) = session.analyze(None)?;
        report
            .issues
            .iter()
            .find(|issue| issue.id == target_id)
            .cloned()
            .ok_or(CoreError::UnknownIssue(target_id))?
    };

    let replacement = resolve_replacement(&issue, explicit_text, rep_index)?;
    let mut candidate = session.doc.clone();
    let stats = candidate.replace_span(issue.span, &replacement)?;
    session.push_undo();
    session.doc = candidate;
    session.invalidate_report();
    Ok(json!({
        "applied_issue_id": raw_id,
        "replacement": replacement,
        "stats": stats_json(stats),
    }))
}

fn resolve_replacement(
    issue: &Issue,
    explicit_text: Option<String>,
    rep_index: Option<u64>,
) -> Result<String, CmdError> {
    if let Some(text) = explicit_text {
        return Ok(text);
    }
    let idx = rep_index.unwrap_or(0);
    let idx_usize = usize::try_from(idx)
        .map_err(|_| CmdError::Params("`replacement_index` out of range".to_owned()))?;
    issue
        .replacements
        .get(idx_usize)
        .map(|r| r.text.clone())
        .ok_or_else(|| {
            CmdError::Params(format!(
                "issue {} has no replacement at index {idx}",
                issue.id
            ))
        })
}

fn review_dismiss(session: &mut Session, params: &Value) -> CmdResult {
    let raw_id = p::req_u64(params, "issue_id")?;
    let target_id = IssueId::from_raw(raw_id);
    let key = {
        let (report, text) = session.analyze(None)?;
        let issue = report
            .issues
            .iter()
            .find(|issue| issue.id == target_id)
            .ok_or(CoreError::UnknownIssue(target_id))?;
        let snippet = issue.span.slice(text)?;
        IgnoredIssueKey::new(issue.rule_id.clone(), issue.span.start(), snippet)?
    };
    session.push_undo();
    let added = session.ignored.ignore(key)?;
    session.invalidate_report();
    Ok(json!({
        "dismissed_issue_id": raw_id,
        "added": added,
        "ignored_count": session.ignored.len(),
    }))
}

fn dict_add(session: &mut Session, params: &Value) -> CmdResult {
    let word = p::req_str(params, "word")?;
    let mut candidate = session.personal_dict.clone();
    let added = candidate.add(word)?;
    session.push_undo();
    session.personal_dict = candidate;
    session.invalidate_report();
    Ok(json!({
        "word": word.trim(),
        "added": added,
        "count": session.personal_dict.len(),
    }))
}

fn dict_remove(session: &mut Session, params: &Value) -> CmdResult {
    let word = p::req_str(params, "word")?;
    let mut candidate = session.personal_dict.clone();
    let removed = candidate.remove(word)?;
    if removed {
        session.push_undo();
        session.personal_dict = candidate;
        session.invalidate_report();
    }
    Ok(json!({
        "word": word.trim(),
        "removed": removed,
        "count": session.personal_dict.len(),
    }))
}

fn report_json(report: &AnalysisReport, doc_text: &str) -> Value {
    let issues: Vec<Value> = report
        .issues
        .iter()
        .map(|issue| {
            let excerpt = issue.span.slice(doc_text).unwrap_or("");
            let replacements: Vec<Value> = issue
                .replacements
                .iter()
                .map(|r| json!({ "text": r.text, "label": r.label }))
                .collect();
            json!({
                "id": issue.id.get(),
                "rule_id": issue.rule_id,
                "category": issue.category.as_str(),
                "severity": issue.severity.as_str(),
                "confidence": issue.confidence.as_str(),
                "start": issue.span.start(),
                "end": issue.span.end(),
                "excerpt": excerpt,
                "title": issue.title,
                "explanation": issue.explanation,
                "replacements": replacements,
            })
        })
        .collect();
    json!({
        "language": report.language_tag,
        "stats": stats_json(report.stats),
        "issues": issues,
        "readability": report.readability,
        "score": report.score,
        "tone": report.tone,
    })
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

fn goals_json_with_lang(goals: &WritingGoals, language: &str) -> Value {
    json!({
        "audience": goals.audience.as_str(),
        "formality": goals.formality.as_str(),
        "context": goals.context.as_str(),
        "purpose": goals.purpose.as_str(),
        "language": language,
    })
}

/// Parameter helpers.
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

    pub fn require_object_or_empty(value: &Value) -> Result<(), CmdError> {
        require_object(value)
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
        if raw.is_null() {
            return Ok(None);
        }
        raw.as_str()
            .map(Some)
            .ok_or_else(|| CmdError::Params(format!("`{key}` must be a string")))
    }

    pub fn u64(value: &Value, key: &str) -> Option<u64> {
        let raw = value.get(key)?;
        raw.as_u64().or_else(|| {
            let float = raw.as_f64()?;
            (float.is_finite() && float >= 0.0 && float.fract() == 0.0 && float <= u64::MAX as f64)
                .then_some(float as u64)
        })
    }

    pub fn opt_u64(value: &Value, key: &str) -> Result<Option<u64>, CmdError> {
        let Some(raw) = value.get(key) else {
            return Ok(None);
        };
        if raw.is_null() {
            return Ok(None);
        }
        u64(value, key)
            .map(Some)
            .ok_or_else(|| CmdError::Params(format!("`{key}` must be a non-negative integer")))
    }

    pub fn req_u64(value: &Value, key: &str) -> Result<u64, CmdError> {
        u64(value, key)
            .ok_or_else(|| CmdError::Params(format!("`{key}` (non-negative integer) is required")))
    }
}
