#!/usr/bin/env bash
# Build "dist/Mycelarium Rust.app" (Bevy port, Apple Silicon) from rust/ + build/assets/.
# Run from anywhere; extract assets first: python3 -I tools/extract.py
# Env: DIST_DIR (output folder, default <repo>/dist), CARGO (default: cargo on PATH or ~/.cargo/bin/cargo),
# GAME_EXE (icon source, default the Steam install's Mycelarium.exe).
# Launched normally the app shares the packaged LÖVE app's save folder (~/Library/Application Support/Mycelarium).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASSETS="$ROOT/build/assets"
DIST="${DIST_DIR:-$ROOT/dist}"
APP="$DIST/Mycelarium Rust.app"
EXE_NAME="Mycelarium Rust"
CARGO="${CARGO:-$(command -v cargo || echo "$HOME/.cargo/bin/cargo")}"
GAME_EXE="${GAME_EXE:-$HOME/GameHub/steamapps/common/Mycelarium/Mycelarium.exe}"

[[ -f "$ASSETS/sprites.json" ]] || { echo "missing $ASSETS (run: python3 -I tools/extract.py)" >&2; exit 1; }

# the `dist` profile (rust/Cargo.toml): thin LTO, stripped, abort on panic
(cd "$ROOT/rust" && "$CARGO" build --profile dist -p myco_app)
TARGET_DIR=$(cd "$ROOT/rust" && "$CARGO" metadata --format-version 1 --no-deps | jq -r .target_directory)
BIN="$TARGET_DIR/dist/myco_app"
[[ -x "$BIN" ]] || { echo "missing $BIN" >&2; exit 1; }

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/$EXE_NAME"  # already stripped by the profile; signed below
cp -R "$ASSETS" "$APP/Contents/Resources/assets"

# Minimum macOS = max(binary's deployment target (LC_BUILD_VERSION minos), 12.0):
# display.rs uses NSScreen.safeAreaInsets, which exists from macOS 12.0.
MINOS=$(otool -l "$BIN" | awk '/LC_BUILD_VERSION/ {f=1} f && /minos/ {print $2; exit}')
MINOS=$(printf '%s\n' "${MINOS:-12.0}" 12.0 | sort -t. -k1,1n -k2,2n | tail -1)

PLIST="$APP/Contents/Info.plist"
plutil -create xml1 "$PLIST"
plutil -insert CFBundleName -string "Mycelarium Rust" "$PLIST"
plutil -insert CFBundleDisplayName -string "Mycelarium Rust" "$PLIST"
plutil -insert CFBundleIdentifier -string dev.rei.mycelarium-rust "$PLIST"
plutil -insert CFBundleExecutable -string "$EXE_NAME" "$PLIST"
plutil -insert CFBundlePackageType -string APPL "$PLIST"
plutil -insert CFBundleShortVersionString -string 1.0.3-RUST "$PLIST"
plutil -insert CFBundleVersion -string 1 "$PLIST"
plutil -insert CFBundleInfoDictionaryVersion -string 6.0 "$PLIST"
plutil -insert LSMinimumSystemVersion -string "$MINOS" "$PLIST"
plutil -insert LSApplicationCategoryType -string public.app-category.games "$PLIST"
plutil -insert NSHighResolutionCapable -bool true "$PLIST"
plutil -insert NSPrincipalClass -string NSApplication "$PLIST"

# Icon: the game's own (Mycelarium.exe's 256 px icon; 512/1024 enlarged nearest-neighbour so the pixel
# art stays crisp), else a square of the splash image. Built in a temp folder, never committed.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
ICONSET="$TMP/icon.iconset"
mkdir -p "$ICONSET"
SQUARE="$TMP/icon_square.png"
if [[ -f "$GAME_EXE" ]] && python3 -I "$ROOT/tools/exe_icon.py" --exe "$GAME_EXE" "$SQUARE" \
  && python3 -I "$ROOT/tools/exe_icon.py" --exe "$GAME_EXE" --scale 2 "$TMP/icon_512.png" \
  && python3 -I "$ROOT/tools/exe_icon.py" --exe "$GAME_EXE" --scale 4 "$TMP/icon_1024.png"; then
  cp "$SQUARE" "$ICONSET/icon_128x128@2x.png"
  cp "$SQUARE" "$ICONSET/icon_256x256.png"
  cp "$TMP/icon_512.png" "$ICONSET/icon_256x256@2x.png"
  cp "$TMP/icon_512.png" "$ICONSET/icon_512x512.png"
  cp "$TMP/icon_1024.png" "$ICONSET/icon_512x512@2x.png"
  for s in 16 32 128; do
    sips -z $s $s "$SQUARE" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  done
  for s in 16 32; do
    sips -z $((s * 2)) $((s * 2)) "$SQUARE" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
  done
elif [[ -f "$ASSETS/splash.png" ]]; then
  echo "no $GAME_EXE icon; using the splash image" >&2
  SIDE=$(sips -g pixelHeight "$ASSETS/splash.png" | awk '/pixelHeight/ {print $2}')
  sips --cropToHeightWidth "$SIDE" "$SIDE" "$ASSETS/splash.png" --out "$SQUARE" >/dev/null
  for s in 16 32 128 256 512; do
    sips -z $s $s "$SQUARE" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    sips -z $((s * 2)) $((s * 2)) "$SQUARE" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
  done
fi
if [[ -f "$ICONSET/icon_16x16.png" ]]; then
  iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/GameIcon.icns"
  plutil -insert CFBundleIconFile -string GameIcon "$PLIST"
fi

plutil -lint "$PLIST" >/dev/null
codesign --force --deep -s - "$APP"
codesign --verify --deep --strict "$APP"
echo "built $APP"
