use craft::theme::Variant;
use craft::theme::variant::{Emphasis, Tone};
use craft::ui::contract::{Props as _, Show as _};
use craft::ui::display::Badge;
use grammarcraft_core::{
    Issue, IssueCategory, ReadabilityLevel, StyleRuleKind, ToneSignal, WritingGoals,
};

use crate::state::{
    GrammarCraftApp, PaneFocusTarget, SidebarTab, StyleRuleDraftKind, craft_button, primary_button,
};
use crate::theme_bridge::category_style;

const CATEGORY_FILTERS: [(Option<IssueCategory>, &str); 6] = [
    (None, "All"),
    (Some(IssueCategory::SpellingAndGrammar), "Grammar"),
    (Some(IssueCategory::ClarityAndConciseness), "Clarity"),
    (Some(IssueCategory::StyleAndVocabulary), "Style"),
    (Some(IssueCategory::ToneAndAppropriateness), "Tone"),
    (Some(IssueCategory::StyleGuide), "Guide"),
];

pub fn show(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    show_tab_strip(app, ui);
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("sidebar_scroll")
        .show(ui, |ui| match app.active_tab {
            SidebarTab::Alerts => show_alerts_tab(app, ui),
            SidebarTab::GoalsAndScore => show_goals_tab(app, ui),
            SidebarTab::Tone => show_tone_tab(app, ui),
            SidebarTab::StyleAndSnippets => show_style_snippets_tab(app, ui),
        });
}

fn show_tab_strip(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    let tabs = [
        (
            SidebarTab::Alerts,
            format!("Alerts ({})", app.issues().len()),
        ),
        (SidebarTab::GoalsAndScore, "Goals & Score".to_owned()),
        (SidebarTab::Tone, "Tone Profile".to_owned()),
        (SidebarTab::StyleAndSnippets, "Style & Snippets".to_owned()),
    ];
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (idx, (tab, label)) in tabs.into_iter().enumerate() {
            let active = app.active_tab == tab;
            let btn = if active {
                primary_button(ui, label, true)
            } else {
                craft_button(ui, label, true)
            };
            if idx == 0 && app.focus_request == Some(PaneFocusTarget::Sidebar) {
                btn.response.request_focus();
                app.focus_request = None;
            }
            if btn.clicked() {
                app.active_tab = tab;
            }
        }
    });
}

fn show_alerts_tab(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    show_category_filters(app, ui);
    ui.add_space(6.0);

    let filtered: Vec<Issue> = app
        .issues()
        .iter()
        .filter(|issue| app.category_filter.is_none_or(|c| issue.category == c))
        .cloned()
        .collect();

    if filtered.is_empty() {
        ui.label("No open alerts in this category.");
        return;
    }

    for issue in &filtered {
        show_issue_card(app, ui, issue);
        ui.add_space(6.0);
    }
}

fn show_category_filters(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (cat, label) in CATEGORY_FILTERS {
            let selected = app.category_filter == cat;
            let btn = if selected {
                primary_button(ui, label, true)
            } else {
                craft_button(ui, label, true)
            };
            if btn.clicked() {
                app.category_filter = cat;
            }
        }
    });
}

fn show_issue_card(app: &mut GrammarCraftApp, ui: &mut egui::Ui, issue: &Issue) {
    let style = category_style(issue.category, issue.severity, &app.colors);
    let selected = app.selected_issue_id == Some(issue.id.get());
    let stroke_color = if selected {
        app.colors.accent
    } else {
        app.colors.border
    };
    let excerpt = issue
        .span
        .slice(app.session.doc().text())
        .unwrap_or("")
        .to_owned();
    let single_word = !excerpt.trim().is_empty() && !excerpt.trim().contains(char::is_whitespace);

    egui::Frame::group(ui.style())
        .stroke(egui::Stroke::new(
            if selected { 1.8 } else { 1.0 },
            stroke_color,
        ))
        .fill(if selected {
            app.colors.accent_soft
        } else {
            app.colors.bg_raised
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                Badge::new(format!("{} {}", style.severity_badge, style.category_label))
                    .variant(Variant::look(Emphasis::Soft, Tone::Info))
                    .show(ui);
                if craft_button(ui, format!("Select `{excerpt}`"), true).clicked() {
                    app.select_issue(issue.id.get(), (issue.span.start(), issue.span.end()));
                }
            });
            ui.strong(&issue.title);
            ui.label(&issue.explanation);

            show_issue_actions(app, ui, issue, &excerpt, single_word);
        });
}

fn show_issue_actions(
    app: &mut GrammarCraftApp,
    ui: &mut egui::Ui,
    issue: &Issue,
    excerpt: &str,
    single_word: bool,
) {
    let issue_id = issue.id.get();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (idx, rep) in issue.replacements.iter().take(3).enumerate() {
            let label = if idx == 0 {
                format!("Apply: {}", rep.text)
            } else {
                rep.text.clone()
            };
            let btn = if idx == 0 {
                primary_button(ui, label, true)
            } else {
                craft_button(ui, label, true)
            };
            if btn.clicked() {
                app.apply_suggestion(issue_id, idx);
            }
        }
        if craft_button(ui, "Dismiss Once", true).clicked() {
            app.dismiss_once(issue_id);
        }
        if craft_button(ui, "Dismiss Document", true).clicked() {
            app.dismiss_document(issue_id);
        }
        if single_word && craft_button(ui, "Add to Dictionary", true).clicked() {
            app.add_word_to_dictionary(excerpt);
        }
    });
}

fn show_goals_tab(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    ui.heading("Writing Goals");
    let current = *app.session.goals();
    let mut updated = current;

    show_goal_selectors(&mut updated, ui);
    if updated != current {
        app.set_goals(updated);
    }

    ui.add_space(10.0);
    ui.separator();
    ui.heading("Writing Score");
    let score = app.writing_score();
    ui.strong(format!("Overall: {} / 100", score.score));
    ui.label(format!(
        "Spelling & Grammar: {} | Clarity: {} | Style: {} | Tone: {} | Style Guide: {}",
        score.breakdown.spelling_and_grammar,
        score.breakdown.clarity_and_conciseness,
        score.breakdown.style_and_vocabulary,
        score.breakdown.tone_and_appropriateness,
        score.breakdown.style_guide,
    ));

    ui.add_space(10.0);
    ui.separator();
    ui.heading("Readability & Stats");
    let r = app.readability();
    let stats = app.session.doc().stats();
    ui.label(format!("Level: {}", readability_label(r.level)));
    ui.label(format!("Flesch Reading Ease: {:.1}", r.flesch_reading_ease));
    ui.label(format!(
        "Flesch-Kincaid Grade: {:.1}",
        r.flesch_kincaid_grade
    ));
    ui.label(format!("Gulpease Index: {:.1}", r.gulpease_index));
    ui.label(format!(
        "Reading time: {:.1} min | Speaking time: {:.1} min",
        r.reading_minutes, r.speaking_minutes
    ));
    ui.label(format!(
        "Words: {} | Sentences: {} | Paragraphs: {}",
        stats.words, stats.sentences, stats.paragraphs
    ));
}

fn show_goal_selectors(goals: &mut WritingGoals, ui: &mut egui::Ui) {
    egui::Grid::new("goals_grid")
        .num_columns(2)
        .spacing([8.0, 6.0])
        .show(ui, |ui| {
            ui.label("Audience");
            let aud_combo = egui::ComboBox::from_id_salt("goal_audience")
                .selected_text(goals.audience.as_str())
                .show_ui(ui, |ui| {
                    for aud in GrammarCraftApp::audiences() {
                        ui.selectable_value(&mut goals.audience, aud, aud.as_str());
                    }
                });
            ui.ctx()
                .accesskit_node_builder(aud_combo.response.id, |node| {
                    node.set_label("Audience");
                });
            ui.end_row();

            ui.label("Formality");
            let form_combo = egui::ComboBox::from_id_salt("goal_formality")
                .selected_text(goals.formality.as_str())
                .show_ui(ui, |ui| {
                    for form in GrammarCraftApp::formalities() {
                        ui.selectable_value(&mut goals.formality, form, form.as_str());
                    }
                });
            ui.ctx()
                .accesskit_node_builder(form_combo.response.id, |node| {
                    node.set_label("Formality");
                });
            ui.end_row();

            ui.label("Context");
            let ctx_combo = egui::ComboBox::from_id_salt("goal_context")
                .selected_text(goals.context.as_str())
                .show_ui(ui, |ui| {
                    for ctx in GrammarCraftApp::contexts() {
                        ui.selectable_value(&mut goals.context, ctx, ctx.as_str());
                    }
                });
            ui.ctx()
                .accesskit_node_builder(ctx_combo.response.id, |node| {
                    node.set_label("Context");
                });
            ui.end_row();

            ui.label("Purpose");
            let purp_combo = egui::ComboBox::from_id_salt("goal_purpose")
                .selected_text(goals.purpose.as_str())
                .show_ui(ui, |ui| {
                    for purp in GrammarCraftApp::purposes() {
                        ui.selectable_value(&mut goals.purpose, purp, purp.as_str());
                    }
                });
            ui.ctx()
                .accesskit_node_builder(purp_combo.response.id, |node| {
                    node.set_label("Purpose");
                });
            ui.end_row();
        });
}

fn show_tone_tab(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    ui.heading("Detected Tone");
    let tone = app.tone_profile();
    if tone.dominant.is_empty() {
        ui.label("Write at least a sentence to detect dominant tones.");
        return;
    }

    for signal in &tone.dominant {
        show_tone_signal(app, ui, signal);
        ui.add_space(6.0);
    }
}

fn show_tone_signal(app: &mut GrammarCraftApp, ui: &mut egui::Ui, signal: &ToneSignal) {
    egui::Frame::group(ui.style())
        .fill(app.colors.bg_raised)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.strong(signal.kind.as_str());
                ui.label(format!("{}%", signal.strength));
            });
            ui.add(egui::ProgressBar::new(f32::from(signal.strength) / 100.0));
            ui.horizontal_wrapped(|ui| {
                for (idx, span) in signal.evidence_spans.iter().enumerate() {
                    let excerpt = span.slice(app.session.doc().text()).unwrap_or("span");
                    let label = format!("Evidence #{}: `{excerpt}`", idx + 1);
                    if craft_button(ui, label, true).clicked() {
                        app.select_tone_span((span.start(), span.end()));
                    }
                }
            });
        });
}

fn show_style_snippets_tab(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    ui.heading("Style Guide Rules");
    show_style_form(app, ui);
    ui.add_space(6.0);
    let rules = app.session.style_guide().rules().to_vec();
    if rules.is_empty() {
        ui.label("No style rules configured.");
    } else {
        for rule in &rules {
            ui.horizontal_wrapped(|ui| {
                let desc = match &rule.kind {
                    StyleRuleKind::PreferredTerm {
                        discouraged,
                        preferred,
                        ..
                    } => format!("{}: `{discouraged}` -> `{preferred}`", rule.id),
                    StyleRuleKind::BrandCasing { exact, .. } => {
                        format!("{}: exact `{exact}`", rule.id)
                    }
                    StyleRuleKind::PatternRule { pattern, .. } => {
                        format!("{}: /{pattern}/", rule.id)
                    }
                };
                ui.label(desc);
                if craft_button(ui, format!("Remove {}", rule.id), true).clicked() {
                    app.remove_style_rule(&rule.id);
                }
            });
        }
    }

    ui.add_space(10.0);
    ui.separator();
    ui.heading("Snippets");
    show_snippet_form(app, ui);
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label("Vars (k=v, ...):");
        let vars_resp = ui.text_edit_singleline(&mut app.snippet_draft.expand_vars);
        ui.ctx().accesskit_node_builder(vars_resp.id, |node| {
            node.set_label("Snippet Variables");
        });
    });
    let snippets: Vec<_> = app.session.snippets().list().into_iter().cloned().collect();
    if snippets.is_empty() {
        ui.label("No snippets saved.");
    } else {
        for snippet in &snippets {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&snippet.prefix);
                ui.label(&snippet.title);
                if primary_button(ui, format!("Insert {}", snippet.prefix), true).clicked() {
                    app.expand_and_insert_snippet(&snippet.prefix);
                }
                if craft_button(ui, format!("Delete {}", snippet.prefix), true).clicked() {
                    app.remove_snippet(&snippet.prefix);
                }
            });
        }
    }
}

fn show_style_form(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut app.style_draft.kind,
            StyleRuleDraftKind::PreferredTerm,
            "Preferred Term",
        );
        ui.selectable_value(
            &mut app.style_draft.kind,
            StyleRuleDraftKind::BrandCasing,
            "Brand Casing",
        );
    });
    egui::Grid::new("style_rule_form")
        .num_columns(2)
        .spacing([6.0, 4.0])
        .show(ui, |ui| {
            ui.label("Rule ID");
            let id_resp = ui.text_edit_singleline(&mut app.style_draft.id);
            ui.ctx().accesskit_node_builder(id_resp.id, |node| {
                node.set_label("Style Rule ID");
            });
            ui.end_row();
            let term_label = match app.style_draft.kind {
                StyleRuleDraftKind::PreferredTerm => "Discouraged",
                StyleRuleDraftKind::BrandCasing => "Exact Brand",
            };
            ui.label(term_label);
            let term_resp = ui.text_edit_singleline(&mut app.style_draft.term);
            ui.ctx().accesskit_node_builder(term_resp.id, |node| {
                node.set_label(term_label);
            });
            ui.end_row();
            if app.style_draft.kind == StyleRuleDraftKind::PreferredTerm {
                ui.label("Preferred");
                let pref_resp = ui.text_edit_singleline(&mut app.style_draft.preferred);
                ui.ctx().accesskit_node_builder(pref_resp.id, |node| {
                    node.set_label("Preferred Term");
                });
                ui.end_row();
            }
            ui.label("Rationale");
            let rat_resp = ui.text_edit_singleline(&mut app.style_draft.rationale);
            ui.ctx().accesskit_node_builder(rat_resp.id, |node| {
                node.set_label("Style Rule Rationale");
            });
            ui.end_row();
        });
    if primary_button(ui, "Add Style Rule", true).clicked() {
        app.add_draft_style_rule();
    }
}

fn show_snippet_form(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    egui::Grid::new("snippet_form")
        .num_columns(2)
        .spacing([6.0, 4.0])
        .show(ui, |ui| {
            ui.label("Prefix");
            let prefix_resp = ui.text_edit_singleline(&mut app.snippet_draft.prefix);
            ui.ctx().accesskit_node_builder(prefix_resp.id, |node| {
                node.set_label("Snippet Prefix");
            });
            ui.end_row();
            ui.label("Title");
            let title_resp = ui.text_edit_singleline(&mut app.snippet_draft.title);
            ui.ctx().accesskit_node_builder(title_resp.id, |node| {
                node.set_label("Snippet Title");
            });
            ui.end_row();
            ui.label("Body");
            let body_resp = ui.text_edit_singleline(&mut app.snippet_draft.body);
            ui.ctx().accesskit_node_builder(body_resp.id, |node| {
                node.set_label("Snippet Body");
            });
            ui.end_row();
        });
    if primary_button(ui, "Add Snippet", true).clicked() {
        app.add_draft_snippet();
    }
}

fn readability_label(level: ReadabilityLevel) -> &'static str {
    match level {
        ReadabilityLevel::VeryEasy => "Very Easy",
        ReadabilityLevel::Easy => "Easy",
        ReadabilityLevel::Standard => "Standard",
        ReadabilityLevel::Difficult => "Difficult",
        ReadabilityLevel::Academic => "Academic",
    }
}
