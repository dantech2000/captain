#!/usr/bin/env bash
# Renders the app icon SVGs in assets/icon/ into a macOS iconset and Captain.icns.
# Needs rsvg-convert (librsvg) and, for the .icns, macOS iconutil.
# 16 and 32 px use captain-small.svg, which drops the glass edges so the wheel stays crisp.
set -euo pipefail

cd "$(dirname "$0")/../assets/icon"
out=Captain.iconset
rm -rf "$out"
mkdir -p "$out"

render() { rsvg-convert -w "$2" -h "$2" "$1" -o "$out/$3"; }

for size in 16 32; do
  render captain-small.svg "$size" "icon_${size}x${size}.png"
  render captain-small.svg "$((size * 2))" "icon_${size}x${size}@2x.png"
done
for size in 128 256 512; do
  render captain.svg "$size" "icon_${size}x${size}.png"
  render captain.svg "$((size * 2))" "icon_${size}x${size}@2x.png"
done

if command -v iconutil >/dev/null; then
  iconutil -c icns "$out" -o Captain.icns
  echo "wrote assets/icon/Captain.icns"
else
  echo "iconutil not found (not macOS); wrote only $out"
fi
