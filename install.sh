#!/bin/sh
set -eu

repo="madzem/soundcli"

if [ "$(uname -s)" != "Linux" ]; then
    echo "soundcli runs on Linux only." >&2
    exit 1
fi

if [ "$(uname -m)" != "x86_64" ]; then
    echo "no prebuilt binary for $(uname -m)." >&2
    echo "build it with: cargo install --git https://github.com/$repo" >&2
    exit 1
fi

api="https://api.github.com/repos/$repo/releases/latest"
asset=$(curl -fsSL "$api" |
    grep -o "https://github.com/$repo/releases/download/[^\"]*x86_64-unknown-linux-gnu.tar.gz" |
    head -n1)

if [ -z "$asset" ]; then
    echo "could not find a release tarball at $api" >&2
    exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

curl -fsSL "$asset" -o "$tmp/soundcli.tar.gz"
tar -xzf "$tmp/soundcli.tar.gz" -C "$tmp"

bindir="/usr/local/bin"
sudo=""
if [ "$(id -u)" -ne 0 ] && [ ! -w "$bindir" ]; then
    if command -v sudo >/dev/null 2>&1; then
        sudo="sudo"
    else
        bindir="$HOME/.local/bin"
        mkdir -p "$bindir"
    fi
fi

$sudo install -m 0755 "$tmp/soundcli" "$bindir/soundcli"
echo "soundcli installed to $bindir/soundcli"

case ":$PATH:" in
    *":$bindir:"*) ;;
    *) echo "note: add $bindir to your PATH" ;;
esac
