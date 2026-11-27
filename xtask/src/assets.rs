//! `cargo xtask assets`: every non-code asset must be attributed.
//!
//! The rule exists because a repository that ships an image without a licence
//! and a source cannot be redistributed safely. The check reads the working
//! tree (not `git ls-files`) so it also works in a fresh copy before `git init`
//! and catches files that are not committed yet — the ones most likely to be
//! forgotten.

use std::path::Path;

/// Extensions treated as assets. A file under [`ASSET_DIRS`] is an asset
/// whatever its extension.
const ASSET_EXT: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "ico", "icns", "svg", "pdf", "psd",
    "icc", "ttf", "otf", "woff", "woff2", "wav", "mp3", "mp4",
];

const ASSET_DIRS: &[&str] = &["assets/", "docs/images/", "examples/"];

/// Directories never walked: build output and local notebooks.
const SKIP_DIRS: &[&str] = &[".git", "target", "dist", "plan", "node_modules"];

pub fn is_asset(path: &str) -> bool {
    let extension = Path::new(path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    ASSET_DIRS
        .iter()
        .any(|directory| path.starts_with(directory))
        || ASSET_EXT.contains(&extension.as_str())
}

/// Asset paths with no `` `path` `` row in `ATTRIBUTION.md`.
pub fn missing(files: &[String], attribution: &str) -> Vec<String> {
    files
        .iter()
        .filter(|file| is_asset(file) && !attribution.contains(&format!("`{file}`")))
        .cloned()
        .collect()
}

pub fn run(root: &Path) -> Result<(), String> {
    let files = walk(root)?;
    let attribution = std::fs::read_to_string(root.join("ATTRIBUTION.md"))
        .map_err(|error| format!("ATTRIBUTION.md: {error}"))?;
    let assets = files.iter().filter(|file| is_asset(file)).count();
    let missing = missing(&files, &attribution);
    if missing.is_empty() {
        println!("assets: all {assets} asset files attributed in ATTRIBUTION.md");
        Ok(())
    } else {
        Err(format!(
            "{} asset file(s) have no ATTRIBUTION.md row (author, source, licence):\n  {}",
            missing.len(),
            missing.join("\n  ")
        ))
    }
}

/// Every file under `root`, as `/`-separated paths relative to it.
fn walk(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(&directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("{}: {error}", path.display()))?;
            if file_type.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) {
                    pending.push(path);
                }
            } else if file_type.is_symlink() {
                // A symlink is attributed by what it points at, not by a path
                // that may leave the repository.
                continue;
            } else if let Ok(relative) = path.strip_prefix(root) {
                let relative: Vec<String> = relative
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect();
                files.push(relative.join("/"));
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_need_a_row_and_code_does_not() {
        let files = vec![
            "assets/icons/a.svg".to_string(),
            "crates/core/src/lib.rs".into(),
            "docs/images/b.png".into(),
            "tests/fixture.png".into(),
            "README.md".into(),
        ];
        let attribution = "| `assets/icons/a.svg` | me | here | MIT |";
        assert_eq!(
            missing(&files, attribution),
            vec!["docs/images/b.png".to_string(), "tests/fixture.png".into()]
        );
        assert!(!is_asset("crates/core/src/lib.rs"));
        assert!(!is_asset("README.md"));
        assert!(
            is_asset("assets/fonts/OFL.txt"),
            "everything under assets/ counts"
        );
        assert!(
            is_asset("docs/images/diagram.SVG"),
            "the extension check is case-insensitive"
        );
    }

    #[test]
    fn a_failed_listing_is_an_error_not_an_empty_pass() {
        let missing_root = std::env::temp_dir().join("grammarcraft-xtask-assets-does-not-exist");
        let _ = std::fs::remove_dir_all(&missing_root);
        assert!(walk(&missing_root).is_err());
    }

    #[test]
    fn the_walk_skips_build_output() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.subsec_nanos());
        let root = std::env::temp_dir().join(format!(
            "grammarcraft-xtask-walk-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("target/debug")).ok();
        std::fs::create_dir_all(root.join("assets")).ok();
        std::fs::write(root.join("assets/a.svg"), "").ok();
        std::fs::write(root.join("target/debug/out.png"), "").ok();
        std::fs::write(root.join("README.md"), "").ok();
        let files = walk(&root);
        let _ = std::fs::remove_dir_all(&root);
        let files = files.expect("the temporary tree can be walked");
        assert!(files.contains(&"assets/a.svg".to_string()));
        assert!(files.contains(&"README.md".to_string()));
        assert!(
            !files.iter().any(|file| file.starts_with("target/")),
            "{files:?}"
        );
    }
}
