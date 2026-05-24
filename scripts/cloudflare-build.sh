#!/usr/bin/env bash
# Cloudflare Pages: fast Trunk build (prebuilt trunk binary, no cargo install).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TRUNK_VERSION="0.21.14"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "$BIN_DIR"

if command -v rustup >/dev/null 2>&1; then
  rustup target add wasm32-unknown-unknown
else
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck source=/dev/null
  source "${HOME}/.cargo/env"
  rustup target add wasm32-unknown-unknown
fi

if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  source "${HOME}/.cargo/env"
fi

if ! command -v trunk >/dev/null 2>&1 || [[ "$(trunk --version 2>&1 || true)" != *"${TRUNK_VERSION}"* ]]; then
  echo "Installing trunk ${TRUNK_VERSION} (prebuilt)..."
  curl -fsSL \
    "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xzf - -C "$BIN_DIR"
  chmod +x "${BIN_DIR}/trunk"
fi

export PATH="${BIN_DIR}:${PATH}"
trunk --version

# data-wasm-opt="0" in index.html — no binaryen required on CI
trunk build --release

echo "Build OK: $(du -sh dist | cut -f1) in dist/"
