#!/usr/bin/env bash
# Build "YouTube Player.app" (a macOS .app bundle) from the release binary,
# including the app icon. Usage: ./build-app.sh
set -euo pipefail

cd "$(dirname "$0")"

APP="dist/YouTube Player.app"
ICONSET="dist/AppIcon.iconset"

echo "==> Building release binary"
cargo build --release

echo "==> Generating app icon (.icns)"
if [ ! -f assets/icon_1024.png ]; then
    python3 assets/make_icon.py
fi
rm -rf "$ICONSET"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
    d=$((s * 2))
    sips -z "$s" "$s" assets/icon_1024.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    sips -z "$d" "$d" assets/icon_1024.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done

echo "==> Assembling $APP"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/youtube-player "$APP/Contents/MacOS/youtube-player"
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
printf 'APPL????' > "$APP/Contents/PkgInfo"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>YouTube Player</string>
    <key>CFBundleDisplayName</key>
    <string>YouTube Player</string>
    <key>CFBundleIdentifier</key>
    <string>com.nuwavenow.youtube-player</string>
    <key>CFBundleVersion</key>
    <string>0.2.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.2.0</string>
    <key>CFBundleExecutable</key>
    <string>youtube-player</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
</dict>
</plist>
PLIST

echo "==> Ad-hoc code signing"
codesign --force --deep --sign - "$APP"

echo "==> Done: $APP"
echo "    Drag it to /Applications, or run: cp -R \"$APP\" /Applications/"
