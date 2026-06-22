//! SoundCloud metadata fetch — *only* for the queue ("what's next").
//!
//! Playback and ads stay in the browser (MPRIS). This module reads public playlist
//! metadata via SoundCloud's internal api-v2, using the same web `client_id` the site
//! itself uses (auto-extracted, or supplied via `SOUNDCLOUD_CLIENT_ID`). No streaming,
//! no downloads — just titles/artists/durations to populate the queue list.
//!
//! Private/personalized sets need the user's token via `SOUNDCLOUD_OAUTH_TOKEN`.

use std::time::Duration;

use crate::model::Track;

const UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124 Safari/537.36";

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(10))
        .build()
}

fn get(agent: &ureq::Agent, url: &str, oauth: Option<&str>) -> Option<String> {
    let mut req = agent.get(url).set("User-Agent", UA);
    if let Some(t) = oauth {
        req = req.set("Authorization", &format!("OAuth {t}"));
    }
    req.call().ok()?.into_string().ok()
}

fn dbg_on() -> bool {
    std::env::var_os("SOUNDCLI_DEBUG").is_some()
}

macro_rules! dbg_eprintln {
    ($($a:tt)*) => { if dbg_on() { eprintln!($($a)*); } };
}

/// Extract the public web `client_id` the soundcloud.com player uses.
pub fn fetch_client_id() -> Option<String> {
    let agent = agent();
    let Some(html) = get(&agent, "https://soundcloud.com/", None) else {
        dbg_eprintln!("soundcli[debug]: homepage GET failed (network/proxy/TLS?)");
        return None;
    };
    dbg_eprintln!("soundcli[debug]: homepage {} bytes", html.len());

    // Collect "https://a-v2.sndcdn.com/assets/*.js" bundle URLs in page order.
    let mut urls = Vec::new();
    let mut rest = html.as_str();
    while let Some(p) = rest.find("https://a-v2.sndcdn.com/assets/") {
        let tail = &rest[p..];
        match tail.find(".js") {
            Some(end) => {
                urls.push(tail[..end + 3].to_string());
                rest = &tail[end + 3..];
            }
            None => break,
        }
    }
    dbg_eprintln!("soundcli[debug]: {} asset bundle(s) found", urls.len());
    // The client_id usually lives in one of the later bundles.
    for u in urls.iter().rev() {
        if let Some(js) = get(&agent, u, None) {
            for key in ["client_id:\"", "client_id=\"", "client_id="] {
                if let Some(id) = extract_after(&js, key) {
                    return Some(id);
                }
            }
        }
    }
    dbg_eprintln!("soundcli[debug]: scanned bundles, no client_id matched");
    None
}

fn extract_after(s: &str, key: &str) -> Option<String> {
    let i = s.find(key)? + key.len();
    let id: String = s[i..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    (id.len() >= 16).then_some(id)
}

/// Resolve a set/playlist URL to its ordered tracklist.
pub fn fetch_playlist(url: &str, client_id: &str, oauth: Option<&str>) -> Option<Vec<Track>> {
    let agent = agent();
    let resolve = format!(
        "https://api-v2.soundcloud.com/resolve?url={}&client_id={}",
        urlencode(url),
        client_id
    );
    let body = get(&agent, &resolve, oauth)?;
    let v: serde_json::Value = serde_json::from_str(&body).ok()?;

    let tracks_json = v.get("tracks")?.as_array()?;

    // Playlists return some hydrated tracks and many stubs ({id, kind}). Keep order;
    // batch-fetch the stubs by id.
    let mut order: Vec<i64> = Vec::new();
    let mut by_id: std::collections::HashMap<i64, Track> = std::collections::HashMap::new();
    let mut missing: Vec<i64> = Vec::new();
    for t in tracks_json {
        let Some(id) = t.get("id").and_then(|x| x.as_i64()) else {
            continue;
        };
        order.push(id);
        if t.get("title").and_then(|x| x.as_str()).is_some() {
            by_id.insert(id, track_from_json(t));
        } else {
            missing.push(id);
        }
    }
    for chunk in missing.chunks(50) {
        let ids = chunk
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let u = format!(
            "https://api-v2.soundcloud.com/tracks?ids={}&client_id={}",
            ids, client_id
        );
        if let Some(b) = get(&agent, &u, oauth) {
            if let Ok(arr) = serde_json::from_str::<serde_json::Value>(&b) {
                if let Some(a) = arr.as_array() {
                    for t in a {
                        if let Some(id) = t.get("id").and_then(|x| x.as_i64()) {
                            by_id.insert(id, track_from_json(t));
                        }
                    }
                }
            }
        }
    }
    let tracks: Vec<Track> = order
        .iter()
        .filter_map(|id| by_id.get(id).cloned())
        .collect();
    if tracks.len() != order.len() {
        dbg_eprintln!(
            "soundcli[debug]: set lists {} tracks but only {} hydrated ({} dropped — \
             these shift the queue index; report this)",
            order.len(),
            tracks.len(),
            order.len() - tracks.len()
        );
    } else {
        dbg_eprintln!(
            "soundcli[debug]: {} tracks, all hydrated (count matches set)",
            tracks.len()
        );
    }
    (!tracks.is_empty()).then_some(tracks)
}

fn track_from_json(t: &serde_json::Value) -> Track {
    let title = t
        .get("title")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let artist = t
        .get("publisher_metadata")
        .and_then(|p| p.get("artist"))
        .and_then(|x| x.as_str())
        .or_else(|| {
            t.get("user")
                .and_then(|u| u.get("username"))
                .and_then(|x| x.as_str())
        })
        .unwrap_or("")
        .to_string();
    let dur = (t.get("duration").and_then(|x| x.as_i64()).unwrap_or(0) / 1000).max(0) as u64;
    Track { title, artist, dur }
}

fn urlencode(s: &str) -> String {
    let mut o = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                o.push(b as char)
            }
            _ => o.push_str(&format!("%{b:02X}")),
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::{extract_after, urlencode};

    #[test]
    fn urlencode_passes_unreserved_chars_through() {
        assert_eq!(urlencode("aZ09-_.~"), "aZ09-_.~");
    }

    #[test]
    fn urlencode_percent_encodes_reserved_chars() {
        assert_eq!(urlencode("a b/c?d"), "a%20b%2Fc%3Fd");
    }

    #[test]
    fn urlencode_encodes_multibyte_utf8_per_byte() {
        assert_eq!(urlencode("é"), "%C3%A9");
    }

    #[test]
    fn extract_after_reads_value_up_to_non_alnum() {
        let js = r#"...,client_id:"abcDEF0123456789xyz",foo..."#;
        assert_eq!(
            extract_after(js, "client_id:\""),
            Some("abcDEF0123456789xyz".to_string())
        );
    }

    #[test]
    fn extract_after_rejects_too_short_value() {
        assert_eq!(extract_after("client_id=\"short\"", "client_id=\""), None);
    }

    #[test]
    fn extract_after_returns_none_when_key_absent() {
        assert_eq!(extract_after("no key here", "client_id="), None);
    }
}
