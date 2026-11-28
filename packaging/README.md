# Packaging

Release packaging scripts, icon verification, and desktop/installer metadata for GrammarCraft (`ai.storyteller.grammarcraft`).

| Path | What |
|---|---|
| `env.sh` | Shared build/version environment (`ROOT`, `VERSION` read from `Cargo.toml` `[workspace.package] version` with `GRAMMARCRAFT_VERSION` override, `DIST`, `GRAMMARCRAFT_BUILD_SHA`, `GRAMMARCRAFT_BUILD_DATE`, `CARGO_TARGET_DIR`, `APP_ID`, `copy_docs` tolerating missing `CRAFT_FONTS_DIR`) |
| `check-icons.sh` | Verifies hicolor PNG IHDR sizes (`16`..`512`), scalable SVG, `1024x1024` padded PNG, and multi-resolution `.ico` |
| `linux/package.sh`, `linux/verified-download.sh`, `linux/nfpm.yaml`, `linux/postinst.sh` | Builds and stages `.tar.gz`, `.deb`/`.rpm` (`nfpm`), and `.AppImage` |
| `linux/flatpak-bundle.sh`, `linux/flatpak/ai.storyteller.grammarcraft.bundle.yml` | Repackages the release Linux tarball into a `.flatpak` bundle and smoke-tests `grammarcraft-cli --version` |
| `linux/ai.storyteller.grammarcraft.{desktop,metainfo.xml.in,mime.xml}` | FreeDesktop entry, AppStream metadata, and MIME type definitions |
| `windows/package.ps1`, `windows/check-icons.ps1`, `windows/check-install-policy.ps1` | Builds, validates PE subsystem headers (`grammarcraft.exe` GUI subsystem 2, `grammarcraft-cli.exe` console subsystem 3), and stages WiX v5 `.msi`, portable `.zip`, and symbols `.zip` |
| `windows/grammarcraft.wxs`, `windows/grammarcraft.en-us.wxl`, `windows/install-policy.{json,wxi}`, `windows/symbols.txt` | WiX v5 installer, install policy, and symbol packaging definitions |
| `windows/sign.ps1` | Authenticode signing (no-op with warning when signing secrets are unset in `dry-run`) |

App icons (12 assets under `../assets/app-icon/`) are derived from the original `../assets/app-icon/grammarcraft.svg` and recorded in `../ATTRIBUTION.md`.
