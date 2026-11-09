#!/usr/bin/env bash
# Build dist/Mycelarium.app from game/ + build/assets/ and LÖVE 11.5 (build/tools/love.app).
# Run from anywhere; extract assets first: python3 -I tools/extract.py
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOVE_APP="${LOVE_APP:-$ROOT/build/tools/love.app}"
ASSETS="$ROOT/build/assets"
APP="$ROOT/dist/Mycelarium.app"

[[ -d "$ASSETS" ]] || { echo "missing $ASSETS (run: python3 -I tools/extract.py)" >&2; exit 1; }
[[ -d "$LOVE_APP" ]] || { echo "missing $LOVE_APP (download love-11.5-macos.zip from github.com/love2d/love/releases)" >&2; exit 1; }

LOVEFILE="$ROOT/build/Mycelarium.love"
rm -f "$LOVEFILE"
(cd "$ROOT/game" && zip -9 -q -r "$LOVEFILE" . -x "assets" -x "assets/*")
(cd "$ROOT/build" && zip -9 -q -r "$LOVEFILE" assets)

rm -rf "$APP"
mkdir -p "$ROOT/dist"
cp -R "$LOVE_APP" "$APP"
cp "$LOVEFILE" "$APP/Contents/Resources/Mycelarium.love"

PLIST="$APP/Contents/Info.plist"
plutil -replace CFBundleName -string Mycelarium "$PLIST"
plutil -replace CFBundleDisplayName -string Mycelarium "$PLIST"
plutil -replace CFBundleIdentifier -string local.dga.mycelarium "$PLIST"
plutil -replace LSMinimumSystemVersion -string 12.0 "$PLIST"
plutil -replace NSHighResolutionCapable -bool true "$PLIST"
plutil -remove UTExportedTypeDeclarations "$PLIST" 2>/dev/null || true
plutil -remove CFBundleDocumentTypes "$PLIST" 2>/dev/null || true

# Icon from the game's splash image (kept in build/, never committed).
ICONSET="$ROOT/build/icon.iconset"
if [[ -f "$ASSETS/splash.png" ]]; then
  rm -rf "$ICONSET" && mkdir -p "$ICONSET"
  SQUARE="$ROOT/build/icon_square.png"
  SIDE=$(sips -g pixelHeight "$ASSETS/splash.png" | awk '/pixelHeight/ {print $2}')
  sips --cropToHeightWidth "$SIDE" "$SIDE" "$ASSETS/splash.png" --out "$SQUARE" >/dev/null
  for s in 16 32 128 256 512; do
    sips -z $s $s "$SQUARE" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    sips -z $((s * 2)) $((s * 2)) "$SQUARE" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
  done
  iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/GameIcon.icns"
  plutil -replace CFBundleIconFile -string GameIcon "$PLIST"
fi

codesign --force --deep -s - "$APP"
echo "built $APP"
