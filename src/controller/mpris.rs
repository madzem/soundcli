//! Controls the user's SoundCloud browser tab over MPRIS / D-Bus.

use std::time::{Duration, Instant};

use mpris::{PlaybackStatus, Player, PlayerFinder};

use super::Controller;
use crate::model::{PlayerState, Track};

const VOL_POLL: Duration = Duration::from_millis(500);
const HOP_GAP: Duration = Duration::from_millis(120);

pub struct MprisController {
    finder: PlayerFinder,
    player: Option<Player>,
    playlist_url: Option<String>,
    player_filter: Option<String>,
    vol_node: Option<u32>,
    last_volume: Option<f64>,
    last_vol_at: Option<Instant>,
    playlist_tracks: Vec<Track>,
    /// Index of the now-playing track within `playlist_tracks`; `None` while off-list.
    matched_idx: Option<usize>,
    /// Remaining queue hops for an in-progress jump (positive = `next`, negative = `previous`).
    jump_remaining: i32,
    last_hop_at: Option<Instant>,
    queue_note: Option<String>,
}

impl MprisController {
    pub fn new(
        playlist_url: Option<String>,
        player_filter: Option<String>,
        client_id: Option<String>,
        oauth_token: Option<String>,
    ) -> anyhow::Result<Self> {
        let finder = PlayerFinder::new()?;
        let playlist_tracks =
            fetch_queue(playlist_url.as_deref(), client_id, oauth_token.as_deref());
        let queue_note = if playlist_url.is_some() && playlist_tracks.is_empty() {
            Some("private set? add oauth_token to ~/.config/soundcli/config.yaml".to_string())
        } else {
            None
        };
        Ok(MprisController {
            finder,
            player: None,
            playlist_url,
            player_filter,
            vol_node: None,
            last_volume: None,
            last_vol_at: None,
            playlist_tracks,
            matched_idx: None,
            jump_remaining: 0,
            last_hop_at: None,
            queue_note,
        })
    }

    /// Advance an in-progress jump by at most one hop per `HOP_GAP`, so the UI never blocks.
    fn advance_jump(&mut self) {
        if self.jump_remaining == 0 {
            return;
        }
        let due = self.last_hop_at.is_none_or(|t| t.elapsed() >= HOP_GAP);
        if !due {
            return;
        }
        let Some(p) = &self.player else {
            self.jump_remaining = 0;
            return;
        };
        if self.jump_remaining > 0 {
            let _ = p.next();
            self.jump_remaining -= 1;
        } else {
            let _ = p.previous();
            self.jump_remaining += 1;
        }
        self.last_hop_at = Some(Instant::now());
    }

    fn vol_node(&mut self) -> Option<u32> {
        if let Some(n) = self.vol_node {
            if crate::volume::get(n).is_some() {
                return Some(n);
            }
        }
        self.vol_node = crate::volume::find_node();
        self.vol_node
    }

    /// Point `self.player` at a SoundCloud player: a `soundcloud.com` url first, else a
    /// playing browser with a title, else any browser with a title.
    fn ensure_player(&mut self) {
        if let Some(p) = &self.player {
            if !p.is_running() {
                self.player = None;
            }
        }
        if self.player.is_some() {
            return;
        }
        let Ok(players) = self.finder.find_all() else {
            return;
        };

        let mut soundcloud = None;
        let mut browser_playing = None;
        let mut browser_any = None;

        for p in players {
            if let Some(f) = &self.player_filter {
                let f = f.to_lowercase();
                if !p.bus_name().to_lowercase().contains(&f)
                    && !p.identity().to_lowercase().contains(&f)
                {
                    continue;
                }
            }
            let meta = p.get_metadata().ok();
            let url_sc = meta
                .as_ref()
                .and_then(|m| m.url())
                .map(|u| u.contains("soundcloud.com"))
                .unwrap_or(false);
            let has_title = meta
                .as_ref()
                .and_then(|m| m.title())
                .map(|t| !t.is_empty())
                .unwrap_or(false);
            let browser = is_browser(p.identity());
            let playing = matches!(p.get_playback_status(), Ok(PlaybackStatus::Playing));

            if url_sc {
                soundcloud = Some(p);
                break;
            }
            if browser && has_title {
                if playing {
                    if browser_playing.is_none() {
                        browser_playing = Some(p);
                    }
                } else if browser_any.is_none() {
                    browser_any = Some(p);
                }
            }
        }
        self.player = soundcloud.or(browser_playing).or(browser_any);
    }
}

fn fetch_queue(url: Option<&str>, client_id: Option<String>, oauth: Option<&str>) -> Vec<Track> {
    let Some(url) = url else { return Vec::new() };
    let Some(cid) = client_id.or_else(crate::soundcloud::fetch_client_id) else {
        eprintln!(
            "soundcli: couldn't get a SoundCloud client_id; queue will show current track only."
        );
        return Vec::new();
    };
    eprintln!("soundcli: fetching playlist tracklist…");
    crate::soundcloud::fetch_playlist(url, &cid, oauth).unwrap_or_default()
}

/// Locate the now-playing title in `tracks`: exact normalized match first, then a
/// length-guarded substring fallback.
fn find_current(tracks: &[Track], current: &str) -> Option<usize> {
    let norm = |s: &str| s.trim().to_lowercase();
    let c = norm(current);
    if c.len() < 2 {
        return None;
    }
    if let Some(i) = tracks.iter().position(|t| norm(&t.title) == c) {
        return Some(i);
    }
    if c.len() >= 4 {
        return tracks.iter().position(|t| {
            let tt = norm(&t.title);
            tt.len() >= 4 && (tt.contains(&c) || c.contains(&tt))
        });
    }
    None
}

fn is_browser(identity: &str) -> bool {
    let id = identity.to_lowercase();
    [
        "chrome",
        "chromium",
        "firefox",
        "mozilla",
        "brave",
        "edge",
        "vivaldi",
        "opera",
        "librewolf",
        "zen",
    ]
    .iter()
    .any(|b| id.contains(b))
}

/// Whether a SoundCloud tab (a player exposing a `soundcloud.com` url) is already open.
pub fn soundcloud_tab_open() -> bool {
    let Ok(finder) = PlayerFinder::new() else {
        return false;
    };
    let Ok(players) = finder.find_all() else {
        return false;
    };
    players.iter().any(|p| {
        p.get_metadata()
            .ok()
            .and_then(|m| m.url().map(|u| u.contains("soundcloud.com")))
            .unwrap_or(false)
    })
}

/// Print every MPRIS player on the session bus (`--list-players`).
pub fn list_players() -> anyhow::Result<()> {
    let finder = PlayerFinder::new()?;
    let players = finder.find_all()?;
    if players.is_empty() {
        println!("No MPRIS players found on the session bus.");
        return Ok(());
    }
    for p in &players {
        let status = p
            .get_playback_status()
            .map(|s| format!("{s:?}"))
            .unwrap_or_else(|_| "?".into());
        let (title, artist, url) = p
            .get_metadata()
            .map(|m| {
                (
                    m.title().unwrap_or("").to_string(),
                    m.artists()
                        .and_then(|a| a.first().map(|s| s.to_string()))
                        .unwrap_or_default(),
                    m.url().unwrap_or("").to_string(),
                )
            })
            .unwrap_or_default();
        let tag = if is_browser(p.identity()) {
            " (browser)"
        } else {
            ""
        };
        println!("• {} [{}]{}", p.identity(), status, tag);
        println!("    bus: {}", p.bus_name());
        println!("    now: {title} — {artist}");
        println!("    url: {}", if url.is_empty() { "(none)" } else { &url });
    }
    Ok(())
}

impl Controller for MprisController {
    fn refresh(&mut self) -> PlayerState {
        self.ensure_player();
        self.advance_jump();

        let due = self.last_vol_at.is_none_or(|t| t.elapsed() >= VOL_POLL);
        if due {
            if let Some(v) = self.vol_node().and_then(crate::volume::get) {
                self.last_volume = Some(v);
            }
            self.last_vol_at = Some(Instant::now());
        }
        let mut st = base_state(&self.playlist_url);
        st.volume = self.last_volume;
        st.queue_note = self.queue_note.clone();

        let Some(p) = &self.player else {
            st.source = "mpris · waiting for SoundCloud tab".into();
            return st;
        };

        st.connected = true;
        st.source = format!("mpris · {}", p.identity());

        if let Ok(meta) = p.get_metadata() {
            let title = meta.title().unwrap_or("(unknown track)").to_string();
            let artist = meta
                .artists()
                .and_then(|a| a.first().map(|s| s.to_string()))
                .unwrap_or_default();
            let dur = meta.length().map(|d| d.as_secs()).unwrap_or(0);

            let matched = if self.playlist_tracks.is_empty() {
                None
            } else {
                find_current(&self.playlist_tracks, &title)
            };
            self.matched_idx = matched;
            match matched {
                Some(idx) => {
                    st.tracks = self.playlist_tracks.clone();
                    st.current_idx = idx;
                    st.queue_partial = false;
                }
                None => {
                    st.tracks = vec![Track { title, artist, dur }];
                    st.current_idx = 0;
                    st.queue_partial = true;
                }
            }
        }
        st.playing = matches!(p.get_playback_status(), Ok(PlaybackStatus::Playing));
        st.elapsed = p.get_position().map(|d| d.as_secs()).unwrap_or(0);
        st
    }

    fn toggle(&mut self) {
        if let Some(p) = &self.player {
            let _ = p.play_pause();
        }
    }

    fn next(&mut self) {
        if let Some(p) = &self.player {
            let _ = p.next();
        }
    }

    fn prev(&mut self) {
        if let Some(p) = &self.player {
            let _ = p.previous();
        }
    }

    fn set_volume(&mut self, v: f64) {
        // Browsers ignore MPRIS volume; drive the PipeWire node. Cache it so repeated
        // +/- steps accumulate between the throttled `wpctl` reads in `refresh`.
        let v = v.clamp(0.0, 1.0);
        if let Some(n) = self.vol_node() {
            crate::volume::set(n, v);
            self.last_volume = Some(v);
        }
    }

    fn seek_to(&mut self, secs: u64) {
        if let Some(p) = &self.player {
            if let Ok(meta) = p.get_metadata() {
                if let Some(tid) = meta.track_id() {
                    let _ = p.set_position(tid, &std::time::Duration::from_secs(secs));
                }
            }
        }
    }

    fn play_index(&mut self, idx: usize) {
        // MPRIS has no absolute seek to a track; hop next/prev by the delta, one hop per
        // tick via `advance_jump`. Assumes in-order play (browser queue matches the API).
        let Some(cur) = self.matched_idx else { return };
        if idx == cur {
            if let Some(p) = &self.player {
                let _ = p.play_pause();
            }
            return;
        }
        self.jump_remaining = idx as i32 - cur as i32;
        self.last_hop_at = None;
    }

    fn stop(&mut self) {
        if let Some(p) = &self.player {
            if p.stop().is_err() {
                let _ = p.pause();
            }
        }
    }
}

fn base_state(url: &Option<String>) -> PlayerState {
    let name = url
        .as_deref()
        .map(parse_set)
        .unwrap_or_else(|| "—".to_string());
    PlayerState {
        playlist_name: name,
        tracks: Vec::new(),
        current_idx: 0,
        elapsed: 0,
        playing: false,
        volume: None,
        queue_partial: true,
        source: "mpris".into(),
        connected: false,
        queue_note: None,
    }
}

/// Derive a display name from a set URL, stripping the `::owner:id` personalized suffix.
fn parse_set(url: &str) -> String {
    let after = url.split("soundcloud.com/").nth(1).unwrap_or("");
    let path = after.split(['?', '#']).next().unwrap_or("");
    let raw = path.split('/').rfind(|s| !s.is_empty()).unwrap_or("");
    let raw = raw.split("::").next().unwrap_or(raw);
    if raw.is_empty() {
        "—".to_string()
    } else {
        raw.replace('-', " ")
    }
}

#[cfg(test)]
mod tests {
    use super::{find_current, parse_set};
    use crate::model::Track;

    #[test]
    fn parse_set_extracts_slug_from_set_url() {
        assert_eq!(
            parse_set("https://soundcloud.com/neoncoast/sets/night-drive?x=1"),
            "night drive"
        );
    }

    #[test]
    fn parse_set_strips_personalized_owner_suffix() {
        assert_eq!(
            parse_set("https://soundcloud.com/user/sets/chill::neoncoast:123456"),
            "chill"
        );
    }

    #[test]
    fn parse_set_returns_dash_for_bare_host() {
        assert_eq!(parse_set("https://soundcloud.com/"), "—");
    }

    fn track(title: &str) -> Track {
        Track::new(title, "Artist", 180)
    }

    #[test]
    fn find_current_prefers_exact_match_over_substring() {
        let tracks = [track("Intro (Reprise)"), track("Intro")];
        assert_eq!(find_current(&tracks, "Intro"), Some(1));
    }

    #[test]
    fn find_current_falls_back_to_substring() {
        let tracks = [track("Midnight City Lights (Extended Mix)")];
        assert_eq!(find_current(&tracks, "Midnight City Lights"), Some(0));
    }

    #[test]
    fn find_current_ignores_too_short_titles() {
        let tracks = [track("OK")];
        assert_eq!(find_current(&tracks, "x"), None);
    }
}
