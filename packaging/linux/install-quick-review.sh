#!/usr/bin/env bash
set -euo pipefail

# Install or uninstall user-scoped GrammarCraft Quick Review desktop entries,
# KDE Dolphin ServiceMenus, and GNOME Files (Nautilus) user scripts without root.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS_DIR="$DATA_HOME/applications"
KIO_DIR="$DATA_HOME/kio/servicemenus"
NAUTILUS_DIR="$DATA_HOME/nautilus/scripts"

DESKTOP_NAME="ai.storyteller.grammarcraft.quick-review.desktop"
DESKTOP_SRC="$SCRIPT_DIR/$DESKTOP_NAME"
DESKTOP_DST="$APPS_DIR/$DESKTOP_NAME"
KIO_DST="$KIO_DIR/grammarcraft-quick-review.desktop"
NAUTILUS_DST="$NAUTILUS_DIR/GrammarCraft Quick Review"

usage() {
  cat <<'EOF'
Usage: install-quick-review.sh [--uninstall] [-h|--help]

Installs GrammarCraft Quick Review into user XDG applications and file-manager
context menus (~/.local/share) without requiring root privileges.

Options:
  --uninstall   Remove user-installed desktop entry, KDE service menu, and Nautilus script
  -h, --help    Show this help
EOF
}

UNINSTALL=0
for arg in "$@"; do
  case "$arg" in
    --uninstall)
      UNINSTALL=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option '$arg'" >&2
      usage >&2
      exit 2
      ;;
  esac
done

refresh_desktop_db() {
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
  fi
}

if [[ "$UNINSTALL" -eq 1 ]]; then
  rm -f "$DESKTOP_DST" "$KIO_DST" "$NAUTILUS_DST"
  refresh_desktop_db
  echo "Removed GrammarCraft Quick Review user hooks."
  exit 0
fi

if [[ ! -f "$DESKTOP_SRC" ]]; then
  echo "error: missing desktop entry template at $DESKTOP_SRC" >&2
  exit 1
fi

mkdir -p "$APPS_DIR" "$KIO_DIR" "$NAUTILUS_DIR"

install -m 644 "$DESKTOP_SRC" "$DESKTOP_DST"

cat > "$KIO_DST" <<'EOF'
[Desktop Entry]
Type=Service
MimeType=text/plain;text/markdown;application/vnd.openxmlformats-officedocument.wordprocessingml.document;
Actions=grammarcraftQuickReview;

[Desktop Action grammarcraftQuickReview]
Name=GrammarCraft Quick Review
Icon=ai.storyteller.grammarcraft
Exec=grammarcraft --quick-review --file %f
EOF
chmod 644 "$KIO_DST"

cat > "$NAUTILUS_DST" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${NAUTILUS_SCRIPT_SELECTED_FILE_PATHS:-}" ]]; then
  first_file="$(printf '%s\n' "$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS" | head -n 1)"
elif [[ "$#" -gt 0 ]]; then
  first_file="$1"
else
  exec grammarcraft --quick-review --from-clipboard
fi

if [[ -n "$first_file" ]]; then
  exec grammarcraft --quick-review --file "$first_file"
fi
exec grammarcraft --quick-review --from-clipboard
EOF
chmod 755 "$NAUTILUS_DST"

refresh_desktop_db

cat <<EOF
Installed GrammarCraft Quick Review user integration:
  Desktop entry:   $DESKTOP_DST
  KDE ServiceMenu: $KIO_DST
  Nautilus script: $NAUTILUS_DST

Global shortcut tip (no root needed):
  Bind Ctrl+Alt+G (or your preferred shortcut) in GNOME Settings, KDE Shortcuts,
  or your Wayland compositor to:
    grammarcraft --quick-review --from-clipboard
EOF
