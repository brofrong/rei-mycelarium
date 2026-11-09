#!/bin/sh
# LÖVE-vs-Rust conformance: build the release app, then run every scenario (or the ones named).
# Usage: rust/conformance/run.sh [scenario ...]   (dumps and logs under $KEEP, default a temp dir)
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root/rust"
"${CARGO:-$HOME/.cargo/bin/cargo}" build --release -p myco_app
cd "$root"
keep="${KEEP:-$(mktemp -d /tmp/myco-conformance-XXXX)}"
if [ "$#" -eq 0 ]; then
  exec python3 -I tools/conformance.py --all --keep "$keep"
fi
rc=0
for s in "$@"; do
  python3 -I tools/conformance.py "$s" --keep "$keep" || rc=1
done
exit $rc
