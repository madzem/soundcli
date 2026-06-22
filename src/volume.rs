//! Per-app volume for the browser's audio stream, via PipeWire's `wpctl`.
//!
//! Browsers ignore MPRIS `SetVolume`, so real volume control means targeting the
//! browser's PipeWire stream node directly.

use std::process::Command;

const BROWSERS: &[&str] = &[
    "chrome",
    "chromium",
    "firefox",
    "brave",
    "edge",
    "vivaldi",
    "opera",
    "librewolf",
    "zen",
    "mozilla",
];

/// Find the PipeWire node id of a browser audio stream from `wpctl status`.
pub fn find_node() -> Option<u32> {
    let out = Command::new("wpctl").arg("status").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut in_streams = false;
    for line in text.lines() {
        let low = line.to_lowercase();
        if low.contains("streams:") {
            in_streams = true;
            continue;
        }
        if !in_streams {
            continue;
        }
        if BROWSERS.iter().any(|b| low.contains(b)) {
            if let Some(id) = leading_id(line) {
                return Some(id);
            }
        }
    }
    None
}

fn leading_id(line: &str) -> Option<u32> {
    let digits: String = line
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Current volume of `node` as 0.0..=1.0 (`wpctl get-volume` -> "Volume: 0.40").
pub fn get(node: u32) -> Option<f64> {
    let out = Command::new("wpctl")
        .args(["get-volume", &node.to_string()])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace().nth(1)?.parse().ok()
}

pub fn set(node: u32, v: f64) {
    let pct = (v.clamp(0.0, 1.0) * 100.0).round() as u32;
    let _ = Command::new("wpctl")
        .args(["set-volume", &node.to_string(), &format!("{pct}%")])
        .status();
}
