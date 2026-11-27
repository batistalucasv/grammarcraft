use std::collections::BTreeMap;
use std::sync::Arc;

use craft::dialogs::{Answer, Dialogs, FileDialog};
use craft::host::{Host, Wake};
use craft::ui::contract::{CommandSet as _, Props as _, Show as _};
use craft::ui::runtime::commands::NavCommand;
use craft::ui::runtime::env::Label;
use craft::ui::runtime::frame::{FrameInput, FrameOutput};
use craft::ui::shell::app::ShellApp;
use craft::ui::shell::embed::{self, CloseDecision};
use grammarcraft_core::{
    AiMode, Audience, CitationStyle, Context, Formality, Issue, IssueCategory, Purpose,
    ReadabilityMetrics, ToneProfile, WritingGoals, WritingScore,
};
use grammarcraft_engine::{AnalysisReport, CmdError, Session};
use serde_json::{Value, json};

use crate::editor;
use crate::sidebar;
use crate::theme_bridge::{AppColors, ThemeMode, build_host, frame_input, sync_egui_visuals};
use crate::toolbar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarTab {
    #[default]
    Alerts,
    GoalsAndScore,
    Tone,
    StyleAndSnippets,
    AiAssist,
    CitationsAndSimilarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaneFocusTarget {
    Toolbar,
    #[default]
    Editor,
    Sidebar,
}

impl PaneFocusTarget {
    pub fn next(self) -> Self {
        match self {
            Self::Toolbar => Self::Editor,
            Self::Editor => Self::Sidebar,
            Self::Sidebar => Self::Toolbar,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Toolbar => Self::Sidebar,
            Self::Editor => Self::Toolbar,
            Self::Sidebar => Self::Editor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    Open,
    Save,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StyleRuleDraftKind {
    #[default]
    PreferredTerm,
    BrandCasing,
}

#[derive(Debug, Clone, Default)]
pub struct StyleDraft {
    pub kind: StyleRuleDraftKind,
    pub id: String,
    pub term: String,
    pub preferred: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Default)]
pub struct SnippetDraft {
    pub prefix: String,
    pub title: String,
    pub body: String,
    pub expand_vars: String,
}

pub struct GrammarCraftApp {
    pub session: Session,
    pub editor_text: String,
    pub theme_mode: ThemeMode,
    pub colors: AppColors,
    pub active_tab: SidebarTab,
    pub category_filter: Option<IssueCategory>,
    pub selected_issue_id: Option<u64>,
    pub highlight_span: Option<(usize, usize)>,
    pub focused_pane: PaneFocusTarget,
    pub focus_request: Option<PaneFocusTarget>,
    pub path_bar_mode: Option<FileAction>,
    pub path_input: String,
    pub current_path: Option<String>,
    pub style_draft: StyleDraft,
    pub snippet_draft: SnippetDraft,
    pub ai_instruction: String,
    pub ai_tone: String,
    pub citation_style: CitationStyle,
    pub bibtex_input: String,
    pub similarity_dir: String,
    pub status_notice: Option<(String, bool)>,
    pub host: Arc<Host>,
    pub(crate) dialogs: Option<Dialogs<FileAction>>,
}

impl Default for GrammarCraftApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GrammarCraftApp {
    pub fn new() -> Self {
        let mode = ThemeMode::Light;
        let mut app = Self {
            session: Session::new(),
            editor_text: String::new(),
            theme_mode: mode,
            colors: AppColors::resolve(mode),
            active_tab: SidebarTab::Alerts,
            category_filter: None,
            selected_issue_id: None,
            highlight_span: None,
            focused_pane: PaneFocusTarget::Editor,
            focus_request: None,
            path_bar_mode: None,
            path_input: String::new(),
            current_path: None,
            style_draft: StyleDraft::default(),
            snippet_draft: SnippetDraft::default(),
            ai_instruction: String::new(),
            ai_tone: "formal".to_owned(),
            citation_style: CitationStyle::Apa,
            bibtex_input: "@article{sample2026,\n  author = {Ada Lovelace},\n  title = {Clean-Room Writing Systems},\n  journal = {Journal of Craft},\n  year = {2026}\n}".to_owned(),
            similarity_dir: String::new(),
            status_notice: None,
            host: build_host(),
            dialogs: None,
        };
        app.refresh_check();
        app
    }

    pub fn with_initial_text(text: &str) -> Self {
        let mut app = Self::new();
        app.editor_text = text.to_owned();
        app.set_text_and_check();
        app
    }

    pub fn install_dialogs(&mut self, ctx: &egui::Context) {
        if self.dialogs.is_none() {
            let wake_ctx = ctx.clone();
            let wake = Wake::new(move || wake_ctx.request_repaint());
            self.dialogs = Some(Dialogs::new(wake));
        }
    }

    pub fn report(&self) -> Option<&AnalysisReport> {
        self.session.last_report()
    }

    pub fn issues(&self) -> &[Issue] {
        self.report().map(|r| r.issues.as_slice()).unwrap_or(&[])
    }

    pub fn writing_score(&self) -> WritingScore {
        self.report().map(|r| r.score).unwrap_or_default()
    }

    pub fn readability(&self) -> ReadabilityMetrics {
        self.report().map(|r| r.readability).unwrap_or_default()
    }

    pub fn tone_profile(&self) -> ToneProfile {
        self.report().map(|r| r.tone.clone()).unwrap_or_default()
    }

    pub fn dispatch(&mut self, id: &str, params: Value) -> Result<Value, CmdError> {
        self.session.run(id, &params)
    }

    pub fn run_and_report(&mut self, id: &str, params: Value) -> bool {
        match self.dispatch(id, params) {
            Ok(_) => {
                self.status_notice = None;
                true
            }
            Err(err) => {
                self.status_notice = Some((err.to_string(), true));
                false
            }
        }
    }

    pub fn refresh_check(&mut self) {
        if let Err(err) = self.dispatch("review.check", json!({})) {
            self.status_notice = Some((err.to_string(), true));
        } else {
            self.reconcile_selection();
        }
    }

    pub fn set_text_and_check(&mut self) {
        if self.session.doc().text() != self.editor_text
            && !self.run_and_report("doc.set_text", json!({ "text": self.editor_text }))
        {
            return;
        }
        self.refresh_check();
    }

    pub fn sync_editor_from_session(&mut self) {
        self.editor_text = self.session.doc().text().to_owned();
        self.refresh_check();
    }

    pub fn apply_suggestion(&mut self, issue_id: u64, replacement_index: usize) {
        let params = json!({
            "issue_id": issue_id,
            "replacement_index": replacement_index,
        });
        if self.run_and_report("review.apply", params) {
            self.selected_issue_id = None;
            self.highlight_span = None;
            self.sync_editor_from_session();
            self.status_notice = Some(("Applied suggestion".to_owned(), false));
        }
    }

    pub fn dismiss_once(&mut self, issue_id: u64) {
        let params = json!({ "issue_id": issue_id });
        if self.run_and_report("review.dismiss", params) {
            if self.selected_issue_id == Some(issue_id) {
                self.selected_issue_id = None;
                self.highlight_span = None;
            }
            self.refresh_check();
            self.status_notice = Some(("Dismissed issue".to_owned(), false));
        }
    }

    pub fn dismiss_document(&mut self, issue_id: u64) {
        let Some((rule_id, excerpt)) = self.issue_signature(issue_id) else {
            return;
        };
        let mut dismissed = 0usize;
        let max_passes = self.issues().len().max(1);
        for _ in 0..max_passes {
            let next_id = self.find_matching_issue(&rule_id, &excerpt);
            let Some(target) = next_id else {
                break;
            };
            if !self.run_and_report("review.dismiss", json!({ "issue_id": target })) {
                break;
            }
            dismissed += 1;
            self.refresh_check();
        }
        if dismissed > 0 {
            self.selected_issue_id = None;
            self.highlight_span = None;
            self.status_notice = Some((
                format!("Dismissed {dismissed} matching issue(s) in document"),
                false,
            ));
        }
    }

    fn issue_signature(&self, issue_id: u64) -> Option<(String, String)> {
        let doc_text = self.session.doc().text();
        let issue = self.issues().iter().find(|i| i.id.get() == issue_id)?;
        let excerpt = issue.span.slice(doc_text).unwrap_or("").to_owned();
        Some((issue.rule_id.clone(), excerpt))
    }

    fn find_matching_issue(&self, rule_id: &str, excerpt: &str) -> Option<u64> {
        let doc_text = self.session.doc().text();
        self.issues()
            .iter()
            .find(|issue| {
                issue.rule_id == rule_id && issue.span.slice(doc_text).unwrap_or("") == excerpt
            })
            .map(|issue| issue.id.get())
    }

    pub fn add_word_to_dictionary(&mut self, word: &str) {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.run_and_report("dict.add", json!({ "word": trimmed })) {
            self.selected_issue_id = None;
            self.highlight_span = None;
            self.refresh_check();
            self.status_notice = Some((format!("Added `{trimmed}` to dictionary"), false));
        }
    }

    pub fn undo(&mut self) {
        if self.run_and_report("history.undo", json!({})) {
            self.selected_issue_id = None;
            self.highlight_span = None;
            self.sync_editor_from_session();
        }
    }

    pub fn redo(&mut self) {
        if self.run_and_report("history.redo", json!({})) {
            self.selected_issue_id = None;
            self.highlight_span = None;
            self.sync_editor_from_session();
        }
    }

    pub fn set_language(&mut self, tag: &str) {
        if self.run_and_report("goals.set", json!({ "language": tag })) {
            self.refresh_check();
        }
    }

    pub fn set_goals(&mut self, goals: WritingGoals) {
        let params = json!({
            "audience": goals.audience.as_str(),
            "formality": goals.formality.as_str(),
            "context": goals.context.as_str(),
            "purpose": goals.purpose.as_str(),
            "inclusive_language": goals.inclusive_language,
        });
        if self.run_and_report("goals.set", params) {
            self.refresh_check();
        }
    }

    pub fn set_ai_consent(&mut self, locality: &str, granted: bool) {
        let params = json!({
            "locality": locality,
            "granted": granted,
        });
        if self.run_and_report("ai.consent", params) {
            let msg = if locality == "local" {
                "AI mode set to Local (Offline)".to_owned()
            } else if granted {
                "Cloud AI consent granted".to_owned()
            } else {
                "Cloud AI mode active (consent required before send)".to_owned()
            };
            self.status_notice = Some((msg, false));
        }
    }

    pub fn run_ai_action(&mut self, mode: AiMode) {
        let (id, params) = match mode {
            AiMode::Rewrite => (
                "ai.rewrite",
                json!({ "instruction": self.ai_instruction.trim() }),
            ),
            AiMode::AdjustTone => ("ai.adjustTone", json!({ "tone": self.ai_tone.trim() })),
            AiMode::Summarize => ("ai.summarize", json!({ "max_sentences": 2 })),
            AiMode::Expand => (
                "ai.expand",
                json!({ "instruction": self.ai_instruction.trim() }),
            ),
            AiMode::Draft => ("ai.draft", json!({ "prompt": self.ai_instruction.trim() })),
        };
        if self.run_and_report(id, params) {
            self.status_notice = Some((format!("Staged {} diff preview", mode.label()), false));
        }
    }

    pub fn apply_ai_proposal(&mut self) {
        if self.run_and_report("ai.applyResult", json!({})) {
            self.selected_issue_id = None;
            self.highlight_span = None;
            self.sync_editor_from_session();
            self.status_notice = Some(("Applied AI proposal".to_owned(), false));
        }
    }

    pub fn discard_ai_proposal(&mut self) {
        if self.run_and_report("ai.discardResult", json!({})) {
            self.status_notice = Some(("Discarded AI proposal".to_owned(), false));
        }
    }

    pub fn validate_citations(&mut self) {
        let bib = self.bibtex_input.clone();
        if self.run_and_report("citations.validate", json!({ "bibtex": bib })) {
            let count = self.session.last_citation_issues().len();
            self.status_notice = Some((format!("Validated BibTeX ({count} warning(s))"), false));
        }
    }

    pub fn format_citations(&mut self) {
        let bib = self.bibtex_input.clone();
        let style = self.citation_style.as_str();
        if self.run_and_report("citations.format", json!({ "bibtex": bib, "style": style })) {
            self.status_notice = Some((
                format!(
                    "Formatted bibliography in {}",
                    self.citation_style.display_name()
                ),
                false,
            ));
        }
    }

    pub fn check_similarity(&mut self) {
        let dir = self.similarity_dir.trim().to_owned();
        if self.run_and_report("similarity.check", json!({ "directory": dir }))
            && let Some(report) = self.session.last_similarity()
        {
            self.status_notice = Some((
                format!(
                    "Similarity: {}% across {} source(s)",
                    report.overall_percent, report.sources_scanned
                ),
                false,
            ));
        }
    }

    pub fn add_draft_style_rule(&mut self) {
        let draft = &self.style_draft;
        let params = match draft.kind {
            StyleRuleDraftKind::PreferredTerm => json!({
                "id": draft.id.trim(),
                "kind": "preferred_term",
                "discouraged": draft.term.trim(),
                "preferred": draft.preferred.trim(),
                "rationale": non_empty_or(&draft.rationale, "Editorial preference"),
            }),
            StyleRuleDraftKind::BrandCasing => json!({
                "id": draft.id.trim(),
                "kind": "brand_casing",
                "exact": draft.term.trim(),
                "rationale": non_empty_or(&draft.rationale, "Brand casing consistency"),
            }),
        };
        if self.run_and_report("style.add_rule", params) {
            self.style_draft.id.clear();
            self.style_draft.term.clear();
            self.style_draft.preferred.clear();
            self.style_draft.rationale.clear();
            self.refresh_check();
            self.status_notice = Some(("Added style rule".to_owned(), false));
        }
    }

    pub fn remove_style_rule(&mut self, id: &str) {
        if self.run_and_report("style.remove_rule", json!({ "id": id })) {
            self.refresh_check();
        }
    }

    pub fn add_draft_snippet(&mut self) {
        let params = json!({
            "prefix": self.snippet_draft.prefix.trim(),
            "title": self.snippet_draft.title.trim(),
            "body": self.snippet_draft.body,
        });
        if self.run_and_report("snippet.add", params) {
            self.snippet_draft.prefix.clear();
            self.snippet_draft.title.clear();
            self.snippet_draft.body.clear();
            self.status_notice = Some(("Added snippet".to_owned(), false));
        }
    }

    pub fn expand_and_insert_snippet(&mut self, prefix: &str) {
        let vars = parse_snippet_vars(&self.snippet_draft.expand_vars);
        let values_json = Value::Object(
            vars.into_iter()
                .map(|(k, v)| (k, Value::String(v)))
                .collect(),
        );
        match self.dispatch(
            "snippet.expand",
            json!({ "prefix": prefix, "values": values_json }),
        ) {
            Ok(result) => {
                if let Some(expanded) = result.get("text").and_then(Value::as_str) {
                    if !self.editor_text.is_empty() && !self.editor_text.ends_with([' ', '\n']) {
                        self.editor_text.push(' ');
                    }
                    self.editor_text.push_str(expanded);
                    self.set_text_and_check();
                    self.status_notice = Some((format!("Inserted `{prefix}`"), false));
                }
            }
            Err(err) => {
                self.status_notice = Some((err.to_string(), true));
            }
        }
    }

    pub fn remove_snippet(&mut self, prefix: &str) {
        let _ = self.run_and_report("snippet.remove", json!({ "prefix": prefix }));
    }

    pub fn open_path(&mut self, path: &str) {
        if self.run_and_report("doc.open", json!({ "path": path })) {
            self.current_path = Some(path.to_owned());
            self.path_bar_mode = None;
            self.selected_issue_id = None;
            self.highlight_span = None;
            self.sync_editor_from_session();
            self.status_notice = Some((format!("Opened {path}"), false));
        }
    }

    pub fn save_path(&mut self, path: &str) {
        if self.run_and_report("doc.save", json!({ "path": path })) {
            self.current_path = Some(path.to_owned());
            self.path_bar_mode = None;
            self.status_notice = Some((format!("Saved {path}"), false));
        }
    }

    pub fn request_native_file_dialog(&mut self, action: FileAction) {
        let Some(dialogs) = self.dialogs.as_mut() else {
            self.path_bar_mode = Some(action);
            return;
        };
        let dialog = match action {
            FileAction::Open => FileDialog::open()
                .title("Open Document")
                .filter("Documents", &["txt", "md", "docx", "json"]),
            FileAction::Save => FileDialog::save()
                .title("Save Document")
                .filter("Documents", &["txt", "md", "docx", "json"])
                .file_name("document.docx"),
        };
        dialogs.ask(dialog, action);
    }

    pub fn pump_dialogs(&mut self, frame: Option<&eframe::Frame>) {
        let Some(dialogs) = self.dialogs.as_mut() else {
            return;
        };
        dialogs.show_pending(frame);
        let answers = dialogs.take_answers();
        for (action, answer) in answers {
            self.handle_dialog_answer(action, answer);
        }
    }

    fn handle_dialog_answer(&mut self, action: FileAction, answer: Answer) {
        match (action, answer) {
            (FileAction::Open, Answer::Paths(paths)) => {
                if let Some(path) = paths.first() {
                    let s = path.to_string_lossy().into_owned();
                    self.open_path(&s);
                }
            }
            (FileAction::Save, Answer::Saved { path, .. }) => {
                let s = path.to_string_lossy().into_owned();
                self.save_path(&s);
            }
            (FileAction::Save, Answer::Paths(paths)) => {
                if let Some(path) = paths.first() {
                    let s = path.to_string_lossy().into_owned();
                    self.save_path(&s);
                }
            }
            (_, Answer::Unavailable(reason)) => {
                self.path_bar_mode = Some(action);
                self.status_notice = Some((reason, true));
            }
            (_, Answer::Cancelled)
            | (_, Answer::Message(_))
            | (FileAction::Open, Answer::Saved { .. }) => {}
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme_mode = self.theme_mode.toggle();
        self.colors = AppColors::resolve(self.theme_mode);
    }

    pub fn select_issue(&mut self, issue_id: u64, span: (usize, usize)) {
        self.selected_issue_id = Some(issue_id);
        self.highlight_span = Some(span);
        self.active_tab = SidebarTab::Alerts;
    }

    pub fn select_tone_span(&mut self, span: (usize, usize)) {
        self.selected_issue_id = None;
        self.highlight_span = Some(span);
    }

    fn reconcile_selection(&mut self) {
        if let Some(selected) = self.selected_issue_id
            && !self.issues().iter().any(|i| i.id.get() == selected)
        {
            self.selected_issue_id = None;
            self.highlight_span = None;
        }
    }

    pub fn handle_frame_output(&mut self, output: &FrameOutput) {
        for fired in &output.fired {
            if fired.command.id == NavCommand::NextPane.id() {
                let next = self.focused_pane.next();
                self.focused_pane = next;
                self.focus_request = Some(next);
            } else if fired.command.id == NavCommand::PreviousPane.id() {
                let prev = self.focused_pane.prev();
                self.focused_pane = prev;
                self.focus_request = Some(prev);
            }
        }
    }

    pub fn ui_frame_input(&self) -> FrameInput {
        frame_input(self.theme_mode, Arc::clone(&self.host))
    }

    pub fn show_root(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        sync_egui_visuals(&ctx, &self.colors);
        self.handle_escape_key(&ctx);

        egui::Panel::top("grammarcraft_toolbar").show(ui, |ui| {
            toolbar::show(self, ui);
        });

        let sidebar_width = (ui.available_width() * 0.38).clamp(300.0, 440.0);
        egui::Panel::right("grammarcraft_sidebar")
            .resizable(true)
            .default_size(sidebar_width)
            .show(ui, |ui| {
                sidebar::show(self, ui);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            editor::show(self, ui);
        });

        craft::ui::runtime::keys::with_window_intents(&ctx, false, |intents| {
            if intents.take(craft::ui::runtime::keys::Intent::Cancel) {
                self.path_bar_mode = None;
                self.selected_issue_id = None;
                self.highlight_span = None;
                return true;
            }
            false
        });
    }

    fn handle_escape_key(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.path_bar_mode.is_some() {
                self.path_bar_mode = None;
            } else if ctx.memory(|m| m.focused().is_some()) {
                ctx.memory_mut(|m| m.surrender_focus(egui::Id::NULL));
            }
        }
    }

    pub const fn audiences() -> [Audience; 3] {
        [
            Audience::General,
            Audience::Specialized,
            Audience::TechnicalExecutive,
        ]
    }

    pub const fn formalities() -> [Formality; 3] {
        [Formality::Informal, Formality::Neutral, Formality::Formal]
    }

    pub const fn contexts() -> [Context; 6] {
        [
            Context::General,
            Context::Academic,
            Context::Corporate,
            Context::Technical,
            Context::Creative,
            Context::Casual,
        ]
    }

    pub const fn purposes() -> [Purpose; 4] {
        [
            Purpose::Inform,
            Purpose::Explain,
            Purpose::Persuade,
            Purpose::Narrate,
        ]
    }
}

impl ShellApp for GrammarCraftApp {
    fn frame_input(&mut self) -> FrameInput {
        self.ui_frame_input()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.install_dialogs(&ctx);
        self.show_root(ui);
        self.pump_dialogs(None);
    }

    fn logic(&mut self, _ctx: &egui::Context) {}

    fn close_requested(&mut self) -> CloseDecision {
        CloseDecision::Close
    }

    fn output(&mut self, output: FrameOutput) {
        self.handle_frame_output(&output);
    }
}

impl eframe::App for GrammarCraftApp {
    fn raw_input_hook(&mut self, ctx: &egui::Context, _raw: &mut egui::RawInput) {
        embed::before_input(ctx, |output| {
            self.handle_frame_output(&output);
            self.ui_frame_input()
        });
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        embed::logic(ctx, || CloseDecision::Close);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        embed::before_ui(&ctx, frame);
        self.install_dialogs(&ctx);
        self.show_root(ui);
        self.pump_dialogs(Some(frame));
    }
}

pub fn craft_button(
    ui: &mut egui::Ui,
    label: impl Into<Label>,
    enabled: bool,
) -> craft::ui::buttons::ButtonOutput {
    craft::ui::buttons::Button::new(label)
        .enabled(enabled)
        .show(ui)
}

pub fn primary_button(
    ui: &mut egui::Ui,
    label: impl Into<Label>,
    enabled: bool,
) -> craft::ui::buttons::ButtonOutput {
    use craft::theme::Variant;
    craft::ui::buttons::Button::new(label)
        .variant(Variant::PRIMARY)
        .enabled(enabled)
        .show(ui)
}

fn non_empty_or<'a>(raw: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

fn parse_snippet_vars(raw: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for pair in raw.split(',') {
        let trimmed = pair.trim();
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim();
            if !key.is_empty() {
                map.insert(key.to_owned(), v.trim().to_owned());
            }
        }
    }
    map
}
