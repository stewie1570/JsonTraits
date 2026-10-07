#!/usr/bin/env bash
# Formats, lints, tests, benchmarks, and checks the packaged crate.
# The publish workflow runs this before uploading to crates.io.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

allow_dirty=()
if [[ -n "$(git status --porcelain)" ]]; then
  if [[ "${REQUIRE_CLEAN_TREE:-}" == "1" ]]; then
    echo "Working tree has uncommitted changes. Commit them before publishing." >&2
    exit 1
  fi
  allow_dirty+=(--allow-dirty)
fi

echo "==> Format"
cargo fmt --all -- --check

echo "==> Clippy"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> Tests"
cargo test --all-features

echo "==> Example"
target_dir="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
mkdir -p "$target_dir"
printf '%s\n' '{"prop1":{"prop2":"value"},"contacts":[{"info":{"name":"Stewie"}}]}' \
  | cargo run --quiet --example list_paths >"$target_dir/list_paths.out"
printf '%s\n' 'contacts.0.info.name' 'prop1.prop2' >"$target_dir/list_paths.expected"
diff -u "$target_dir/list_paths.expected" "$target_dir/list_paths.out"

echo "==> Benchmarks"
cargo bench --all-features

echo "==> Package"
cargo package "${allow_dirty[@]}" --all-features
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
crate_file="$target_dir/package/json-traits-${version}.crate"
if [[ ! -f "$crate_file" ]]; then
  echo "Packaged crate was not created: $crate_file" >&2
  exit 1
fi

work="$(mktemp -d)"
cleanup() {
  rm -rf "$work"
}
trap cleanup EXIT
tar -xzf "$crate_file" -C "$work"

echo "==> Test the packaged crate"
(
  cd "$work/json-traits-${version}"
  # A fresh target directory, so this cannot reuse the workspace build.
  CARGO_TARGET_DIR="$work/target" cargo test --all-features
)

echo "==> Publish dry run"
cargo publish --dry-run "${allow_dirty[@]}"

echo "Package is ready to publish."
