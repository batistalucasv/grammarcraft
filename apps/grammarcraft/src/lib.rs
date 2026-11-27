//! GrammarCraft native `egui` desktop app powered by the Craft design system.
//!
//! Front-end rule: every document and session mutation routes strictly through
//! `Session::run(id, params)` in `grammarcraft-engine`.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod editor;
pub mod sidebar;
pub mod state;
pub mod theme_bridge;
pub mod toolbar;

#[cfg(test)]
mod tests;

pub use state::{FileAction, GrammarCraftApp, PaneFocusTarget, SidebarTab};
pub use theme_bridge::ThemeMode;

use craft::behavior::placement::Defaults;
use craft::ui::shell::app::{AppSpec, run};

pub fn app_spec() -> AppSpec {
    AppSpec::new(
        "org.artcraft.grammarcraft",
        "GrammarCraft",
        Defaults::new([1180.0, 760.0], [760.0, 480.0]),
    )
}

pub fn run_desktop() -> eframe::Result {
    run(app_spec(), |_ctx| GrammarCraftApp::new())
}
