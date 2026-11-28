# Packaging

Release packaging scripts and desktop metadata for GrammarCraft (`ai.storyteller.grammarcraft`).

| Path | What |
|---|---|
| `env.sh` | Shared build/version environment (`ROOT`, `VERSION`, `DIST`, `APP_ID`, `copy_docs`) |
| `check-icons.sh` | Verifies hicolor PNG IHDR sizes, scalable SVG, 1024px padded PNG, and `.ico` |
| `linux/package.sh` | Builds and stages `.tar.gz`, `.deb`, `.rpm` (`nfpm`), and `.AppImage` |
| `linux/flatpak-bundle.sh` | Repackages the release Linux tarball into a `.flatpak` bundle |
| `linux/ai.storyteller.grammarcraft.*` | Desktop entry, AppStream `.metainfo.xml.in`, and MIME definitions |
| `windows/package.ps1` | Builds, validates PE headers, stages WiX v5 `.msi`, portable `.zip`, and symbols `.zip` |
| `windows/sign.ps1` | Authenticode signing (warns and leaves unsigned when secrets are unset) |

App icons live under `../assets/app-icon/` and every non-code asset is recorded in `../ATTRIBUTION.md`.
