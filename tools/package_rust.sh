#!/usr/bin/env bash
# Build "dist/Mycelarium Rust.app" (Bevy port, Apple Silicon) from rust/ + build/assets/.
# Run from anywhere; extract assets first: python3 -I tools/extract.py
# Env: DIST_DIR (output folder, default <repo>/dist), CARGO (default: cargo on PATH or ~/.cargo/bin/cargo).
# Launched normally the app uses the LÖVE port's save folder (~/Library/Application Support/LOVE/Mycelarium).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASSETS="$ROOT/build/assets"
DIST="${DIST_DIR:-$ROOT/dist}"
APP="$DIST/Mycelarium Rust.app"
EXE_NAME="Mycelarium Rust"
CARGO="${CARGO:-$(command -v cargo || echo "$HOME/.cargo/bin/cargo")}"

[[ -f "$ASSETS/sprites.json" ]] || { echo "missing $ASSETS (run: python3 -I tools/extract.py)" >&2; exit 1; }

(cd "$ROOT/rust" && "$CARGO" build --release -p myco_app)
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/rust/target}"
BIN="$TARGET_DIR/release/myco_app"
[[ -x "$BIN" ]] || { echo "missing $BIN" >&2; exit 1; }

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/$EXE_NAME"
strip -x "$APP/Contents/MacOS/$EXE_NAME" 2>/dev/null  # local symbols only (~136 MB -> ~91 MB); signed below
cp -R "$ASSETS" "$APP/Contents/Resources/assets"

# Minimum macOS = the binary's own deployment target (LC_BUILD_VERSION minos).
MINOS=$(otool -l "$BIN" | awk '/LC_BUILD_VERSION/ {f=1} f && /minos/ {print $2; exit}')
MINOS="${MINOS:-11.0}"

PLIST="$APP/Contents/Info.plist"
plutil -create xml1 "$PLIST"
plutil -insert CFBundleName -string "Mycelarium Rust" "$PLIST"
plutil -insert CFBundleDisplayName -string "Mycelarium Rust" "$PLIST"
plutil -insert CFBundleIdentifier -string dev.rei.mycelarium-rust "$PLIST"
plutil -insert CFBundleExecutable -string "$EXE_NAME" "$PLIST"
plutil -insert CFBundlePackageType -string APPL "$PLIST"
plutil -insert CFBundleShortVersionString -string 1.0 "$PLIST"
plutil -insert CFBundleVersion -string 1 "$PLIST"
plutil -insert CFBundleInfoDictionaryVersion -string 6.0 "$PLIST"
plutil -insert LSMinimumSystemVersion -string "$MINOS" "$PLIST"
plutil -insert LSApplicationCategoryType -string public.app-category.games "$PLIST"
plutil -insert NSHighResolutionCapable -bool true "$PLIST"
plutil -insert NSPrincipalClass -string NSApplication "$PLIST"

# Icon from the game's splash image (built in a temp folder, never committed).
if [[ -f "$ASSETS/splash.png" ]]; then
  TMP="$(mktemp -d)"
  trap 'rm -rf "$TMP"' EXIT
  ICONSET="$TMP/icon.iconset"
  mkdir -p "$ICONSET"
  SQUARE="$TMP/icon_square.png"
  SIDE=$(sips -g pixelHeight "$ASSETS/splash.png" | awk '/pixelHeight/ {print $2}')
  sips --cropToHeightWidth "$SIDE" "$SIDE" "$ASSETS/splash.png" --out "$SQUARE" >/dev/null
  for s in 16 32 128 256 512; do
    sips -z $s $s "$SQUARE" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    sips -z $((s * 2)) $((s * 2)) "$SQUARE" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
  done
  iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/GameIcon.icns"
  plutil -insert CFBundleIconFile -string GameIcon "$PLIST"
fi

plutil -lint "$PLIST" >/dev/null
codesign --force --deep -s - "$APP"
codesign --verify --deep --strict "$APP"
echo "built $APP"
