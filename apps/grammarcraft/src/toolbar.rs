use craft::theme::Variant;
use craft::theme::variant::{Emphasis, Tone};
use craft::ui::contract::{Bind, Props as _, Show as _};
use craft::ui::display::{Badge, Divider};
use craft::ui::text::TextField;

use crate::state::{FileAction, GrammarCraftApp, PaneFocusTarget, craft_button, primary_button};

const SUPPORTED_LANGUAGES: [(&str, &str); 4] = [
    ("en-US", "English (US)"),
    ("en-GB", "English (UK)"),
    ("pt-BR", "Português (BR)"),
    ("es", "Español"),
];

pub fn show(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let open_btn = craft_button(ui, "Open", true);
        if app.focus_request == Some(PaneFocusTarget::Toolbar) {
            open_btn.response.request_focus();
            app.focus_request = None;
        }
        if open_btn.clicked() {
            app.path_bar_mode = Some(FileAction::Open);
            if let Some(current) = &app.current_path {
                app.path_input = current.clone();
            }
        }

        if craft_button(ui, "Save", true).clicked() {
            app.path_bar_mode = Some(FileAction::Save);
            if let Some(current) = &app.current_path {
                app.path_input = current.clone();
            }
        }

        Divider::vertical().show(ui);

        if craft_button(ui, "Undo", app.session.can_undo()).clicked() {
            app.undo();
        }
        if craft_button(ui, "Redo", app.session.can_redo()).clicked() {
            app.redo();
        }

        Divider::vertical().show(ui);
        show_language_selector(app, ui);
        show_live_translate_selector(app, ui);

        let theme_label = format!("Theme: {}", app.theme_mode.label());
        if craft_button(ui, theme_label, true).clicked() {
            app.toggle_theme();
        }

        Divider::vertical().show(ui);
        show_score_badge(app, ui);
    });

    if app.path_bar_mode.is_some() {
        ui.add_space(4.0);
        show_path_bar(app, ui);
    }

    if let Some((msg, is_error)) = &app.status_notice {
        ui.add_space(4.0);
        let color = if *is_error {
            app.colors.status_bad
        } else {
            app.colors.status_good
        };
        ui.colored_label(color, msg);
    }
}

fn show_language_selector(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    let current_lang = app.session.language_tag().to_owned();
    let mut selected_lang = current_lang.clone();
    let combo = egui::ComboBox::from_id_salt("toolbar_language")
        .selected_text(format!("Lang: {current_lang}"))
        .show_ui(ui, |ui| {
            for (tag, label) in SUPPORTED_LANGUAGES {
                ui.selectable_value(
                    &mut selected_lang,
                    tag.to_owned(),
                    format!("{tag} — {label}"),
                );
            }
        });
    ui.ctx().accesskit_node_builder(combo.response.id, |node| {
        node.set_label("Document Language");
    });
    if selected_lang != current_lang {
        app.set_language(&selected_lang);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LiveTranslateChoice {
    Off,
    En,
    PtBr,
    Es,
}

impl LiveTranslateChoice {
    fn from_app(app: &GrammarCraftApp) -> Self {
        let cfg = &app.session.doc().live_translation;
        if !cfg.enabled {
            return Self::Off;
        }
        let lower = cfg.target_lang.to_ascii_lowercase();
        if lower.starts_with("pt") {
            Self::PtBr
        } else if lower.starts_with("es") {
            Self::Es
        } else {
            Self::En
        }
    }

    const fn button_text(self) -> &'static str {
        match self {
            Self::Off => "Live: Off",
            Self::En => "Live: EN",
            Self::PtBr => "Live: PT-BR",
            Self::Es => "Live: ES",
        }
    }

    const fn option_label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::En => "EN",
            Self::PtBr => "PT-BR",
            Self::Es => "ES",
        }
    }
}

pub(crate) fn show_live_translate_selector(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    let current = LiveTranslateChoice::from_app(app);
    let mut selected = current;
    let combo = egui::ComboBox::from_id_salt(ui.scope_id().with("live_translate_selector"))
        .selected_text(current.button_text())
        .show_ui(ui, |ui| {
            for choice in [
                LiveTranslateChoice::Off,
                LiveTranslateChoice::En,
                LiveTranslateChoice::PtBr,
                LiveTranslateChoice::Es,
            ] {
                ui.selectable_value(&mut selected, choice, choice.option_label());
            }
        });
    ui.ctx().accesskit_node_builder(combo.response.id, |node| {
        node.set_label("Live Translate");
    });
    if selected != current {
        match selected {
            LiveTranslateChoice::Off => {
                let keep_target = app.session.doc().live_translation.target_lang.clone();
                app.set_live_translation(false, &keep_target);
            }
            LiveTranslateChoice::En => app.set_live_translation(true, "en"),
            LiveTranslateChoice::PtBr => app.set_live_translation(true, "pt-BR"),
            LiveTranslateChoice::Es => app.set_live_translation(true, "es"),
        }
    }
}

fn show_score_badge(app: &GrammarCraftApp, ui: &mut egui::Ui) {
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
}

fn show_path_bar(app: &mut GrammarCraftApp, ui: &mut egui::Ui) {
    let Some(mode) = app.path_bar_mode else {
        return;
    };
    ui.horizontal(|ui| {
        let action_label = match mode {
            FileAction::Open => "Open path:",
            FileAction::Save => "Save path (.txt, .md, .docx, .grammarcraft.json):",
        };
        ui.label(action_label);
        TextField::new("Document Path", Bind::bound(&mut app.path_input))
            .placeholder("document.docx")
            .show(ui);

        let confirm_label = match mode {
            FileAction::Open => "Confirm Open",
            FileAction::Save => "Confirm Save",
        };
        if primary_button(ui, confirm_label, !app.path_input.trim().is_empty()).clicked() {
            let path = app.path_input.trim().to_owned();
            match mode {
                FileAction::Open => app.open_path(&path),
                FileAction::Save => app.save_path(&path),
            }
        }
        if craft_button(ui, "Browse...", true).clicked() {
            app.request_native_file_dialog(mode);
        }
        if craft_button(ui, "Cancel", true).clicked() {
            app.path_bar_mode = None;
        }
    });
}
