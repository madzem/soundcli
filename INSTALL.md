# Installing soundcli

soundcli is a Linux terminal app. You do not need Rust or a compiler to use it.

## Quick install

x86_64 Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/madzem/soundcli/main/install.sh | sh
```

This downloads the latest release binary and installs it to `/usr/local/bin`
(or `~/.local/bin` if that needs no root).

## Requirements at runtime

- A Linux desktop with a D-Bus session (standard on GNOME, KDE, etc.).
- A browser, which plays the audio.
- PipeWire's `wpctl`, only for in-app volume control; everything else works without it.

`libdbus` is the only shared-library dependency; the distro packages declare it for you.

## Packages

Download from the [Releases](https://github.com/madzem/soundcli/releases) page
(replace `VERSION`, e.g. `1.4.0`).

Debian / Ubuntu / Mint / Pop!_OS:

```sh
sudo apt install ./soundcli_VERSION-1_amd64.deb
```

Fedora / RHEL / openSUSE:

```sh
sudo dnf install ./soundcli-VERSION-1.x86_64.rpm
```

Portable tarball (any distro):

```sh
tar -xzf soundcli-VERSION-x86_64-unknown-linux-gnu.tar.gz
sudo install -m755 soundcli /usr/local/bin/soundcli
```

Verify a download against the published checksums:

```sh
sha256sum -c SHA256SUMS --ignore-missing
```

Homebrew (Linux only — soundcli does not run on macOS):

```sh
brew install madzem/soundcli/soundcli
```

## From source

```sh
sudo apt install libdbus-1-dev pkg-config        # Debian/Ubuntu build deps
cargo install --git https://github.com/madzem/soundcli
```

Or clone and build: `cargo build --release`. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Configuration

On first run soundcli writes a commented template to `~/.config/soundcli/config.yaml`
(honouring `$XDG_CONFIG_HOME`). All keys are optional:

| Key | Default | Meaning |
|-----|---------|---------|
| `browser` | auto-detect | Browser binary to use (`chromium`, `google-chrome`, `firefox`, ...). |
| `autoplay` | `true` | Launch a Chromium-family browser with an autoplay flag; `false` hands off to the default browser. |
| `managed` | `false` | Use a dedicated Chromium profile under `~/.config/soundcli`. |
| `client_id` | auto-extract | SoundCloud web `client_id` for the queue fetch. |
| `oauth_token` | none | OAuth token, needed only for private or personalized sets. |

The environment variables `SOUNDCLOUD_CLIENT_ID` and `SOUNDCLOUD_OAUTH_TOKEN` take
precedence over the config file. Keep credentials in your config dir; never commit them.

## Queue metadata

MPRIS exposes only the current track, so to show what is next soundcli fetches the set's
tracklist from SoundCloud's internal API (metadata only — playback and ads stay in the
browser). It auto-extracts the public web `client_id`; private or personalized sets need a
token:

```sh
export SOUNDCLOUD_OAUTH_TOKEN=...
soundcli --playlist "https://soundcloud.com/<owner>/sets/<name>"
soundcli --dump-queue "<set-url>"   # print the fetched tracklist
```
