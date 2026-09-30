#!/usr/bin/env bash
# Downloads the tools Captain.app ships with, checks their SHA-256 sums, and puts
# them in target/tools/<platform>/ in the same layout as Contents/Resources:
#   lima/bin/limactl, lima/share/lima, bin/docker, bin/docker-credential-osxkeychain,
#   bin/kubectl, bin/helm, cli-plugins/docker-{compose,buildx}
# Versions and pinned sums live in scripts/tool-versions.env.
# Usage: scripts/fetch-tools.sh [darwin-arm64|darwin-x86_64]  (default: this Mac)
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=tool-versions.env
source "$root/scripts/tool-versions.env"

platform="${1:-}"
if [[ -z "$platform" ]]; then
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) platform=darwin-arm64 ;;
    Darwin-x86_64) platform=darwin-x86_64 ;;
    *) echo "fetch-tools: only macOS bundles tools; pass a platform" >&2; exit 1 ;;
  esac
fi

# Each project names the architectures its own way.
case "$platform" in
  darwin-arm64)
    lima_arch=arm64 docker_arch=aarch64 compose_arch=aarch64 buildx_arch=arm64
    docker_sha="$DOCKER_SHA256_DARWIN_ARM64" osxkeychain_sha="$OSXKEYCHAIN_SHA256_DARWIN_ARM64"
    kubectl_sha="$KUBECTL_SHA256_DARWIN_ARM64" helm_sha="$HELM_SHA256_DARWIN_ARM64" ;;
  darwin-x86_64)
    lima_arch=x86_64 docker_arch=x86_64 compose_arch=x86_64 buildx_arch=amd64
    docker_sha="$DOCKER_SHA256_DARWIN_X86_64" osxkeychain_sha="$OSXKEYCHAIN_SHA256_DARWIN_X86_64"
    kubectl_sha="$KUBECTL_SHA256_DARWIN_X86_64" helm_sha="$HELM_SHA256_DARWIN_X86_64" ;;
  *) echo "fetch-tools: unknown platform $platform" >&2; exit 1 ;;
esac

target_dir="${CARGO_TARGET_DIR:-$root/target}"
out="$target_dir/tools/$platform"
stamp="lima=$LIMA_VERSION docker=$DOCKER_VERSION compose=$COMPOSE_VERSION buildx=$BUILDX_VERSION credential-helpers=$CREDENTIAL_HELPERS_VERSION kubectl=$KUBECTL_VERSION helm=$HELM_VERSION licenses=3"
if [[ -f "$out/VERSIONS" && "$(cat "$out/VERSIONS")" == "$stamp" ]]; then
  echo "$out is up to date ($stamp)"
  exit 0
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/captain-tools.XXXXXX")"
trap 'rm -rf "$work"' EXIT

sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1
}

fetch() { curl -fsSL --retry 3 -o "$2" "$1"; }

# Checks that file $1 has sum $2. Fails loudly otherwise.
verify() {
  local actual
  actual="$(sha256 "$1")"
  if [[ -z "$2" || "$actual" != "$2" ]]; then
    echo "fetch-tools: checksum mismatch for $(basename "$1"): got $actual, want ${2:-nothing}" >&2
    exit 1
  fi
}

# The sum of file name $2 in the checksum files at URLs $3... (`<sum>  <name>` or `<sum> *<name>`).
sum_from() {
  local name="$1"
  shift
  for url in "$@"; do
    fetch "$url" "$work/sums" || continue
    local sum
    sum="$(awk -v n="$name" '{f=$2; sub(/^\*/, "", f)} f == n {print $1}' "$work/sums")"
    if [[ -n "$sum" ]]; then echo "$sum"; return; fi
  done
}

rm -rf "$out"
mkdir -p "$out/lima" "$out/bin" "$out/cli-plugins"

# Lima: https://github.com/lima-vm/lima/releases
lima_name="lima-$LIMA_VERSION-Darwin-$lima_arch.tar.gz"
lima_base="https://github.com/lima-vm/lima/releases/download/v$LIMA_VERSION"
echo "Lima $LIMA_VERSION"
fetch "$lima_base/$lima_name" "$work/$lima_name"
verify "$work/$lima_name" "$(sum_from "$lima_name" "$lima_base/SHA256SUMS")"
mkdir -p "$work/lima"
tar -xzf "$work/$lima_name" -C "$work/lima"
rm "$work/$lima_name"
# limactl finds share/lima (templates, the Linux guest agent) next to bin/. The vz
# driver is built in, so libexec/lima (krunkit driver, MCP server) and the macOS
# guest agent stay out.
mkdir -p "$out/lima/bin" "$out/lima/share"
cp "$work/lima/bin/limactl" "$out/lima/bin/limactl"
cp -R "$work/lima/share/lima" "$out/lima/share/lima"
rm -f "$out/lima/share/lima/lima-guestagent.Darwin-"*
rm -rf "$work/lima"

# The docker CLI: https://download.docker.com/mac/static/stable/
docker_name="docker-$DOCKER_VERSION.tgz"
echo "docker CLI $DOCKER_VERSION"
fetch "https://download.docker.com/mac/static/stable/$docker_arch/$docker_name" "$work/$docker_name"
verify "$work/$docker_name" "$docker_sha"
tar -xzf "$work/$docker_name" -C "$work" docker/docker
mv "$work/docker/docker" "$out/bin/docker"
rm -rf "${work:?}/$docker_name" "${work:?}/docker"

# Compose: https://github.com/docker/compose/releases
compose_name="docker-compose-darwin-$compose_arch"
compose_base="https://github.com/docker/compose/releases/download/v$COMPOSE_VERSION"
echo "Compose $COMPOSE_VERSION"
fetch "$compose_base/$compose_name" "$out/cli-plugins/docker-compose"
verify "$out/cli-plugins/docker-compose" "$(sum_from "$compose_name" "$compose_base/checksums.txt")"

# Buildx: https://github.com/docker/buildx/releases. Since 0.37 the signed macOS
# binaries are listed in checksums-signed.txt instead of checksums.txt.
buildx_name="buildx-v$BUILDX_VERSION.darwin-$buildx_arch"
buildx_base="https://github.com/docker/buildx/releases/download/v$BUILDX_VERSION"
echo "Buildx $BUILDX_VERSION"
fetch "$buildx_base/$buildx_name" "$out/cli-plugins/docker-buildx"
verify "$out/cli-plugins/docker-buildx" \
  "$(sum_from "$buildx_name" "$buildx_base/checksums-signed.txt" "$buildx_base/checksums.txt")"

# The credential helper for "credsStore": "osxkeychain":
# https://github.com/docker/docker-credential-helpers/releases. Pinned in tool-versions.env.
osxkeychain_name="docker-credential-osxkeychain-v$CREDENTIAL_HELPERS_VERSION.darwin-$buildx_arch"
echo "docker-credential-osxkeychain $CREDENTIAL_HELPERS_VERSION"
fetch "https://github.com/docker/docker-credential-helpers/releases/download/v$CREDENTIAL_HELPERS_VERSION/$osxkeychain_name" \
  "$out/bin/docker-credential-osxkeychain"
verify "$out/bin/docker-credential-osxkeychain" "$osxkeychain_sha"

# kubectl: https://kubernetes.io/docs/tasks/tools/install-kubectl-macos/. Pinned
# in tool-versions.env. Kubernetes names the architectures like Buildx.
echo "kubectl $KUBECTL_VERSION"
fetch "https://dl.k8s.io/release/v$KUBECTL_VERSION/bin/darwin/$buildx_arch/kubectl" "$out/bin/kubectl"
verify "$out/bin/kubectl" "$kubectl_sha"

# Helm: https://github.com/helm/helm/releases. Pinned in tool-versions.env.
helm_name="helm-v$HELM_VERSION-darwin-$buildx_arch.tar.gz"
echo "Helm $HELM_VERSION"
fetch "https://get.helm.sh/$helm_name" "$work/$helm_name"
verify "$work/$helm_name" "$helm_sha"
tar -xzf "$work/$helm_name" -C "$work" "darwin-$buildx_arch/helm"
mv "$work/darwin-$buildx_arch/helm" "$out/bin/helm"
rm -rf "${work:?}/$helm_name" "${work:?}/darwin-$buildx_arch"

# The tools are Apache-2.0 (the credential helpers are MIT), which asks for their license and notice to ship with them.
raw="https://raw.githubusercontent.com"
for entry in "lima lima-vm/lima v$LIMA_VERSION LICENSE" \
  "docker docker/cli v$DOCKER_VERSION LICENSE" \
  "docker docker/cli v$DOCKER_VERSION NOTICE" \
  "compose docker/compose v$COMPOSE_VERSION LICENSE" \
  "buildx docker/buildx v$BUILDX_VERSION LICENSE" \
  "credential-helpers docker/docker-credential-helpers v$CREDENTIAL_HELPERS_VERSION LICENSE" \
  "kubectl kubernetes/kubernetes v$KUBECTL_VERSION LICENSE" \
  "helm helm/helm v$HELM_VERSION LICENSE"; do
  read -r tool repo tag file <<< "$entry"
  mkdir -p "$out/licenses/$tool"
  fetch "$raw/$repo/$tag/$file" "$out/licenses/$tool/$file"
done

chmod +x "$out/lima/bin/limactl" "$out/bin/"* "$out/cli-plugins/"*
echo "$stamp" > "$out/VERSIONS"
echo "$out"
