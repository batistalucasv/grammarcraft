use craft::theme::Variant;
use craft::theme::variant::{Emphasis, Tone};
use craft::ui::contract::{Props as _, Show as _};
use craft::ui::display::{Badge, Divider};
use grammarcraft_core::Issue;

use crate::editor;
use crate::sidebar;
use crate::state::{GrammarCraftApp, craft_button, primary_button};

pub fn show(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    ui.vertical(|ui| {
        show_header(app, ui);
        ui.add_space(4.0);
        show_action_bar(app, ui, &ctx);

        if let Some((msg, is_error)) = &app.status_notice {
            ui.add_space(4.0);
            let color = if *is_error {
                app.colors.status_bad
            } else {
                app.colors.status_good
            };
            ui.colored_label(color, msg);
        }

        ui.add_space(4.0);
        editor::show_with_rows(app, ui, 6, 150.0);
        ui.add_space(6.0);
        Divider::horizontal().show(ui);

        let issues: Vec<Issue> = app.issues().to_vec();
        egui::ScrollArea::vertical()
            .id_salt("quick_review_alerts")
            .show(ui, |ui| {
                if issues.is_empty() {
                    ui.label("No issues detected.");
                    return;
                }
                for issue in &issues {
                    sidebar::show_issue_card(app, ui, issue);
                    ui.add_space(6.0);
                }
            });
    });
}

fn show_header(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.strong("Quick Review");
        let score = app.writing_score().score;
        let tone = if score >= 85 {
            Tone::Good
        } else if score >= 60 {
            Tone::Warn
        } else {
            Tone::Bad
        };
        Badge::new(format!("Score: {score}/100"))
            .variant(Variant::look(Emphasis::Soft, tone))
            .show(ui);
        let theme_label = format!("Theme: {}", app.theme_mode.label());
        if craft_button(ui, theme_label, true).clicked() {
            app.toggle_theme();
        }
        if craft_button(ui, "Undo", app.session.can_undo()).clicked() {
            app.undo();
        }
    });
}

fn show_action_bar(app: &mut GrammarCraftApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let has_fixable = app.issues().iter().any(|i| !i.replacements.is_empty());
    let has_text = !app.session.doc().is_empty();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if primary_button(ui, "Apply All", has_fixable).clicked() {
            app.apply_all_suggestions();
        }
        if craft_button(ui, "Paste & Review", true).clicked() {
            app.paste_clipboard_and_review(ctx);
        }
        if craft_button(ui, "Copy Text", has_text).clicked() {
            app.copy_text_to_clipboard(ctx);
        }
    });
}
