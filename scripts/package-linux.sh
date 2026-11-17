#!/usr/bin/env bash
# Builds a release binary and produces dist/open-rust-<version>-linux-<arch>.tar.gz and .deb.
# Validated only on x86_64 Linux. The .deb is built with dpkg-deb; it is not signed.
set -euo pipefail

cd "$(dirname "$0")/.."
VERSION="$(cargo pkgid -p open-rust | sed 's/.*[#@]//')"
ARCH_UNAME="$(uname -m)"
case "$ARCH_UNAME" in
  x86_64) DEB_ARCH=amd64 ;;
  aarch64) DEB_ARCH=arm64 ;;
  *) echo "unsupported architecture: $ARCH_UNAME" >&2; exit 1 ;;
esac

cargo build --release -p open-rust
BIN=target/release/open-rust
mkdir -p dist
NAME="open-rust-${VERSION}-linux-${ARCH_UNAME}"

# tar.gz
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/$NAME"
cp "$BIN" LICENSE-MIT LICENSE-APACHE README.md "$STAGE/$NAME/"
cp packaging/open-rust.desktop assets/icons/icon.png "$STAGE/$NAME/"
tar -C "$STAGE" -czf "dist/${NAME}.tar.gz" "$NAME"

# .deb
DEB="$STAGE/deb"
mkdir -p "$DEB/DEBIAN" "$DEB/usr/bin" "$DEB/usr/share/applications" \
         "$DEB/usr/share/icons/hicolor/512x512/apps" "$DEB/usr/share/doc/open-rust"
install -m 0755 "$BIN" "$DEB/usr/bin/open-rust"
install -m 0644 packaging/open-rust.desktop "$DEB/usr/share/applications/open-rust.desktop"
install -m 0644 assets/icons/icon.png "$DEB/usr/share/icons/hicolor/512x512/apps/open-rust.png"
install -m 0644 LICENSE-MIT LICENSE-APACHE "$DEB/usr/share/doc/open-rust/"
cat > "$DEB/DEBIAN/control" <<CONTROL
Package: open-rust
Version: ${VERSION}
Section: devel
Priority: optional
Architecture: ${DEB_ARCH}
Maintainer: robotjaol <https://github.com/robotjaol>
Description: Production-oriented starter for native Rust desktop applications
 Open Rust is an egui based desktop application template with SQLite storage,
 background tasks, a design system and a security foundation.
CONTROL
dpkg-deb --build --root-owner-group "$DEB" "dist/open-rust_${VERSION}_${DEB_ARCH}.deb" >/dev/null

echo "Created:"
ls -1 dist
