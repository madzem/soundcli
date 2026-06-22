//! soundcli — a terminal remote + now-playing dashboard for SoundCloud.
//!
//! It does NOT stream audio. Your browser's SoundCloud tab plays the music (ads and all);
//! soundcli controls it over the OS media bus (MPRIS / D-Bus) and renders a TUI.

mod app;
mod browser;
mod config;
mod controller;
mod model;
mod soundcloud;
mod theme;
mod ui;
mod volume;

use anyhow::Result;
use clap::Parser;

use controller::{list_players, soundcloud_tab_open, Controller, DemoController, MprisController};

#[derive(Parser)]
#[command(
    name = "soundcli",
    version,
    about = "Terminal remote + now-playing dashboard for SoundCloud (controls your browser over MPRIS)"
)]
struct Cli {
    /// SoundCloud track/set/playlist URL to open in your browser and control
    #[arg(long, value_name = "URL")]
    playlist: Option<String>,

    /// Run with built-in sample data — no browser needed (great for a first look)
    #[arg(long)]
    demo: bool,

    /// List every MPRIS player on the bus (for diagnosing detection), then exit
    #[arg(long)]
    list_players: bool,

    /// Fetch + print a set's tracklist from the SoundCloud API (debug), then exit
    #[arg(long, value_name = "URL")]
    dump_queue: Option<String>,

    /// Pin a specific player by bus-name/identity substring (e.g. "firefox")
    #[arg(long, value_name = "MATCH")]
    player: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    config::init_template();
    let cfg = config::load();
    // Credentials: environment variable wins, then config.yaml.
    let client_id = std::env::var("SOUNDCLOUD_CLIENT_ID")
        .ok()
        .or_else(|| cfg.client_id.clone());
    let oauth_token = std::env::var("SOUNDCLOUD_OAUTH_TOKEN")
        .ok()
        .or_else(|| cfg.oauth_token.clone());

    if cli.list_players {
        return list_players();
    }

    if let Some(url) = cli.dump_queue {
        let Some(cid) = client_id.clone().or_else(soundcloud::fetch_client_id) else {
            eprintln!("could not obtain a client_id");
            std::process::exit(1);
        };
        eprintln!("client_id = {cid}");
        match soundcloud::fetch_playlist(&url, &cid, oauth_token.as_deref()) {
            Some(tracks) => {
                println!("{} tracks:", tracks.len());
                for (i, t) in tracks.iter().enumerate() {
                    println!(
                        "  {:02}  {}  —  {}  ({}:{:02})",
                        i + 1,
                        t.title,
                        t.artist,
                        t.dur / 60,
                        t.dur % 60
                    );
                }
            }
            None => {
                eprintln!("resolve returned no tracks (private set? needs SOUNDCLOUD_OAUTH_TOKEN)")
            }
        }
        return Ok(());
    }

    if cli.demo {
        return app::run(Box::new(DemoController::new()));
    }

    if let Some(url) = &cli.playlist {
        if !is_soundcloud(url) {
            eprintln!("error: --playlist must be a soundcloud.com URL");
            std::process::exit(2);
        }
        if soundcloud_tab_open() {
            eprintln!("soundcli: SoundCloud already open — press play in that tab to start.");
        } else {
            match browser::open(url, &cfg) {
                browser::Opened::DirectChromium => eprintln!(
                    "soundcli: opened in your browser with autoplay. \
                     (If chrome was already running, autoplay can't apply — press play once.)"
                ),
                browser::Opened::Managed => {
                    eprintln!("soundcli: launched dedicated chromium profile (sign in once; it persists).")
                }
                browser::Opened::DefaultBrowser => {
                    eprintln!("soundcli: opened in your default browser; press play once.")
                }
            }
        }
    }

    let controller: Box<dyn Controller> = match MprisController::new(
        cli.playlist.clone(),
        cli.player.clone(),
        client_id,
        oauth_token,
    ) {
        Ok(c) => Box::new(c),
        Err(e) => {
            eprintln!("warning: no MPRIS/D-Bus session ({e}); starting demo mode instead.");
            Box::new(DemoController::new())
        }
    };
    app::run(controller)
}

/// Whether `url`'s host is `soundcloud.com` or a subdomain, matching the host rather than
/// a substring so look-alikes like `soundcloud.com.evil.example` are rejected.
fn is_soundcloud(url: &str) -> bool {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .rsplit('@')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    host == "soundcloud.com" || host.ends_with(".soundcloud.com")
}

#[cfg(test)]
mod tests {
    use super::is_soundcloud;

    #[test]
    fn accepts_canonical_soundcloud_url() {
        assert!(is_soundcloud(
            "https://soundcloud.com/neoncoast/sets/night-drive"
        ));
    }

    #[test]
    fn accepts_soundcloud_subdomain() {
        assert!(is_soundcloud("https://on.soundcloud.com/abc123"));
    }

    #[test]
    fn rejects_lookalike_host() {
        assert!(!is_soundcloud("https://soundcloud.com.evil.example/track"));
    }

    #[test]
    fn rejects_substring_in_path() {
        assert!(!is_soundcloud("https://evil.example/soundcloud.com/x"));
    }
}
