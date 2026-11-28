# Asset attribution

Every non-code asset in this repository (fonts, icons, images, sample documents) is listed here with
its author, source and licence. `cargo xtask assets` (part of `cargo xtask ci`) fails if an asset
file is missing from this table.

**Policy:** every asset is original work by the project's contributors or third-party material under
an open licence (OSI open source, public domain / CC0, or a Creative Commons licence that allows
redistribution). Never copy or recreate an icon, image, template, theme or screenshot from another
product, even temporarily. Art drawn in code has no file to list; say so in the table if you add a
generator.

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `assets/app-icon/grammarcraft.svg` | GrammarCraft contributors | Original work in this repository | MIT OR Apache-2.0 | Minimal geometric nib and check emblem. |
| `assets/app-icon/grammarcraft.ico` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | Multi-resolution Windows app icon (16..256 px). |
| `assets/app-icon/grammarcraft-padded-1024.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | Padded 1024x1024 master PNG icon. |
| `assets/app-icon/hicolor/scalable/apps/ai.storyteller.grammarcraft.svg` | GrammarCraft contributors | Copy of `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | Scalable Linux hicolor icon. |
| `assets/app-icon/hicolor/16x16/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 16x16 Linux hicolor PNG icon. |
| `assets/app-icon/hicolor/24x24/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 24x24 Linux hicolor PNG icon. |
| `assets/app-icon/hicolor/32x32/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 32x32 Linux hicolor PNG icon. |
| `assets/app-icon/hicolor/48x48/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 48x48 Linux hicolor PNG icon. |
| `assets/app-icon/hicolor/64x64/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 64x64 Linux hicolor PNG icon. |
| `assets/app-icon/hicolor/128x128/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 128x128 Linux hicolor PNG icon. |
| `assets/app-icon/hicolor/256x256/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 256x256 Linux hicolor PNG icon. |
| `assets/app-icon/hicolor/512x512/apps/ai.storyteller.grammarcraft.png` | GrammarCraft contributors | Derived from `assets/app-icon/grammarcraft.svg` | MIT OR Apache-2.0 | 512x512 Linux hicolor PNG icon. |

## Assets bundled through dependencies

`craft-proof` (`storytold/craft-libs` at `8be80b006888ae023d61eed265241c9b78b84981`) embeds:

| Dependency asset | Author | Source | Licence |
|---|---|---|---|
| `crates/proof/data/en-words.dic` | Grady Ward (Moby Hyphenator II); break points dropped by `craft-proof` contributors | [Project Gutenberg eBook #3204](https://www.gutenberg.org/ebooks/3204), via `storytold/craft-i18n` `language-data/en-us/en-us.dic` | Public domain |
| `crates/proof/data/en-extra.txt` | WordCraft contributors | `storytold/craft-i18n` `language-data/en-us/en-extra.txt` | MIT OR Apache-2.0 |
| Optional build-time `CRAFT_I18N_DIR` language data | Upstream `craft-i18n` / SCOWL / TeX hyphenation authors | `storytold/craft-i18n` (`language-data/`) | Recorded in `NOTICE` and `storytold/craft-libs` (`ATTRIBUTION.md`) |
