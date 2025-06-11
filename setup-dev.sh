#!/bin/bash
set -euo pipefail

# Setup development environment for wanglandau
# This script installs required packages, the Rust toolchain, and vendors
# dependencies so that the project can be built without an internet connection.
# It is idempotent and safe to run multiple times.

main() {
    echo "[setup-dev] Starting environment setup";

    install_packages
    install_rust
    vendor_dependencies
    verify_build

    echo "[setup-dev] Setup complete";
}

install_packages() {
    echo "[setup-dev] Installing system packages";
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update -y
        sudo apt-get install -y build-essential curl git
    else
        echo "Unsupported package manager. Install build tools, curl, and git manually." >&2
    fi
}

install_rust() {
    local desired_rust_version="1.87.0"
    if ! command -v rustup >/dev/null 2>&1; then
        echo "[setup-dev] Installing rustup";
        curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain "$desired_rust_version"
        export PATH="$HOME/.cargo/bin:$PATH"
    else
        export PATH="$HOME/.cargo/bin:$PATH"
        local installed_version="$(rustc --version | awk '{print $2}')"
        if [[ "$installed_version" != "$desired_rust_version" ]]; then
            echo "[setup-dev] Installing Rust $desired_rust_version";
            rustup toolchain install "$desired_rust_version" --profile minimal
            rustup default "$desired_rust_version"
        fi
    fi
}

vendor_dependencies() {
    echo "[setup-dev] Vendoring crate dependencies";
    mkdir -p .cargo
    if [ ! -f .cargo/config.toml ]; then
        cat > .cargo/config.toml <<'CFG'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
CFG
    fi
    cargo vendor vendor > /dev/null
}

verify_build() {
    echo "[setup-dev] Building project to populate cache";
    cargo build --tests > /dev/null
}

main "$@"
