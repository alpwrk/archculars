# Maintainer: Alp <sinn1os@proton.me>
pkgname=archculars
pkgver=0.1.0
pkgrel=1
pkgdesc="Modern TUI for Arch Linux + AUR package management"
arch=('x86_64')
url="https://github.com/alpwrk/archculars"
license=('MIT')
depends=('pacman' 'gcc-libs')
optdepends=(
    'paru: AUR helper for installing AUR packages'
    'yay: alternative AUR helper'
    'polkit: graphical sudo prompts via pkexec'
)
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::file://$PWD")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$srcdir"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

check() {
    cd "$srcdir"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}

package() {
    cd "$srcdir"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
