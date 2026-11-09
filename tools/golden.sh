#!/bin/sh
# Capture a reference table from the LÖVE port: runs rust/golden/scripts/<topic>.lua inside the
# booted game (MYCO_GOLDEN) and writes rust/golden/data/<topic>.json.
# Usage: tools/golden.sh <topic>   (extra env such as MYCO_SEED is passed through)
set -eu
topic="${1:?usage: tools/golden.sh <topic>}"
root="$(cd "$(dirname "$0")/.." && pwd)"
love="${LOVE:-$root/build/tools/love.app/Contents/MacOS/love}"
script="$root/rust/golden/scripts/$topic.lua"
out="$root/rust/golden/data/$topic.json"
[ -f "$script" ] || { echo "no script $script" >&2; exit 2; }
tmp="$out.tmp"
rm -f "$tmp"
trap 'rm -f "$tmp"' EXIT INT TERM
cd "$root"
MYCO_FOCUS=1 MYCO_GOLDEN="$script" MYCO_GOLDEN_OUT="$tmp" "$love" game
[ -s "$tmp" ] || { echo "golden: $topic produced no output" >&2; exit 1; }
mv "$tmp" "$out"
echo "golden: wrote $out"
