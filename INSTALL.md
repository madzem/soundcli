# Installing soundcli

soundcli is a **Linux** terminal app. You do **not** need Rust or a compiler to use it —
grab a prebuilt binary or distro package from the
[**Releases**](https://github.com/madzem/soundcli/releases) page.

## Requirements at runtime

- A Linux desktop with a **D-Bus** session (standard on GNOME, KDE, etc.).
- A **Chromium-family browser** or any browser (it plays the audio).
- **PipeWire** with `wpctl` — only needed for in-app volume control; everything else
  works without it.

`libdbus` is the only hard shared-library dependency; distro packages declare it for you.

---

## Debian / Ubuntu / Mint / Pop!_OS (`.deb`)

```bash
# Replace VERSION with the latest release, e.g. 1.4.0
curl -LO https://github.com/madzem/soundcli/releases/latest/download/soundcli_VERSION-1_amd64.deb
sudo apt install ./soundcli_VERSION-1_amd64.deb
```

`apt` pulls in `libdbus-1-3` automatically. Uninstall with `sudo apt remove soundcli`.

## Fedora / RHEL / CentOS / openSUSE (`.rpm`)

```bash
curl -LO https://github.com/madzem/soundcli/releases/latest/download/soundcli-VERSION-1.x86_64.rpm
sudo dnf install ./soundcli-VERSION-1.x86_64.rpm   # or: sudo zypper install ./...
```

## Any distro — portable tarball

```bash
curl -LO https://github.com/madzem/soundcli/releases/latest/download/soundcli-VERSION-x86_64-unknown-linux-gnu.tar.gz
tar -xzf soundcli-VERSION-x86_64-unknown-linux-gnu.tar.gz
sudo install -m755 soundcli /usr/local/bin/soundcli
```

Verify the download against the published checksums:

```bash
curl -LO https://github.com/madzem/soundcli/releases/latest/download/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

## Arch Linux

No official AUR package yet. Build from the tarball with the generic steps above, or
install via Cargo (below). Contributions of a `PKGBUILD` are welcome.

## Homebrew (Linux only)

soundcli cannot run on macOS (no MPRIS/D-Bus), so this is **Linuxbrew** only:

```bash
brew install madzem/soundcli/soundcli
```

(See [`packaging/homebrew/soundcli.rb`](packaging/homebrew/soundcli.rb) for the formula.)

---

## Install via Cargo (if you already have Rust)

```bash
# Build dependency: libdbus headers + pkg-config
sudo apt install libdbus-1-dev pkg-config   # Debian/Ubuntu
sudo dnf install dbus-devel pkgconf-pkg-config   # Fedora

cargo install --git https://github.com/madzem/soundcli
```

Prebuilt-binary installs (`cargo binstall soundcli`) work once the crate is published to
crates.io.

## Build from source (contributors)

```bash
git clone https://github.com/madzem/soundcli
cd soundcli
cargo build --release
./target/release/soundcli --demo
```

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## First run

```bash
soundcli --demo                                   # try the UI, no browser needed
soundcli --playlist "https://soundcloud.com/<owner>/sets/<name>"
```

soundcli writes a commented config template to `~/.config/soundcli/config.yaml` on first
run — see the [Configuration](README.md#configuration) section.
