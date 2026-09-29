#!/usr/bin/env bash
# Wraps the captain binary in Captain.app so macOS shows the app icon in the Dock,
# the app switcher, and Finder. Usage: scripts/bundle-macos.sh [debug|release]
# The result is target/<profile>/Captain.app. Open it with `open`.
set -euo pipefail

profile="${1:-debug}"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [[ "$profile" == "release" ]]; then
  cargo build -p captain-app --release
else
  cargo build -p captain-app
fi

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
target_dir="${CARGO_TARGET_DIR:-$root/target}"
app="$target_dir/$profile/Captain.app"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$target_dir/$profile/captain" "$app/Contents/MacOS/captain"
cp assets/icon/Captain.icns "$app/Contents/Resources/Captain.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Captain</string>
  <key>CFBundleDisplayName</key><string>Captain</string>
  <key>CFBundleIdentifier</key><string>dev.captain.Captain</string>
  <key>CFBundleExecutable</key><string>captain</string>
  <key>CFBundleIconFile</key><string>Captain</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

echo "$app"
