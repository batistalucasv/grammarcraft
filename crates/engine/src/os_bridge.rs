use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use grammarcraft_core::MAX_DOCUMENT_BYTES;
use serde::{Deserialize, Serialize};

/// Screen-space bounding rectangle of an external text selection or caret, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnchorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Text captured from the active OS application along with optional geometry and source app name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsSelection {
    pub text: String,
    pub anchor: Option<AnchorRect>,
    pub app_name: Option<String>,
}

/// Capabilities exposed by an [`OsTextBridge`] implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsOverlayCapabilities {
    pub read_selection: bool,
    pub replace_selection: bool,
    pub anchor_rect: bool,
}

/// Safe, `unsafe`-free abstraction for capturing and replacing active OS text selections.
pub trait OsTextBridge: Send + Sync + std::fmt::Debug {
    fn adapter_name(&self) -> &'static str;
    fn capabilities(&self) -> OsOverlayCapabilities;
    fn capture_selection(&self) -> Result<Option<OsSelection>, String>;
    fn replace_selection(&self, text: &str) -> Result<(), String>;
}

/// Pure-Rust OS bridge backed by platform accessibility / clipboard CLI tools (zero `unsafe`).
#[derive(Debug, Default)]
pub struct PlatformOsBridge;

impl OsTextBridge for PlatformOsBridge {
    fn adapter_name(&self) -> &'static str {
        #[cfg(target_os = "linux")]
        {
            "linux-atspi-clipboard"
        }
        #[cfg(target_os = "windows")]
        {
            "windows-uia-clipboard"
        }
        #[cfg(target_os = "macos")]
        {
            "macos-ax-clipboard"
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            "unsupported-os-bridge"
        }
    }

    fn capabilities(&self) -> OsOverlayCapabilities {
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            OsOverlayCapabilities {
                read_selection: true,
                replace_selection: true,
                anchor_rect: true,
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            OsOverlayCapabilities {
                read_selection: false,
                replace_selection: false,
                anchor_rect: false,
            }
        }
    }

    fn capture_selection(&self) -> Result<Option<OsSelection>, String> {
        #[cfg(target_os = "linux")]
        {
            capture_linux()
        }
        #[cfg(target_os = "windows")]
        {
            capture_windows()
        }
        #[cfg(target_os = "macos")]
        {
            capture_macos()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err("OS overlay bridge is unsupported on this platform".to_owned())
        }
    }

    fn replace_selection(&self, text: &str) -> Result<(), String> {
        if text.len() > MAX_DOCUMENT_BYTES {
            return Err(format!(
                "replacement text exceeds maximum size of {MAX_DOCUMENT_BYTES} bytes"
            ));
        }
        #[cfg(target_os = "linux")]
        {
            replace_linux(text)
        }
        #[cfg(target_os = "windows")]
        {
            replace_windows(text)
        }
        #[cfg(target_os = "macos")]
        {
            replace_macos(text)
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            let _ = text;
            Err("OS overlay bridge is unsupported on this platform".to_owned())
        }
    }
}

#[derive(Debug, Default)]
struct MockBridgeState {
    selection: Option<OsSelection>,
    replaced: Vec<String>,
}

/// Deterministic in-memory [`OsTextBridge`] for headless engine and UI tests.
#[derive(Debug)]
pub struct MockOsTextBridge {
    adapter: &'static str,
    caps: OsOverlayCapabilities,
    state: Mutex<MockBridgeState>,
}

impl MockOsTextBridge {
    pub fn new(initial: Option<OsSelection>) -> Self {
        Self {
            adapter: "mock-os-bridge",
            caps: OsOverlayCapabilities {
                read_selection: true,
                replace_selection: true,
                anchor_rect: true,
            },
            state: Mutex::new(MockBridgeState {
                selection: initial,
                replaced: Vec::new(),
            }),
        }
    }

    pub fn set_selection(&self, selection: Option<OsSelection>) {
        if let Ok(mut guard) = self.state.lock() {
            guard.selection = selection;
        }
    }

    pub fn last_replaced(&self) -> Option<String> {
        self.state
            .lock()
            .ok()
            .and_then(|guard| guard.replaced.last().cloned())
    }

    pub fn replaced_history(&self) -> Vec<String> {
        self.state
            .lock()
            .map(|guard| guard.replaced.clone())
            .unwrap_or_default()
    }
}

impl OsTextBridge for MockOsTextBridge {
    fn adapter_name(&self) -> &'static str {
        self.adapter
    }

    fn capabilities(&self) -> OsOverlayCapabilities {
        self.caps
    }

    fn capture_selection(&self) -> Result<Option<OsSelection>, String> {
        let guard = self
            .state
            .lock()
            .map_err(|_| "mock OS bridge state lock poisoned".to_owned())?;
        Ok(guard.selection.clone())
    }

    fn replace_selection(&self, text: &str) -> Result<(), String> {
        if text.len() > MAX_DOCUMENT_BYTES {
            return Err(format!(
                "replacement text exceeds maximum size of {MAX_DOCUMENT_BYTES} bytes"
            ));
        }
        let mut guard = self
            .state
            .lock()
            .map_err(|_| "mock OS bridge state lock poisoned".to_owned())?;
        guard.replaced.push(text.to_owned());
        if let Some(sel) = &mut guard.selection {
            sel.text = text.to_owned();
        }
        Ok(())
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn run_command_bounded(
    program: &str,
    args: &[&str],
    stdin_bytes: Option<&[u8]>,
) -> Result<String, String> {
    let mut cmd = Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if stdin_bytes.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    let mut child = cmd
        .spawn()
        .map_err(|err| format!("failed to launch `{program}`: {err}"))?;

    if let Some(input) = stdin_bytes
        && let Some(mut stdin) = child.stdin.take()
    {
        stdin
            .write_all(input)
            .map_err(|err| format!("failed writing stdin to `{program}`: {err}"))?;
    }

    let mut out_buf = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let limit = u64::try_from(MAX_DOCUMENT_BYTES)
            .map_err(|_| "document byte limit exceeds u64".to_owned())?;
        stdout
            .take(limit.saturating_add(1))
            .read_to_end(&mut out_buf)
            .map_err(|err| format!("failed reading stdout from `{program}`: {err}"))?;
    }
    if out_buf.len() > MAX_DOCUMENT_BYTES {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!(
            "`{program}` output exceeds maximum size of {MAX_DOCUMENT_BYTES} bytes"
        ));
    }

    let status = child
        .wait()
        .map_err(|err| format!("failed waiting for `{program}`: {err}"))?;
    if !status.success() {
        return Err(format!("`{program}` exited with status {status}"));
    }

    String::from_utf8(out_buf).map_err(|err| format!("`{program}` emitted invalid UTF-8: {err}"))
}

#[cfg(target_os = "linux")]
fn capture_linux() -> Result<Option<OsSelection>, String> {
    let app_name = probe_linux_atspi_service();
    let candidates: [(&str, &[&str]); 4] = [
        ("wl-paste", &["--primary", "--no-newline"]),
        ("wl-paste", &["--no-newline"]),
        ("xclip", &["-o", "-selection", "primary"]),
        ("xclip", &["-o", "-selection", "clipboard"]),
    ];
    let mut last_err = None;
    for (bin, args) in candidates {
        match run_command_bounded(bin, args, None) {
            Ok(text) => {
                if !text.trim().is_empty() {
                    return Ok(Some(OsSelection {
                        text,
                        anchor: None,
                        app_name,
                    }));
                }
                return Ok(None);
            }
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        "no Wayland/X11 selection tool available (install wl-clipboard or xclip)".to_owned()
    }))
}

#[cfg(target_os = "linux")]
fn probe_linux_atspi_service() -> Option<String> {
    run_command_bounded(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.a11y.Bus",
            "--object-path",
            "/org/a11y/bus",
            "--method",
            "org.a11y.Bus.GetAddress",
        ],
        None,
    )
    .ok()
    .filter(|out| !out.trim().is_empty())
    .map(|_| "linux-atspi".to_owned())
}

#[cfg(target_os = "linux")]
fn replace_linux(text: &str) -> Result<(), String> {
    let candidates: [(&str, &[&str]); 2] =
        [("wl-copy", &[]), ("xclip", &["-selection", "clipboard"])];
    let mut last_err = None;
    for (bin, args) in candidates {
        match run_command_bounded(bin, args, Some(text.as_bytes())) {
            Ok(_) => return Ok(()),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        "no Wayland/X11 clipboard writer available (install wl-clipboard or xclip)".to_owned()
    }))
}

#[cfg(target_os = "windows")]
fn capture_windows() -> Result<Option<OsSelection>, String> {
    let script = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Add-Type -AssemblyName UIAutomationClient -ErrorAction SilentlyContinue
$rect = $null
$app = $null
$selText = $null
try {
  $el = [System.Windows.Automation.AutomationElement]::FocusedElement
  if ($el) {
    $app = $el.Current.Name
    $r = $el.Current.BoundingRectangle
    if ($r -and -not [double]::IsInfinity($r.X) -and -not [double]::IsInfinity($r.Y)) {
      $rect = @($r.X, $r.Y, $r.Width, $r.Height)
    }
    $tp = $null
    if ($el.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$tp)) {
      $ranges = $tp.GetSelection()
      if ($ranges -and $ranges.Length -gt 0) { $selText = $ranges[0].GetText(-1) }
    }
  }
} catch {}
if (-not $selText) { $selText = Get-Clipboard -Raw -ErrorAction SilentlyContinue }
@{ text = $selText; app = $app; rect = $rect } | ConvertTo-Json -Compress
"#;
    let raw = run_command_bounded(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        None,
    )?;
    let parsed: serde_json::Value = serde_json::from_str(raw.trim())
        .map_err(|err| format!("failed parsing Windows UIA payload: {err}"))?;
    let text = parsed
        .get("text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    if text.trim().is_empty() {
        return Ok(None);
    }
    let anchor = parsed
        .get("rect")
        .and_then(serde_json::Value::as_array)
        .and_then(|arr| {
            let x = arr.first()?.as_f64()? as f32;
            let y = arr.get(1)?.as_f64()? as f32;
            let width = arr.get(2)?.as_f64()? as f32;
            let height = arr.get(3)?.as_f64()? as f32;
            Some(AnchorRect {
                x,
                y,
                width,
                height,
            })
        });
    let app_name = parsed
        .get("app")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    Ok(Some(OsSelection {
        text,
        anchor,
        app_name,
    }))
}

#[cfg(target_os = "windows")]
fn replace_windows(text: &str) -> Result<(), String> {
    let script = "[Console]::InputEncoding = [System.Text.Encoding]::UTF8; $t = [Console]::In.ReadToEnd(); Set-Clipboard -Value $t";
    run_command_bounded(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Some(text.as_bytes()),
    )?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn capture_macos() -> Result<Option<OsSelection>, String> {
    let text = run_command_bounded("pbpaste", &[], None)?;
    if text.trim().is_empty() {
        return Ok(None);
    }
    let app_name = run_command_bounded(
        "osascript",
        &[
            "-e",
            "tell application \"System Events\" to get name of first application process whose frontmost is true",
        ],
        None,
    )
    .ok()
    .map(|s| s.trim().to_owned())
    .filter(|s| !s.is_empty());
    Ok(Some(OsSelection {
        text,
        anchor: None,
        app_name,
    }))
}

#[cfg(target_os = "macos")]
fn replace_macos(text: &str) -> Result<(), String> {
    run_command_bounded("pbcopy", &[], Some(text.as_bytes()))?;
    Ok(())
}
