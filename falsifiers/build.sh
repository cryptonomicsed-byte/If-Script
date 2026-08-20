#!/usr/bin/env bash
# Build every falsifier and print its content address.
#
# The address is what a claim carries: Crucible is content-addressed so two
# agents can be certain they ran the same bytes. Rebuild reproducibly or the
# address moves and every claim referencing the old one dangles.
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown
echo
printf '%-34s %-8s %s\n' MODULE BYTES 'CONTENT ADDRESS'
for w in target/wasm32-unknown-unknown/release/*.wasm; do
  printf '%-34s %-8s sha256:%s\n' \
    "$(basename "$w" .wasm)" "$(stat -c%s "$w")" "$(sha256sum "$w" | cut -d' ' -f1)"
done
