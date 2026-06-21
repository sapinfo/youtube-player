#!/usr/bin/env bash
# Build "YouTube Player.app" (a macOS .app bundle) from the release binary.
# Usage: ./build-app.sh
set -euo pipefail

cd "$(dirname "$0")"

APP="dist/YouTube Player.app"

echo "==> Building release binary"
cargo build --release

echo "==> Assembling $APP"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/youtube-player "$APP/Contents/MacOS/youtube-player"
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
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleExecutable</key>
    <string>youtube-player</string>
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
