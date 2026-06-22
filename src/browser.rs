//! Opening the SoundCloud tab, in order of preference: the user's own chromium-family
//! browser with the autoplay flag, an opt-in dedicated chromium profile, or `xdg-open`.
//! The autoplay flag only applies when chrome isn't already running.

use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::config::{self, Config};

/// Chromium-family binaries we can pass the autoplay flag to, in order of preference.
const CHROME_BINS: &[&str] = &[
    "chromium",
    "chromium-browser",
    "google-chrome",
    "google-chrome-stable",
    "brave-browser",
    "brave",
];

const AUTOPLAY_FLAG: &str = "--autoplay-policy=no-user-gesture-required";
// Chrome prints only FATAL, so its GCM/push ERROR spam never reaches the terminal.
const QUIET_FLAG: &str = "--log-level=3";

/// How the tab ended up open — so the caller can print the right hint.
pub enum Opened {
    /// User's own browser + profile, with the autoplay flag.
    DirectChromium,
    /// Dedicated soundcli profile (opt-in), with the autoplay flag.
    Managed,
    /// Plain hand-off to the default browser; no autoplay.
    DefaultBrowser,
}

/// Locate a chromium-family binary, honouring an explicit `browser:` override.
fn find_chrome(cfg: &Config) -> Option<String> {
    if let Some(b) = &cfg.browser {
        let lb = b.to_lowercase();
        if CHROME_BINS.iter().any(|c| lb.contains(c))
            || lb.contains("chrom")
            || lb.contains("brave")
        {
            return which(b);
        }
        return None; // firefox etc. -> not autoplay-capable
    }
    CHROME_BINS.iter().find_map(|b| which(b))
}

fn which(bin: &str) -> Option<String> {
    let out = Command::new("which").arg(bin).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!path.is_empty()).then_some(path)
}

fn silent(cmd: &mut Command) -> std::io::Result<std::process::Child> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // New process group: a launched browser can't grab the terminal and outlive us
        // spewing to it.
        .process_group(0)
        .spawn()
}

/// Open `url`, returning how it was launched.
pub fn open(url: &str, cfg: &Config) -> Opened {
    let autoplay = cfg.autoplay.unwrap_or(true);
    let chrome = find_chrome(cfg);

    if cfg.managed == Some(true) {
        if let Some(bin) = &chrome {
            let profile = config::profile_dir();
            let _ = std::fs::create_dir_all(&profile);
            let ok = silent(
                Command::new(bin)
                    .arg(format!("--user-data-dir={}", profile.display()))
                    .arg(AUTOPLAY_FLAG)
                    .arg(QUIET_FLAG)
                    .arg("--password-store=basic")
                    .arg("--no-first-run")
                    .arg("--no-default-browser-check")
                    .arg(url),
            )
            .is_ok();
            if ok {
                return Opened::Managed;
            }
        }
    }

    if autoplay {
        if let Some(bin) = &chrome {
            if silent(
                Command::new(bin)
                    .arg(AUTOPLAY_FLAG)
                    .arg(QUIET_FLAG)
                    .arg(url),
            )
            .is_ok()
            {
                return Opened::DirectChromium;
            }
        }
    }

    if let Some(b) = &cfg.browser {
        if silent(Command::new(b).arg(url)).is_ok() {
            return Opened::DefaultBrowser;
        }
    }
    let _ = silent(Command::new("xdg-open").arg(url));
    Opened::DefaultBrowser
}
