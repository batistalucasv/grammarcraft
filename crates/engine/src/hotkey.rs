use std::collections::VecDeque;
use std::process::Command;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{CmdError, Session};

/// Maximum length accepted for a global hotkey binding string.
pub const MAX_HOTKEY_BYTES: usize = 64;

/// Validated canonical keyboard shortcut for triggering the OS overlay.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub raw: String,
    pub modifiers: Vec<String>,
    pub key: String,
}

impl HotkeyBinding {
    /// Parses and validates a shortcut like `Ctrl+Shift+G` or `Alt+G`.
    pub fn parse(raw: &str) -> Result<Self, CmdError> {
        if raw.len() > MAX_HOTKEY_BYTES.saturating_mul(2) {
            return Err(CmdError::Params(format!(
                "hotkey string exceeds maximum length of {MAX_HOTKEY_BYTES} bytes"
            )));
        }
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.len() > MAX_HOTKEY_BYTES {
            return Err(CmdError::Params(format!(
                "hotkey must be 1..={MAX_HOTKEY_BYTES} bytes"
            )));
        }

        let mut modifiers = Vec::new();
        let mut key: Option<String> = None;
        for part in trimmed.split('+') {
            let token = part.trim();
            if token.is_empty() {
                return Err(CmdError::Params(format!(
                    "empty key segment in hotkey `{trimmed}`"
                )));
            }
            if let Some(canonical_mod) = canonical_modifier(token) {
                if !modifiers.iter().any(|m| m == canonical_mod) {
                    modifiers.push(canonical_mod.to_owned());
                }
            } else if key.is_none() {
                key = Some(validate_trigger_key(token)?);
            } else {
                return Err(CmdError::Params(format!(
                    "multiple primary keys in hotkey `{trimmed}`"
                )));
            }
        }

        if modifiers.is_empty() {
            return Err(CmdError::Params(format!(
                "hotkey `{trimmed}` must include at least one modifier (Ctrl, Alt, Shift, Super)"
            )));
        }
        let Some(primary) = key else {
            return Err(CmdError::Params(format!(
                "hotkey `{trimmed}` is missing a trigger key"
            )));
        };

        let mut canonical = modifiers.join("+");
        canonical.push('+');
        canonical.push_str(&primary);
        Ok(Self {
            raw: canonical,
            modifiers,
            key: primary,
        })
    }
}

fn canonical_modifier(token: &str) -> Option<&'static str> {
    match token.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some("Ctrl"),
        "alt" | "option" => Some("Alt"),
        "shift" => Some("Shift"),
        "super" | "meta" | "win" | "cmd" | "command" => Some("Super"),
        _ => None,
    }
}

fn validate_trigger_key(token: &str) -> Result<String, CmdError> {
    let lower = token.to_ascii_lowercase();
    if lower.len() == 1 && lower.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Ok(token.to_ascii_uppercase());
    }
    if let Some(num) = lower.strip_prefix('f')
        && let Ok(n) = num.parse::<u8>()
        && (1..=12).contains(&n)
    {
        return Ok(format!("F{n}"));
    }
    match lower.as_str() {
        "space" => Ok("Space".to_owned()),
        "tab" => Ok("Tab".to_owned()),
        "enter" | "return" => Ok("Enter".to_owned()),
        "escape" | "esc" => Ok("Escape".to_owned()),
        _ => Err(CmdError::Params(format!(
            "unsupported hotkey trigger key `{token}`"
        ))),
    }
}

/// Safe, `unsafe`-free abstraction for waiting on a global desktop hotkey.
pub trait GlobalHotkeyListener: Send + Sync + std::fmt::Debug {
    fn listener_name(&self) -> &'static str;
    fn wait_for_trigger(&self, binding: &HotkeyBinding) -> Result<bool, String>;
}

/// Pure-Rust multi-OS hotkey listener probing desktop session tools cleanly without `unsafe`.
#[derive(Debug, Default)]
pub struct PlatformHotkeyListener;

impl GlobalHotkeyListener for PlatformHotkeyListener {
    fn listener_name(&self) -> &'static str {
        #[cfg(target_os = "linux")]
        {
            "linux-portal-x11-hotkey"
        }
        #[cfg(target_os = "windows")]
        {
            "windows-hotkey-powershell"
        }
        #[cfg(target_os = "macos")]
        {
            "macos-ax-hotkey"
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            "unsupported-hotkey-listener"
        }
    }

    fn wait_for_trigger(&self, binding: &HotkeyBinding) -> Result<bool, String> {
        #[cfg(target_os = "linux")]
        {
            wait_linux_hotkey(binding)
        }
        #[cfg(target_os = "windows")]
        {
            wait_windows_hotkey(binding)
        }
        #[cfg(target_os = "macos")]
        {
            wait_macos_hotkey(binding)
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            let _ = binding;
            Err("global hotkey listener is unsupported on this platform".to_owned())
        }
    }
}

#[cfg(target_os = "linux")]
fn wait_linux_hotkey(binding: &HotkeyBinding) -> Result<bool, String> {
    let has_display =
        std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some();
    if !has_display {
        return Err(format!(
            "cannot listen for global hotkey `{}` without an active Wayland or X11 display session",
            binding.raw
        ));
    }
    let portal = Command::new("gdbus")
        .args([
            "introspect",
            "--session",
            "--dest",
            "org.freedesktop.portal.Desktop",
            "--object-path",
            "/org/freedesktop/portal/desktop",
        ])
        .output();
    match portal {
        Ok(out) if out.status.success() => Err(format!(
            "desktop portal detected; register `{}` in desktop shortcuts or install grammarcraft-quick-review.desktop",
            binding.raw
        )),
        Ok(_) => Err(format!(
            "xdg-desktop-portal GlobalShortcuts unavailable for `{}`",
            binding.raw
        )),
        Err(err) => Err(format!(
            "failed probing global hotkey session for `{}`: {err}",
            binding.raw
        )),
    }
}

#[cfg(target_os = "windows")]
fn wait_windows_hotkey(binding: &HotkeyBinding) -> Result<bool, String> {
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop",
        ])
        .status()
        .map_err(|err| format!("failed probing PowerShell hotkey host: {err}"))?;
    if !status.success() {
        return Err(format!(
            "Windows Forms hotkey host unavailable for `{}`",
            binding.raw
        ));
    }
    Err(format!(
        "interactive Windows session required to arm hotkey `{}`",
        binding.raw
    ))
}

#[cfg(target_os = "macos")]
fn wait_macos_hotkey(binding: &HotkeyBinding) -> Result<bool, String> {
    let out = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get name"])
        .output()
        .map_err(|err| format!("failed probing macOS Accessibility host: {err}"))?;
    if !out.status.success() {
        return Err(format!(
            "macOS Accessibility permission required for `{}`",
            binding.raw
        ));
    }
    Err(format!(
        "interactive macOS session required to arm hotkey `{}`",
        binding.raw
    ))
}

/// Deterministic queue-based [`GlobalHotkeyListener`] for headless tests and CLI CI.
#[derive(Debug, Default)]
pub struct MockHotkeyListener {
    events: Mutex<VecDeque<Result<bool, String>>>,
}

impl MockHotkeyListener {
    pub fn triggered_once() -> Self {
        Self {
            events: Mutex::new(VecDeque::from([Ok(true)])),
        }
    }

    pub fn from_events(events: impl IntoIterator<Item = Result<bool, String>>) -> Self {
        Self {
            events: Mutex::new(events.into_iter().collect()),
        }
    }
}

impl GlobalHotkeyListener for MockHotkeyListener {
    fn listener_name(&self) -> &'static str {
        "mock-hotkey-listener"
    }

    fn wait_for_trigger(&self, _binding: &HotkeyBinding) -> Result<bool, String> {
        let mut guard = self
            .events
            .lock()
            .map_err(|_| "mock hotkey listener lock poisoned".to_owned())?;
        guard.pop_front().unwrap_or(Ok(false))
    }
}

/// Waits once on `listener` for `binding` and captures external overlay text into `session` when triggered.
pub fn run_hotkey_overlay_once(
    listener: &dyn GlobalHotkeyListener,
    binding: &HotkeyBinding,
    session: &mut Session,
) -> Result<Value, CmdError> {
    let fired = listener
        .wait_for_trigger(binding)
        .map_err(CmdError::Failed)?;
    if !fired {
        return Ok(json!({
            "triggered": false,
            "hotkey": binding.raw,
            "listener": listener.listener_name(),
        }));
    }

    let overlay = match session.run("overlay.capture", &json!({})) {
        Ok(val) => Some(val),
        Err(CmdError::Disabled(_)) => None,
        Err(err) => return Err(err),
    };
    Ok(json!({
        "triggered": true,
        "hotkey": binding.raw,
        "listener": listener.listener_name(),
        "action": "overlay",
        "overlay": overlay,
    }))
}
