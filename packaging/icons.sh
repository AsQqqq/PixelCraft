#!/usr/bin/env bash
# Regenerate every app icon from assets/app-icon/pixelcraft.svg (the canonical master).
#
# Pure Rust (resvg, through xtask); on macOS, iconutil also writes the .icns. The outputs are
# committed, so packaging never needs these tools.
#
#   packaging/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
cargo run -q -p xtask --features icons -- icons
