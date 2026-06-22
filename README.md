# soundcli

[![CI](https://github.com/madzem/soundcli/actions/workflows/ci.yml/badge.svg)](https://github.com/madzem/soundcli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platform: Linux](https://img.shields.io/badge/platform-Linux-informational)

Control SoundCloud from your terminal. soundcli is a Rust TUI that drives your browser's
SoundCloud tab over MPRIS and shows a now-playing dashboard with the queue. It does not
stream audio or remove ads — your logged-in browser tab plays the music.

```
╭────────────────────────────────────────────────────────────╮
│ soundcli · Night Drive · 6 tracks                          │
│                                                            │
│ NOW PLAYING ●  Midnight City Lights  —  Neon Coast         │
│ 0:37 [===========>-----------------------]  -3:05 / 3:42   │
│ prev      || pause      next            vol ███████░ 70%   │
│                                                            │
│ QUEUE                                                      │
│  01 ●  Midnight City Lights          Neon Coast   3:42     │
│  02    Sunset Overdrive              Palm Reader   3:04    │
│                                                            │
│ space play  ↑↓ select  enter jump  ←→ seek  +- vol  q quit │
╰────────────────────────────────────────────────────────────╯
```

## Install

x86_64 Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/madzem/soundcli/main/install.sh | sh
```

Packages (`.deb`, `.rpm`), Homebrew, Cargo, and build-from-source are in
[INSTALL.md](INSTALL.md). soundcli needs a D-Bus session (any standard Linux desktop) and
a browser; PipeWire's `wpctl` is used only for volume.

## Usage

```sh
soundcli --playlist "https://soundcloud.com/<owner>/sets/<name>"
soundcli            # control the SoundCloud tab already playing
soundcli --demo     # sample data, no browser
```

Keys: `space` play/pause, `n`/`p` next/prev, `↑`/`↓` select, `enter` jump, `←`/`→` seek,
`+`/`-` volume, `q` quit.

## How it works

soundcli talks to your browser's SoundCloud tab over the OS media bus (MPRIS / D-Bus),
reading now-playing metadata and sending transport commands. Audio and ads stay in the
browser, so your account and private sets work normally. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the design and [INSTALL.md](INSTALL.md) for
configuration and queue metadata.

## License

MIT — see [LICENSE](LICENSE). Independent project, not affiliated with SoundCloud; use it
in accordance with [SoundCloud's Terms of Use](https://soundcloud.com/terms-of-use).
