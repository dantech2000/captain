#!/usr/bin/env bash
# Signs Captain.app (optional), wraps it in a .dmg, and notarizes the .dmg (optional).
# Run scripts/bundle-macos.sh release first. Usage: scripts/package-macos.sh
#
# Signing, when CAPTAIN_SIGN_IDENTITY is set (a "Developer ID Application: ..."
# identity in the keychain). Without it the .dmg holds an unsigned app.
# Notarization, when the app is signed and APPLE_ID, APPLE_TEAM_ID, and
# APPLE_APP_PASSWORD (an app-specific password) are set.
# See docs/features/0021-packaging.md.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

target_dir="${CARGO_TARGET_DIR:-$root/target}"
app="$target_dir/release/Captain.app"
if [[ ! -d "$app" ]]; then
  echo "package-macos: $app is missing; run scripts/bundle-macos.sh release" >&2
  exit 1
fi

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
dmg="$target_dir/release/Captain-$version-$(uname -m).dmg"
identity="${CAPTAIN_SIGN_IDENTITY:-}"

# Notarization needs every executable signed with the hardened runtime and a
# timestamp, inside out: the tools first, then the app.
sign() { codesign --force --options runtime --timestamp --sign "$identity" "$@"; }

if [[ -n "$identity" ]]; then
  resources="$app/Contents/Resources"
  if [[ -f "$resources/lima/bin/limactl" ]]; then
    sign --entitlements scripts/macos/limactl.entitlements "$resources/lima/bin/limactl"
  fi
  # bin/ holds docker and the captain CLI.
  for tool in "$resources/bin/"* "$resources/cli-plugins/"*; do
    if [[ -f "$tool" ]]; then sign "$tool"; fi
  done
  sign "$app"
  codesign --verify --strict --verbose=2 "$app"
else
  echo "package-macos: CAPTAIN_SIGN_IDENTITY is not set; the app stays unsigned"
fi

staging="$(mktemp -d "${TMPDIR:-/tmp}/captain-dmg.XXXXXX")"
trap 'rm -rf "$staging"' EXIT
cp -R "$app" "$staging/"
ln -s /Applications "$staging/Applications"
rm -f "$dmg"
hdiutil create -volname Captain -srcfolder "$staging" -fs HFS+ -format UDZO -ov "$dmg" >/dev/null

if [[ -n "$identity" ]]; then
  codesign --force --timestamp --sign "$identity" "$dmg"
  if [[ -n "${APPLE_ID:-}" && -n "${APPLE_TEAM_ID:-}" && -n "${APPLE_APP_PASSWORD:-}" ]]; then
    xcrun notarytool submit "$dmg" --wait \
      --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD"
    xcrun stapler staple "$dmg"
  else
    echo "package-macos: Apple notary credentials are not set; the .dmg is not notarized"
  fi
fi

echo "$dmg"
