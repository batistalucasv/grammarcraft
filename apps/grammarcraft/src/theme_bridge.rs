use std::sync::Arc;

use craft::host::target::ShellInfo;
use craft::host::{AppInfo, Host};
use craft::theme::{
    AppSizes, Appearance, Contrast, Mode, ModeChoice, Rgba, Size, TextSize, ThemeDef, ThemeId,
    Tokens, builtins, resolve_sized,
};
use craft::theme_store::{Store, StoreOptions, SuiteDir};
use craft::ui::InstallSpec;
use craft::ui::fonts::{I18nScripts, UiFonts};
use craft::ui::runtime::env::RootEnv;
use craft::ui::runtime::frame::FrameInput;
use craft::ui::testing::windows::NoWindows;
use egui::{Color32, CornerRadius, Stroke, Visuals};
use grammarcraft_core::{IssueCategory, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    pub fn mode_choice(self) -> ModeChoice {
        match self {
            Self::Light => ModeChoice::Light,
            Self::Dark => ModeChoice::Dark,
        }
    }

    pub fn theme_mode(self) -> Mode {
        match self {
            Self::Light => Mode::Light,
            Self::Dark => Mode::Dark,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderlinePattern {
    Wavy,
    Double,
    Dashed,
    Dotted,
    SolidBracket,
}

#[derive(Debug, Clone, Copy)]
pub struct CategoryStyle {
    pub color: Color32,
    pub pattern: UnderlinePattern,
    pub category_label: &'static str,
    pub severity_badge: &'static str,
}

#[derive(Debug, Clone)]
pub struct AppColors {
    pub mode: ThemeMode,
    pub bg_window: Color32,
    pub bg_panel: Color32,
    pub bg_chrome: Color32,
    pub bg_sunken: Color32,
    pub bg_raised: Color32,
    pub text_normal: Color32,
    pub text_muted: Color32,
    pub text_faint: Color32,
    pub border: Color32,
    pub divider: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub selection: Color32,
    pub status_good: Color32,
    pub status_warn: Color32,
    pub status_bad: Color32,
    pub status_info: Color32,
    pub status_bad_bg: Color32,
    pub status_good_bg: Color32,
    pub corner_radius: f32,
}

impl AppColors {
    pub fn resolve(mode: ThemeMode) -> Self {
        let theme = select_theme();
        let appearance = Appearance {
            mode: mode.theme_mode(),
            contrast: Contrast::Standard,
            size: Size::Compact,
            text_size: TextSize::default(),
            reduced_motion: false,
        };
        let resolved = resolve_sized(&theme, appearance, None, &AppSizes::default());
        Self::from_tokens(mode, &resolved.tokens)
    }

    fn from_tokens(mode: ThemeMode, tokens: &Tokens) -> Self {
        Self {
            mode,
            bg_window: to_color32(tokens.color.surface.background),
            bg_panel: to_color32(tokens.color.surface.panel),
            bg_chrome: to_color32(tokens.color.surface.chrome),
            bg_sunken: to_color32(tokens.color.surface.sunken),
            bg_raised: to_color32(tokens.color.surface.raised),
            text_normal: to_color32(tokens.color.text.normal),
            text_muted: to_color32(tokens.color.text.muted),
            text_faint: to_color32(tokens.color.text.faint),
            border: to_color32(tokens.color.line.border),
            divider: to_color32(tokens.color.line.divider),
            accent: to_color32(tokens.color.accent.solid),
            accent_soft: to_color32(tokens.color.accent.soft),
            selection: to_color32(tokens.color.state.text_selection),
            status_good: to_color32(tokens.color.status.good.solid),
            status_warn: to_color32(tokens.color.status.warn.solid),
            status_bad: to_color32(tokens.color.status.bad.solid),
            status_info: to_color32(tokens.color.status.info.solid),
            status_bad_bg: to_color32(tokens.color.status.bad.bg),
            status_good_bg: to_color32(tokens.color.status.good.bg),
            corner_radius: tokens.shape.radius.surface,
        }
    }
}

fn select_theme() -> ThemeDef {
    let mut store = Store::open(&SuiteDir::memory(), StoreOptions::new("grammarcraft 0.1"));
    let _ = store.load_selection();
    if let Ok(artcraft) = ThemeId::parse("craft.artcraft")
        && let Some(entry) = store.theme(&artcraft)
    {
        return entry.clone();
    }
    let list = builtins();
    list.iter()
        .find(|t| t.id.as_str() == craft::theme::SUITE_DEFAULT)
        .unwrap_or(&list[0])
        .clone()
}

pub fn to_color32(c: Rgba) -> Color32 {
    let [r, g, b, a] = c.0;
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    c.gamma_multiply(f32::from(alpha) / 255.0)
}

pub fn category_style(
    category: IssueCategory,
    severity: Severity,
    colors: &AppColors,
) -> CategoryStyle {
    let severity_badge = match severity {
        Severity::Error => "[!]",
        Severity::Warning => "[~]",
        Severity::Suggestion => "[i]",
    };
    let (color, pattern, category_label) = match category {
        IssueCategory::SpellingAndGrammar => (
            colors.status_bad,
            UnderlinePattern::Wavy,
            "Spelling & Grammar",
        ),
        IssueCategory::ClarityAndConciseness => {
            (colors.status_info, UnderlinePattern::Double, "Clarity")
        }
        IssueCategory::StyleAndVocabulary => (colors.accent, UnderlinePattern::Dashed, "Style"),
        IssueCategory::ToneAndAppropriateness => {
            (colors.status_warn, UnderlinePattern::Dotted, "Tone")
        }
        IssueCategory::StyleGuide => (
            colors.status_good,
            UnderlinePattern::SolidBracket,
            "Style Guide",
        ),
    };
    CategoryStyle {
        color,
        pattern,
        category_label,
        severity_badge,
    }
}

pub fn sync_egui_visuals(ctx: &egui::Context, colors: &AppColors) {
    let mut visuals = match colors.mode {
        ThemeMode::Light => Visuals::light(),
        ThemeMode::Dark => Visuals::dark(),
    };
    visuals.window_fill = colors.bg_window;
    visuals.panel_fill = colors.bg_panel;
    visuals.extreme_bg_color = colors.bg_sunken;
    visuals.faint_bg_color = colors.bg_chrome;
    visuals.override_text_color = Some(colors.text_normal);
    visuals.selection.bg_fill = colors.selection;
    visuals.selection.stroke = Stroke::new(1.0, colors.accent);
    visuals.widgets.noninteractive.bg_fill = colors.bg_panel;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text_normal);
    visuals.widgets.inactive.bg_fill = colors.bg_chrome;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text_normal);
    visuals.widgets.hovered.bg_fill = colors.accent_soft;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors.accent);
    visuals.widgets.active.bg_fill = colors.accent_soft;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, colors.accent);
    let corner = CornerRadius::same(colors.corner_radius.round().clamp(0.0, 24.0) as u8);
    visuals.widgets.noninteractive.corner_radius = corner;
    visuals.widgets.inactive.corner_radius = corner;
    ctx.set_visuals(visuals);
}

pub fn app_info() -> AppInfo {
    AppInfo {
        id: "grammarcraft",
        name: "GrammarCraft",
        version: env!("CARGO_PKG_VERSION"),
    }
}

pub fn build_host() -> Arc<Host> {
    Arc::new(Host::builder(app_info()).build())
}

pub fn frame_input(mode: ThemeMode, host: Arc<Host>) -> FrameInput {
    let artcraft = ThemeId::parse("craft.artcraft").unwrap_or_else(|_| ThemeId::suite_default());
    let root = RootEnv::new()
        .theme(artcraft)
        .mode(mode.mode_choice())
        .density(Size::Compact);
    FrameInput::new(root).host(host)
}

pub fn ui_fonts() -> UiFonts<'static> {
    UiFonts::new(&I18nScripts)
}

pub fn headless_install_spec() -> InstallSpec {
    InstallSpec::new(ShellInfo::default(), Arc::new(NoWindows))
}
