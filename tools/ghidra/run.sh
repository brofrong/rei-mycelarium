#!/usr/bin/env bash
# Usage: tools/ghidra/run.sh [--reuse]   (run from repo root; analysis takes a long time)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
GAME="${MYCO_GAME_DIR:-$HOME/GameHub/steamapps/common/Mycelarium}"
export GHIDRA_HEADLESS_MAXMEM="${GHIDRA_HEADLESS_MAXMEM:-12G}"
export JAVA_HOME="${JAVA_HOME:-$(brew --prefix openjdk@25)/libexec/openjdk.jdk/Contents/Home}"
PROJ="${MYCO_GHIDRA_PROJECT:-$HOME/GhidraProjects/myco}"  # Ghidra rejects path elements starting with "."
HEADLESS="${GHIDRA_HEADLESS:-$(brew --prefix ghidra)/libexec/support/analyzeHeadless}"
mkdir -p "$PROJ" "$ROOT/re/decomp"
if [[ "${1:-}" == "--reuse" ]]; then
  "$HEADLESS" "$PROJ" Myco -process Mycelarium.exe -noanalysis \
    -scriptPath "$ROOT/tools/ghidra" -postScript NameYYC.java "$ROOT/re" -postScript ExportDecomp.java "$ROOT/re/decomp"
else
  "$HEADLESS" "$PROJ" Myco -import "$GAME/Mycelarium.exe" -overwrite -max-cpu 8 \
    -scriptPath "$ROOT/tools/ghidra" -preScript PreAnalysis.java \
    -postScript NameYYC.java "$ROOT/re" -postScript ExportDecomp.java "$ROOT/re/decomp"
fi
