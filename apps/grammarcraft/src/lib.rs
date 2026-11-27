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

pub mod control;
pub mod editor;
pub mod sidebar;
pub mod state;
pub mod theme_bridge;
pub mod toolbar;

#[cfg(test)]
mod tests;

pub use state::{FileAction, GrammarCraftApp, PaneFocusTarget, SidebarTab};
pub use theme_bridge::ThemeMode;

use craft::automation::control::ControlChannel;
use craft::behavior::placement::Defaults;
use craft::ui::runtime::windows::WindowHost;
use craft::ui::shell::app::{AppSpec, run};
use serde_json::json;

pub fn app_spec() -> AppSpec {
    AppSpec::new(
        "org.artcraft.grammarcraft",
        "GrammarCraft",
        Defaults::new([1180.0, 760.0], [760.0, 480.0]),
    )
}

pub fn run_desktop() -> eframe::Result {
    run_desktop_with(control::LaunchArgs::default())
}

pub fn run_desktop_with(launch: control::LaunchArgs) -> eframe::Result {
    run(app_spec(), move |ctx| {
        let mut app = GrammarCraftApp::new();
        app.session.enable_desktop_ai_storage();
        if let Some(first_file) = launch.files.first()
            && app.run_and_report("doc.open", json!({ "path": first_file }))
        {
            app.current_path = Some(first_file.clone());
            app.sync_editor_from_session();
        }
        if let Some(port) = launch.control.port {
            let host = WindowHost::of(ctx);
            match control::ControlServer::start(&launch.control, port, host) {
                Ok((rx, server)) => {
                    let channel = ControlChannel::new(rx);
                    app = app.with_control(channel, Some(server));
                }
                Err(err) => {
                    eprintln!("warning: {err}");
                    app.status_notice = Some((err, true));
                }
            }
        }
        app
    })
}
