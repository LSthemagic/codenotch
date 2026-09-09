#!/usr/bin/env bash
set -euo pipefail

profile="${1:-debug}"
host="$(rustc -vV | sed -n 's/^host: //p')"

case "$host" in
  *-unknown-linux-gnu) ;;
  *)
    echo "unsupported Linux sidecar host: $host" >&2
    exit 1
    ;;
esac

case "$profile" in
  debug)
    cargo build -p nyrva-hook --locked
    source_bin="target/debug/nyrva-hook"
    ;;
  release)
    cargo build -p nyrva-hook --release --locked
    source_bin="target/release/nyrva-hook"
    ;;
  *)
    echo "usage: $0 [debug|release]" >&2
    exit 2
    ;;
esac

destination="nyrva/binaries/nyrva-hook-$host"
mkdir -p "$(dirname "$destination")"
cp "$source_bin" "$destination"
chmod 0755 "$destination"
echo "prepared $destination from $source_bin"
