//! The engine: a [`Session`], its command registry, bounded undo/redo,
//! atomic persistence and automation root confinement.
//!
//! Layering: this crate is layer 2 and depends on `grammarcraft-core` (L0) and
//! `grammarcraft-rules` (L1). Front ends depend on `grammarcraft-engine` only
//! and route every user-visible action through [`Session::run`].
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod ai;
mod cmd;
mod error;
pub mod file_access;
mod io;

pub use ai::{AiBackend, CraftProviderBackend, LocalOfflineBackend};
pub use cmd::{CommandSpec, Registry, registry};
pub use error::{CmdError, CmdResult};
pub use grammarcraft_rules::AnalysisReport;

use std::collections::VecDeque;

use grammarcraft_core::{
    AiProposal, CitationValidationIssue, Document, IgnoredIssueSet, MAX_UNDO_STEPS,
    PersonalDictionary, SimilarityReport, SnippetCatalog, StyleGuide, WritingGoals,
};
use grammarcraft_rules::AnalysisEngine;
use serde_json::Value;

use crate::ai::AiSessionState;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistorySnapshot {
    doc: Document,
    goals: WritingGoals,
    language_tag: String,
    personal_dict: PersonalDictionary,
    ignored: IgnoredIssueSet,
    style_guide: StyleGuide,
    snippets: SnippetCatalog,
}

/// One editing session: the document, goals, dictionary, style guide, snippets,
/// bounded history and analysis cache.
#[derive(Debug)]
pub struct Session {
    doc: Document,
    goals: WritingGoals,
    language_tag: String,
    personal_dict: PersonalDictionary,
    ignored: IgnoredIssueSet,
    style_guide: StyleGuide,
    snippets: SnippetCatalog,
    engine: AnalysisEngine,
    undo_stack: VecDeque<HistorySnapshot>,
    redo_stack: VecDeque<HistorySnapshot>,
    last_report: Option<AnalysisReport>,
    pub(crate) ai: AiSessionState,
    last_similarity: Option<SimilarityReport>,
    last_citations: Vec<String>,
    last_citation_issues: Vec<CitationValidationIssue>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            doc: Document::new(),
            goals: WritingGoals::default(),
            language_tag: "en-US".to_owned(),
            personal_dict: PersonalDictionary::new(),
            ignored: IgnoredIssueSet::new(),
            style_guide: StyleGuide::new(),
            snippets: SnippetCatalog::new(),
            engine: AnalysisEngine::new(),
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            last_report: None,
            ai: AiSessionState::default(),
            last_similarity: None,
            last_citations: Vec::new(),
            last_citation_issues: Vec::new(),
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn goals(&self) -> &WritingGoals {
        &self.goals
    }

    pub fn language_tag(&self) -> &str {
        &self.language_tag
    }

    pub fn personal_dict(&self) -> &PersonalDictionary {
        &self.personal_dict
    }

    pub fn ignored(&self) -> &IgnoredIssueSet {
        &self.ignored
    }

    pub fn style_guide(&self) -> &StyleGuide {
        &self.style_guide
    }

    pub fn snippets(&self) -> &SnippetCatalog {
        &self.snippets
    }

    pub fn last_report(&self) -> Option<&AnalysisReport> {
        self.last_report.as_ref()
    }

    pub fn pending_ai_proposal(&self) -> Option<&AiProposal> {
        self.ai.pending_proposal.as_ref()
    }

    pub fn has_pending_ai_proposal(&self) -> bool {
        self.ai.pending_proposal.is_some()
    }

    pub fn ai_locality_mode(&self) -> grammarcraft_core::AiLocalityMode {
        self.ai.mode
    }

    pub fn ai_consent_status(&self) -> Result<grammarcraft_core::AiConsentStatus, CmdError> {
        self.ai.consent_status(self.doc.len_bytes())
    }

    pub fn set_cloud_provider(
        &mut self,
        model: &str,
        provider: std::sync::Arc<dyn craft_ai_core::Provider>,
    ) -> Result<(), CmdError> {
        self.ai.set_cloud_provider(model, provider)
    }

    pub fn last_similarity(&self) -> Option<&SimilarityReport> {
        self.last_similarity.as_ref()
    }

    pub fn last_citations(&self) -> &[String] {
        &self.last_citations
    }

    pub fn last_citation_issues(&self) -> &[CitationValidationIssue] {
        &self.last_citation_issues
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }

    /// Runs a command by id.
    ///
    /// Unknown ids, disabled commands and bad parameters are errors, never
    /// panics: the id and the parameters can both come from a file, a socket
    /// or an agent.
    pub fn run(&mut self, id: &str, params: &Value) -> CmdResult {
        let spec = registry()
            .get(id)
            .ok_or_else(|| CmdError::UnknownCommand(id.to_owned()))?;
        if let Some(reason) = (spec.enabled)(self) {
            return Err(CmdError::Disabled(reason.to_owned()));
        }
        (spec.run)(self, params)
    }

    fn snapshot(&self) -> HistorySnapshot {
        HistorySnapshot {
            doc: self.doc.clone(),
            goals: self.goals,
            language_tag: self.language_tag.clone(),
            personal_dict: self.personal_dict.clone(),
            ignored: self.ignored.clone(),
            style_guide: self.style_guide.clone(),
            snippets: self.snippets.clone(),
        }
    }

    fn restore(&mut self, snap: HistorySnapshot) {
        self.doc = snap.doc;
        self.goals = snap.goals;
        self.language_tag = snap.language_tag;
        self.personal_dict = snap.personal_dict;
        self.ignored = snap.ignored;
        self.style_guide = snap.style_guide;
        self.snippets = snap.snippets;
        self.last_report = None;
    }

    fn push_undo(&mut self) {
        if self.undo_stack.len() >= MAX_UNDO_STEPS {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(self.snapshot());
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> Result<(), CmdError> {
        let prev = self
            .undo_stack
            .pop_back()
            .ok_or_else(|| CmdError::Disabled("nothing to undo".to_owned()))?;
        if self.redo_stack.len() >= MAX_UNDO_STEPS {
            self.redo_stack.pop_front();
        }
        self.redo_stack.push_back(self.snapshot());
        self.restore(prev);
        Ok(())
    }

    fn redo(&mut self) -> Result<(), CmdError> {
        let next = self
            .redo_stack
            .pop_back()
            .ok_or_else(|| CmdError::Disabled("nothing to redo".to_owned()))?;
        if self.undo_stack.len() >= MAX_UNDO_STEPS {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(self.snapshot());
        self.restore(next);
        Ok(())
    }

    fn invalidate_report(&mut self) {
        self.last_report = None;
    }

    fn analyze(
        &mut self,
        language_override: Option<&str>,
    ) -> Result<(&AnalysisReport, &str), CmdError> {
        if let Some(tag) = language_override {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                return Err(CmdError::Params("`language` must not be empty".to_owned()));
            }
            if trimmed != self.language_tag {
                self.language_tag = trimmed.to_owned();
                self.last_report = None;
            }
        }
        if self.last_report.is_none() {
            let report = self.engine.analyze(
                &self.doc,
                Some(&self.language_tag),
                self.goals,
                &self.personal_dict,
                &self.ignored,
                &self.style_guide,
            )?;
            self.last_report = Some(report);
        }
        let report = self
            .last_report
            .as_ref()
            .ok_or_else(|| CmdError::Failed("analysis report missing".to_owned()))?;
        Ok((report, self.doc.text()))
    }
}

#[cfg(test)]
mod tests;
