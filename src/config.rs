//! Persistent settings at `~/.config/soundcli/config.yaml`.
//!
//! Tiny hand-rolled flat-YAML reader/writer (no serde_yaml dep). Only `key: value`
//! scalar lines are understood — enough for our handful of options. The managed
//! chromium's login lives in the profile dir (`chrome-profile/`), not here; this file
//! only holds settings and the optional api-v2 credentials for the queue fetch.

use std::path::PathBuf;

#[derive(Default, Debug, Clone)]
pub struct Config {
    /// Override the browser binary (e.g. "chromium", "google-chrome", "firefox").
    pub browser: Option<String>,
    /// Opt in to a dedicated chromium profile under `~/.config/soundcli` (default off).
    pub managed: Option<bool>,
    /// Launch chromium with the autoplay flag so playback starts on its own. Default on;
    /// set `autoplay: false` to just hand off to the default browser.
    pub autoplay: Option<bool>,
    /// SoundCloud web `client_id` for the api-v2 queue fetch (else auto-extracted).
    pub client_id: Option<String>,
    /// User OAuth token — only needed to resolve private/personalized sets.
    pub oauth_token: Option<String>,
}

/// `~/.config/soundcli` (honours `$XDG_CONFIG_HOME`).
pub fn dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("soundcli")
}

pub fn path() -> PathBuf {
    dir().join("config.yaml")
}

/// Dedicated chromium profile dir — the user signs in here once and it persists.
pub fn profile_dir() -> PathBuf {
    dir().join("chrome-profile")
}

pub fn load() -> Config {
    match std::fs::read_to_string(path()) {
        Ok(text) => parse(&text),
        Err(_) => Config::default(),
    }
}

/// Parse the flat `key: value` config text. Unknown keys, blank lines and `#` comments
/// are ignored. Split out from [`load`] so it can be unit-tested without touching disk.
fn parse(text: &str) -> Config {
    let mut cfg = Config::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim();
        let val = v.trim().trim_matches(|c| c == '"' || c == '\'').trim();
        if val.is_empty() {
            continue;
        }
        match key {
            "browser" => cfg.browser = Some(val.to_string()),
            "managed" => cfg.managed = Some(matches!(val, "true" | "yes" | "1")),
            "autoplay" => cfg.autoplay = Some(matches!(val, "true" | "yes" | "1")),
            "client_id" => cfg.client_id = Some(val.to_string()),
            "oauth_token" => cfg.oauth_token = Some(val.to_string()),
            _ => {}
        }
    }
    cfg
}

/// Write the config back, creating the directory if needed. Best-effort.
#[allow(dead_code)]
pub fn save(cfg: &Config) {
    let _ = std::fs::create_dir_all(dir());
    let mut out = String::from("# soundcli config\n");
    if let Some(b) = &cfg.browser {
        out.push_str(&format!("browser: {b}\n"));
    }
    if let Some(m) = cfg.managed {
        out.push_str(&format!("managed: {m}\n"));
    }
    if let Some(a) = cfg.autoplay {
        out.push_str(&format!("autoplay: {a}\n"));
    }
    if let Some(c) = &cfg.client_id {
        out.push_str(&format!("client_id: {c}\n"));
    }
    if let Some(t) = &cfg.oauth_token {
        out.push_str(&format!("oauth_token: {t}\n"));
    }
    let _ = std::fs::write(path(), out);
}

/// On first run, drop a commented template so the user can see what's configurable —
/// notably where to paste an `oauth_token` for private-set queue metadata.
pub fn init_template() {
    if path().exists() {
        return;
    }
    let _ = std::fs::create_dir_all(dir());
    let body = "\
# soundcli config — ~/.config/soundcli/config.yaml
#
# browser:      override the browser binary (chromium | google-chrome | firefox | ...)
# autoplay:     true (default) launches your chromium-family browser with an autoplay
#               flag so playback starts on its own; false hands off to the default
#               browser with no autoplay. Autoplay only applies if chrome isn't already
#               running (the flag is read at process start).
# managed:      true to opt in to a DEDICATED chromium profile under ~/.config/soundcli
#               (sign in once there). Default off — soundcli uses your real browser
#               profile so your existing login (and private sets) just work.
# client_id:    SoundCloud web client_id for the queue fetch (auto-extracted if omitted)
# oauth_token:  your OAuth token — only needed to resolve PRIVATE/personalized sets.
#               Grab it from a logged-in browser: DevTools → Application → Cookies →
#               soundcloud.com → `oauth_token`, then uncomment the line below.
#
# oauth_token: 2-XXXXXX-...
";
    let _ = std::fs::write(path(), body);
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parse_reads_known_scalar_keys() {
        let cfg = parse("browser: firefox\nclient_id: abc123\n");
        assert_eq!(cfg.browser.as_deref(), Some("firefox"));
        assert_eq!(cfg.client_id.as_deref(), Some("abc123"));
    }

    #[test]
    fn parse_strips_quotes_and_whitespace() {
        let cfg = parse("oauth_token:   \"2-secret\"  \n");
        assert_eq!(cfg.oauth_token.as_deref(), Some("2-secret"));
    }

    #[test]
    fn parse_reads_bool_flags() {
        let cfg = parse("managed: yes\nautoplay: false\n");
        assert_eq!(cfg.managed, Some(true));
        assert_eq!(cfg.autoplay, Some(false));
    }

    #[test]
    fn parse_ignores_comments_blanks_and_unknown_keys() {
        let cfg = parse("# comment\n\nbogus: 1\nbrowser: brave\n");
        assert_eq!(cfg.browser.as_deref(), Some("brave"));
        assert_eq!(cfg.client_id, None);
    }
}
