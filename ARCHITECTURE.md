# soundcli — Architecture

A terminal UI for **controlling SoundCloud playback** from your shell. Open source.

## What it is (and isn't)

soundcli is a **remote control + now-playing dashboard**, not a music streamer.

- It does **not** stream audio, decode tracks, or strip ads.
- Your real, logged-in **soundcloud.com browser tab** does the playback: full tracks,
  ads served normally, your personalized/private sets work via your own session.
- soundcli talks to that tab over the **OS media bus (MPRIS / D-Bus on Linux)** — the
  same channel media keys and `playerctl` use — and renders a rich Ratatui interface.

### Why this design

SoundCloud's API Terms of Use prohibit building an "alternative digital content service
that aggregates and streams User Content," forbid stripping ads, and forbid caching/
downloading. There is also no public ad-serving endpoint — free-tier ads exist only inside
SoundCloud's own apps. A remote control over the official web app sidesteps all of that:
the official client streams and serves ads; soundcli only sends transport commands and
displays state. (Honest caveat: this is not a SoundCloud-sanctioned remote API like Spotify
Connect — but ads genuinely play and no content is streamed or cached by soundcli.)

## Components

```
┌────────────────────┐   MPRIS / D-Bus    ┌──────────────────────────────┐
│  soundcli (Rust)   │ ◀────────────────▶ │  Browser (Chrome/Firefox)    │
│  Ratatui TUI       │  read metadata,    │  soundcloud.com tab (your     │
│  - now playing     │  position, status  │  logged-in session)          │
│  - progress/seek   │                    │  - plays full tracks + ads    │
│  - controls        │  send Play/Pause/  │  - publishes MediaSession →   │
│  - queue view      │  Next/Prev/Seek/   │    org.mpris.MediaPlayer2.*   │
│  --playlist <url>  │  Volume            │                              │
└─────────┬──────────┘                    └──────────────────────────────┘
          │ xdg-open <url>  (load a track/set/playlist into the tab)
          └──────────────────────────────────────────────────────────────▶
```

## MPRIS interface used

Linux browsers expose `org.mpris.MediaPlayer2.<instance>` over D-Bus when a tab plays media
with MediaSession metadata (SoundCloud's web app sets this).

**Read** (`org.mpris.MediaPlayer2.Player` properties):
- `PlaybackStatus` — Playing / Paused / Stopped
- `Metadata` — `xesam:title`, `xesam:artist`, `mpris:artUrl`, `mpris:length`, `xesam:url`
- `Position` — current playback position (µs)
- `Volume`

**Send** (methods / property writes):
- `Play`, `Pause`, `PlayPause`, `Next`, `Previous`, `Stop`
- `Seek(offset_us)`, `SetPosition(track_id, pos_us)`
- set `Volume` (browser support varies)

**Discovery:** enumerate D-Bus names matching `org.mpris.MediaPlayer2.*`, pick the one whose
`Identity`/`Metadata` URL is a soundcloud.com track (filter out Spotify/VLC/etc.).

## `--playlist <url>` flow

```
soundcli --playlist "https://soundcloud.com/discover/sets/personalized-tracks::madzem:..."
```
1. Validate it's a soundcloud.com URL.
2. `xdg-open <url>` → opens/focuses the SoundCloud tab on that set (uses your session, so
   personalized/private sets resolve).
3. Poll D-Bus until a soundcloud MPRIS player appears.
4. Auto-issue `Play` (MediaSession autoplay is often blocked → fall back to prompting the
   user to press play once in the tab).
5. Hand off to the live TUI: render now-playing + controls.

## TUI (Ratatui) — screens

- **Now Playing**: artwork (sixel/kitty image protocol if terminal supports, else ASCII),
  title, artist, progress bar with elapsed/total, play state.
- **Controls**: `space` play/pause, `n`/`p` next/prev, `←/→` seek, `+/-` volume, `q` quit.
- **Queue/Up-next**: from `getSounds()`-equivalent metadata where available (MPRIS exposes
  current track only; full queue needs the optional bridge below).
- **Status line**: shows "Ad playing…" when metadata indicates an ad, connection state.

## Limitations (honest)

- **No in-terminal catalog search/browse.** MPRIS exposes transport + current-track metadata
  only. You search/browse in the browser; you *control + watch* from the terminal.
- **A browser must be running** (can be minimized/backgrounded). Audio + ads come from it.
- **Volume/seek fidelity** depends on the browser's MPRIS implementation.
- **Linux-first.** MPRIS is Linux/D-Bus. macOS would need a `MediaRemote`/AppleScript layer;
  Windows would need SMTC (System Media Transport Controls). Cross-platform = later milestone.

## Optional later: browser-extension bridge

To add in-terminal search/browse/queue, a companion browser extension on soundcloud.com can
expose a local WebSocket speaking a small JSON protocol (search, load, full queue, full
state) while the real web app still streams + serves ads. Architect the control layer behind
a `Controller` trait so `MprisController` ships first and `BridgeController` slots in later.

## Crate plan

- `ratatui` + `crossterm` — TUI
- `mpris` (or `zbus` directly) — D-Bus / MPRIS client
- `tokio` — async event loop (D-Bus polling + input)
- `clap` — CLI args (`--playlist`, etc.)
- image protocol: `ratatui-image` or `viuer` for artwork
- `anyhow` / `thiserror` — errors

## Milestones

1. **M0 — spike:** detect a soundcloud MPRIS player, print now-playing, send play/pause.
2. **M1 — MVP TUI:** Now-Playing screen + transport controls + `--playlist` open flow.
3. **M2 — polish:** artwork rendering, seek/volume, ad-state display, config, packaging.
4. **M3 — distribution:** `cargo install`, prebuilt release binaries, README + ToS disclaimer.
5. **M4 — (optional) bridge:** extension + WebSocket for in-terminal search/browse.
