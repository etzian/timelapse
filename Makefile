.PHONY: check ui-check i18n legal licence gate dev build

# The GitHub Packages token for @etzianco/* comes from the gh keyring.
export NODE_AUTH_TOKEN ?= $(shell gh auth token -u etzian 2>/dev/null)

# fmt + clippy -D warnings + cargo test
check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

# svelte-check (0 warnings) + vitest + vite build + project checks
ui-check:
	npm --prefix frontend run check
	npm --prefix frontend run test
	npm --prefix frontend run build
	./scripts/verify.sh

# Fluent parity and hardcoded strings (fr-CA is a release blocker)
i18n:
	ez i18n-check .

# data map valid, vendors/DPAs, profile complete, documents regenerated
legal:
	ez legal check .
	ez legal gen . > /dev/null

# deny.toml + licence-exceptions.yaml; fails until the user approves option-ext
licence:
	ez licence-audit .

gate: check ui-check i18n legal licence

dev:
	cargo tauri dev

build:
	cargo tauri build --target aarch64-apple-darwin
