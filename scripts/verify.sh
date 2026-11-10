#!/usr/bin/env bash
#
# verify.sh — blocking project checks. Needs only bash, grep, sed and perl.
# Run by `make ui-check`. A line containing `verify:allow` is exempt from every
# check here; use it sparingly and always with a reason on the same line.
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

FAILURES=0
ok()   { printf '  ok    %s\n' "$1"; }
bad()  { printf '  FAIL  %s\n' "$1"; FAILURES=$((FAILURES + 1)); }
show() { printf '%s\n' "$1" | sed 's/^/          /'; }

# App CSS and Svelte (managed legal-ux copy excluded: it is not ours to edit).
FILES=$( { find frontend/src -name '*.css'; find frontend/src -name '*.svelte' -not -path 'frontend/src/lib/legal-ux/*'; } 2>/dev/null | sort)
strip() { perl -0pe 's{/\*(.*?)\*/}{ my $c = $1; $c =~ s/[^\n]/ /g; "  $c  " }gse' "$1"; }
scan() { # pattern files… → file:line:text, comments blanked, verify:allow lines dropped
    local pat="$1"; shift; local f
    for f in "$@"; do strip "$f" | grep -nE "$pat" | grep -v 'verify:allow' | sed "s|^|$f:|"; done
}

echo "verify.sh"

# shellcheck disable=SC2086
HITS=$(scan '#([0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3,4})([^0-9a-zA-Z_-]|$)|\brgba?\(|\bhsla?\(|\boklch\(' $FILES)
[ -n "$HITS" ] && { bad "hardcoded colours — use --ez-* or an --app-* token derived from one"; show "$HITS"; } || ok "hardcoded colours"

# shellcheck disable=SC2086
HITS=$(scan 'z-index:[[:space:]]*[-0-9]' $FILES)
[ -n "$HITS" ] && { bad "literal z-index — use --ez-z-*"; show "$HITS"; } || ok "literal z-index"

# shellcheck disable=SC2086
HITS=$(scan 'outline:[[:space:]]*(none|0)' $FILES)
[ -n "$HITS" ] && { bad "outline removed — keep a visible :focus-visible (GUIDELINES §2.1)"; show "$HITS"; } || ok "outline removal"

# shellcheck disable=SC2086
HITS=$(scan '^[[:space:]]*--ez-[a-z0-9-]+[[:space:]]*:' $FILES)
[ -n "$HITS" ] && { bad "--ez-* token redefined — add an --app-* token instead"; show "$HITS"; } || ok "--ez-* redefinition"

HITS=$(grep -rnE 'https?://' frontend/src frontend/index.html src-tauri/src crates 2>/dev/null \
    | grep -v 'frontend/src/lib/legal-ux/' | grep -v 'verify:allow' \
    | grep -vE '://(localhost|127\.0\.0\.1|tauri\.localhost|ipc\.localhost)' | grep -vE '://(www\.)?w3\.org|://schemas\.')
[ -n "$HITS" ] && { bad "external host referenced — the app calls no host unless an ADR says so"; show "$HITS"; } || ok "external hosts"

grep -qE '^[[:space:]]*tauri' crates/core/Cargo.toml && bad "crates/core depends on tauri — it must stay pure" || ok "core purity"

HITS=$( { grep -rnE 'Command::new|tauri_plugin_shell|tauri_plugin_http' crates/*/src src-tauri/src 2>/dev/null
          grep -nE 'tauri-plugin-(shell|http)|externalBin' src-tauri/Cargo.toml src-tauri/tauri.conf.json 2>/dev/null
          grep -rnE '"(shell|http|fs):' src-tauri/capabilities 2>/dev/null; } | grep -v 'verify:allow')
[ -n "$HITS" ] && { bad "subprocess or shell/http/fs plugin — widening a capability needs the user and an ADR"; show "$HITS"; } || ok "capabilities stay narrow"

# The data map must exist: privacy by design starts with knowing what is stored.
[ -f legal/data-map.yaml ] && ok "data map present" || bad "legal/data-map.yaml missing"

echo
[ "$FAILURES" -eq 0 ] && { echo "verify.sh: pass"; exit 0; }
echo "verify.sh: $FAILURES failure(s)"; exit 1
