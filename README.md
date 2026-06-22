# soundcli

[![CI](https://github.com/madzem/soundcli/actions/workflows/ci.yml/badge.svg)](https://github.com/madzem/soundcli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platform: Linux](https://img.shields.io/badge/platform-Linux-informational)

A terminal **remote control + now-playing dashboard** for SoundCloud, written in Rust
([Ratatui](https://ratatui.rs)).

```
╭──────────────────────────────────────────────────────────────────────────────╮
│ soundcli v1.4.0                          Night Drive · 6 tracks · 20:54 · @neoncoast │
│────────────────────────────────────────────────────────────────────────────────────│
│ NOW PLAYING ●  Midnight City Lights  —  Neon Coast                                   │
│ 0:37 [===========>------------------------------------------]-3:05 / 3:42            │
│  ⏮ prev   ❚❚ pause   next ⏭                                  VOL ███████░░░ 70%       │
│ QUEUE · ↑↓ select · enter play                                                       │
│  01 ▶  Midnight City Lights                                       Neon Coast  3:42    │
│  02    Sunset Overdrive                                          Palm Reader  3:04    │
│ space play/pause   n next   p prev   +/− vol   ←→ seek   ↑↓ select          q quit    │
╰──────────────────────────────────────────────────────────────────────────────╯
```

## How it works (important)

soundcli **does not stream audio and does not strip ads.** Your normal, logged-in
**soundcloud.com browser tab** plays the music — full tracks, ads served as usual, and
your personalized/private sets work because it uses your own session.

soundcli talks to that tab over the **OS media bus** (MPRIS / D-Bus on Linux — the same
channel media keys and `playerctl` use). It reads now-playing metadata and sends transport
commands (play/pause/next/prev/seek/volume), then renders the interface in your terminal.

```
  soundcli (Rust/Ratatui)  ⇄  MPRIS/D-Bus  ⇄  Browser tab on soundcloud.com (plays audio + ads)
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full design and rationale.

## Install

soundcli is a **Linux** app and ships **prebuilt** — you don't need Rust or a compiler to
use it. Grab a package from the [Releases](https://github.com/madzem/soundcli/releases)
page:

```bash
# Debian / Ubuntu / Mint / Pop!_OS
sudo apt install ./soundcli_*_amd64.deb

# Fedora / RHEL / openSUSE
sudo dnf install ./soundcli-*.x86_64.rpm

# Any distro (portable binary)
tar -xzf soundcli-*-x86_64-unknown-linux-gnu.tar.gz && sudo install -m755 soundcli /usr/local/bin/

# Linuxbrew
brew install madzem/soundcli/soundcli
```

**Runtime needs:** a D-Bus session (any standard Linux desktop), a browser (it plays the
audio), and — for in-app volume only — PipeWire's `wpctl`. The only shared-library
dependency is `libdbus`, declared by the `.deb`/`.rpm`.

📖 Full step-by-step instructions (checksums, Cargo, building from source) are in
[**INSTALL.md**](INSTALL.md).

## Usage

```bash
# Open a SoundCloud set in your browser and control it from the terminal:
soundcli --playlist "https://soundcloud.com/neoncoast/sets/night-drive"

# Control whatever SoundCloud tab is already playing:
soundcli

# See the full interface with sample data — no browser needed:
soundcli --demo
```

`--playlist <url>` opens the URL in your own Chromium-family browser with an autoplay
flag (falling back to a dedicated profile, or `xdg-open` for non-Chromium browsers) so
your logged-in session resolves it — including personalized sets — then the terminal takes
over control and display. See [`config.yaml`](#configuration) to change the browser or
disable autoplay.

### Keys

| Key | Action |
|-----|--------|
| `space` | play / pause |
| `n` / `p` | next / previous |
| `←` / `→` | seek ∓5s |
| `+` / `−` | volume |
| `↑` / `↓` (`k`/`j`) | move queue selection |
| `enter` | play selected track |
| `q` / `esc` | quit |

## Queue ("up next")

MPRIS only knows the *current* track, so to show what's next soundcli fetches the set's
tracklist from SoundCloud's API (metadata only — playback and ads stay in the browser).
It auto-extracts the public web `client_id`; you can override it, and supply a token for
private/personalized sets:

```bash
export SOUNDCLOUD_CLIENT_ID=...      # optional, otherwise auto-detected
export SOUNDCLOUD_OAUTH_TOKEN=...    # required only for private/personalized sets
soundcli --playlist "https://soundcloud.com/<owner>/sets/<name>"
soundcli --dump-queue "<set-url>"    # debug: print the fetched tracklist
```

Note: this uses SoundCloud's internal API with the web client_id (as yt-dlp/scdl do) —
metadata only, no streaming/downloads, but a gray area vs the official API terms.

## Configuration

On first run soundcli writes a commented template to
`~/.config/soundcli/config.yaml` (honouring `$XDG_CONFIG_HOME`). All keys are optional:

| Key | Default | Meaning |
|-----|---------|---------|
| `browser` | auto-detect | Override the browser binary (`chromium`, `google-chrome`, `firefox`, …). |
| `autoplay` | `true` | Launch a Chromium-family browser with an autoplay flag; `false` just hands off to the default browser. |
| `managed` | `false` | Use a dedicated Chromium profile under `~/.config/soundcli` (sign in once there). |
| `client_id` | auto-extract | SoundCloud web `client_id` for the queue fetch. |
| `oauth_token` | — | Your OAuth token; only needed to resolve private/personalized sets. |

Environment variables `SOUNDCLOUD_CLIENT_ID` / `SOUNDCLOUD_OAUTH_TOKEN` take precedence
over the config file. Credentials live only in your config dir — never commit them.

Per-track **volume** is driven through PipeWire's `wpctl` (browsers ignore MPRIS volume),
so volume control needs PipeWire; everything else works without it.

## Limitations

- **Queue needs `--playlist`.** Without a set URL there's no tracklist to fetch, so the
  queue shows the current track only. Search/browse still happens in the browser.
- **A browser must be running** (it can be minimized). Audio and ads come from it.
- **Linux-first.** MPRIS is Linux/D-Bus. macOS (MediaRemote) and Windows (SMTC) are future work.
- Volume/seek fidelity depends on the browser's MPRIS implementation.

## Legal / Terms

soundcli is an independent open-source project, not affiliated with or endorsed by SoundCloud.
It is a **remote control** for the official SoundCloud web player running in your own browser —
it does not stream, download, cache, or redistribute any content, and it does not remove ads.
You are responsible for using it in accordance with
[SoundCloud's Terms of Use](https://soundcloud.com/terms-of-use). Use at your own risk.

## License

MIT — see [LICENSE](LICENSE).
