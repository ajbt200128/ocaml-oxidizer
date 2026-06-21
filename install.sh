#!/bin/sh
# Install the latest ocaml-oxidizer release into ~/.local/bin.
#   curl -fsSL https://raw.githubusercontent.com/ajbt200128/ocaml-oxidizer/main/install.sh | sh
set -eu

REPO="${OCAML_OXIDIZER_REPO:-ajbt200128/ocaml-oxidizer}"

os=$(uname -s)
arch=$(uname -m)
case "$os" in
  Darwin) os=macos ;;
  Linux) os=linux ;;
  *) echo "unsupported OS: $os" >&2; exit 1 ;;
esac
case "$arch" in
  x86_64 | amd64) arch=x86_64 ;;
  arm64 | aarch64) arch=aarch64 ;;
  *) echo "unsupported arch: $arch" >&2; exit 1 ;;
esac
asset="ox-${os}-${arch}"

tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | head -1 | cut -d'"' -f4)
if [ -z "${tag}" ]; then
  echo "could not find a latest release for ${REPO}" >&2
  exit 1
fi

url="https://github.com/${REPO}/releases/download/${tag}/${asset}"
dest="${HOME}/.local/bin"
mkdir -p "$dest"

echo "downloading ${asset} (${tag})..."
curl -fsSL "$url" -o "${dest}/ox"
chmod +x "${dest}/ox"
echo "installed ${tag} -> ${dest}/ox"

case ":${PATH}:" in
  *":${dest}:"*) ;;
  *) echo "note: add ${dest} to your PATH" ;;
esac
