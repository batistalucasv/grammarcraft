//! Directory confinement for agent and MCP file operations (`--automation-read-root`
//! and `--automation-write-root`).

use std::cell::RefCell;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Read or write file access mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
}

/// Allowed read and write directory roots.
#[derive(Debug)]
pub struct AutomationRoots {
    read: Option<Root>,
    write: Option<Root>,
}

#[derive(Debug)]
struct Root {
    resolved: PathBuf,
    shown: PathBuf,
}

impl Root {
    fn open(path: &Path, access: Access) -> Result<Self, String> {
        let kind = match access {
            Access::Read => "read",
            Access::Write => "write",
        };
        let resolved = std::fs::canonicalize(path)
            .and_then(|p| {
                if p.is_dir() {
                    Ok(p)
                } else {
                    Err(std::io::ErrorKind::NotADirectory.into())
                }
            })
            .map_err(|e| {
                format!(
                    "cannot open the automation {kind} root `{}`: {e}",
                    path.display()
                )
            })?;
        Ok(Self {
            shown: plain(resolved.clone()),
            resolved,
        })
    }
}

impl AutomationRoots {
    /// Builds confinement roots from existing directories.
    ///
    /// Returns `Ok(None)` when neither root is supplied.
    pub fn new(read: Option<&Path>, write: Option<&Path>) -> Result<Option<Arc<Self>>, String> {
        if read.is_none() && write.is_none() {
            return Ok(None);
        }
        let read = read.map(|p| Root::open(p, Access::Read)).transpose()?;
        let write = write.map(|p| Root::open(p, Access::Write)).transpose()?;
        Ok(Some(Arc::new(Self { read, write })))
    }

    pub fn read_root(&self) -> Option<&Path> {
        self.read.as_ref().map(|r| r.shown.as_path())
    }

    pub fn write_root(&self) -> Option<&Path> {
        self.write.as_ref().map(|r| r.shown.as_path())
    }

    /// Verifies that `path` stays inside the authorized root for `access`.
    pub fn check(&self, path: &str, access: Access) -> Result<(), String> {
        let root = match access {
            Access::Read => self.read.as_ref(),
            Access::Write => self.write.as_ref(),
        };
        let Some(Root {
            resolved: folder,
            shown,
        }) = root
        else {
            return match access {
                Access::Read => Err(format!(
                    "automation filesystem access is not granted: read authority is absent: {path}"
                )),
                Access::Write => Err(format!(
                    "automation filesystem access is not granted: write authority is absent: {path}"
                )),
            };
        };
        let entry_inside = match access {
            Access::Read => true,
            Access::Write => inside(&resolve_entry(path)?, folder),
        };
        if inside(&resolve(path)?, folder) && entry_inside {
            return Ok(());
        }
        let root_display = shown.display();
        match access {
            Access::Read => Err(format!(
                "automation path rejected: outside the read root {root_display}: {path}"
            )),
            Access::Write => Err(format!(
                "automation path rejected: outside the write root {root_display}: {path}"
            )),
        }
    }
}

#[derive(Clone, Debug)]
enum Scope {
    Confined(Arc<AutomationRoots>),
    Unconfined,
}

thread_local! {
    static SCOPE: RefCell<Option<Scope>> = const { RefCell::new(None) };
}

static PROCESS: OnceLock<Arc<AutomationRoots>> = OnceLock::new();

/// Installs process-wide confinement roots (`grammarcraft-cli mcp`).
pub fn confine_process(roots: Arc<AutomationRoots>) -> bool {
    PROCESS.set(roots).is_ok()
}

struct Restore(Option<Scope>);

impl Drop for Restore {
    fn drop(&mut self) {
        let previous = self.0.take();
        SCOPE.with(|s| *s.borrow_mut() = previous);
    }
}

fn with_scope<T>(scope: Option<Scope>, f: impl FnOnce() -> T) -> T {
    let _restore = Restore(SCOPE.with(|s| s.replace(scope)));
    f()
}

/// Enforces `roots` on the current thread for the duration of `f`.
pub fn confine<T>(roots: Option<&Arc<AutomationRoots>>, f: impl FnOnce() -> T) -> T {
    match roots {
        Some(r) => with_scope(Some(Scope::Confined(r.clone())), f),
        None => f(),
    }
}

/// Temporarily lifts process confinement for trusted internal operations.
pub fn unconfined<T>(f: impl FnOnce() -> T) -> T {
    with_scope(Some(Scope::Unconfined), f)
}

/// Returns the active confinement roots on this thread or process.
pub fn current() -> Option<Arc<AutomationRoots>> {
    match SCOPE.with(|s| s.borrow().clone()) {
        Some(Scope::Confined(r)) => Some(r),
        Some(Scope::Unconfined) => None,
        None => PROCESS.get().cloned(),
    }
}

/// Checks whether `path` may be read under active confinement roots.
pub fn check_read(path: &str) -> Result<(), String> {
    current().map_or(Ok(()), |r| r.check(path, Access::Read))
}

/// Checks whether `path` may be written under active confinement roots.
pub fn check_write(path: &str) -> Result<(), String> {
    current().map_or(Ok(()), |r| r.check(path, Access::Write))
}

fn plain(path: PathBuf) -> PathBuf {
    if !cfg!(windows) {
        return path;
    }
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    match text.strip_prefix(r"\\?\") {
        Some(rest) => PathBuf::from(rest),
        None => path,
    }
}

fn resolve(path: &str) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("automation path rejected: path is empty".to_owned());
    }
    if cfg!(windows)
        && Path::new(path)
            .components()
            .any(|c| matches!(c, Component::Normal(n) if is_device_name(&n.to_string_lossy())))
    {
        return Err(format!(
            "automation path rejected: reserved device names are not allowed: {path}"
        ));
    }
    let absolute = std::path::absolute(path)
        .map_err(|e| format!("automation path rejected: cannot resolve {path}: {e}"))?;
    resolve_absolute(path, &absolute)
}

fn resolve_entry(path: &str) -> Result<PathBuf, String> {
    let absolute = std::path::absolute(path)
        .map_err(|e| format!("automation path rejected: cannot resolve {path}: {e}"))?;
    let (Some(folder), Some(name)) = (absolute.parent(), absolute.file_name()) else {
        return Err(format!("automation path rejected: not a file name: {path}"));
    };
    let mut entry = resolve_absolute(path, folder)?;
    entry.push(name);
    Ok(entry)
}

fn resolve_absolute(path: &str, absolute: &Path) -> Result<PathBuf, String> {
    let mut missing: Vec<OsString> = Vec::new();
    let mut at = absolute;
    loop {
        if let Some(resolved) = try_canonicalize(path, at, &missing)? {
            return Ok(resolved);
        }
        let Some((name, parent)) = next_missing_parent(path, at)? else {
            let e = std::io::Error::from(std::io::ErrorKind::NotFound);
            return Err(format!(
                "automation path rejected: cannot resolve {path}: {e}"
            ));
        };
        missing.push(name);
        at = parent;
    }
}

fn try_canonicalize(
    path: &str,
    at: &Path,
    missing: &[OsString],
) -> Result<Option<PathBuf>, String> {
    match std::fs::canonicalize(at) {
        Ok(mut found) => {
            found.extend(missing.iter().rev());
            Ok(Some(found))
        }
        Err(err) => match std::fs::symlink_metadata(at) {
            Ok(meta) if meta.file_type().is_symlink() => Err(format!(
                "automation path rejected: a link that leads nowhere: {path}"
            )),
            Ok(_) => Err(format!(
                "automation path rejected: cannot resolve {path}: {err}"
            )),
            Err(meta_err) if meta_err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(meta_err) => Err(format!(
                "automation path rejected: cannot resolve {path}: {meta_err}"
            )),
        },
    }
}

fn next_missing_parent<'a>(
    path: &str,
    at: &'a Path,
) -> Result<Option<(OsString, &'a Path)>, String> {
    match (at.components().next_back(), at.parent()) {
        (Some(Component::Normal(name)), Some(parent)) => Ok(Some((name.to_os_string(), parent))),
        (Some(Component::ParentDir), _) => Err(format!(
            "automation path rejected: parent traversal below a folder that doesn't exist: {path}"
        )),
        _ => Ok(None),
    }
}

fn inside(path: &Path, root: &Path) -> bool {
    let mut path_components = path.components();
    root.components().all(|expected| {
        path_components
            .next()
            .is_some_and(|actual| same_component(actual, expected))
    })
}

fn same_component(a: Component, b: Component) -> bool {
    if a == b {
        return true;
    }
    if !cfg!(windows) {
        return false;
    }
    match (a.as_os_str().to_str(), b.as_os_str().to_str()) {
        (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
        _ => false,
    }
}

fn is_device_name(component: &str) -> bool {
    let stem = component
        .split('.')
        .next()
        .unwrap_or(component)
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    let numbered = |prefix: &str| {
        stem.strip_prefix(prefix)
            .is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
    };
    matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
    ) || numbered("COM")
        || numbered("LPT")
}
