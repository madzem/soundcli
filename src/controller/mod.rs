//! A `Controller` is the bridge between the TUI and whatever is actually playing audio.
//!
//! soundcli never streams audio itself. The real playback happens in your browser's
//! SoundCloud tab; `MprisController` drives it over the OS media bus (MPRIS / D-Bus).
//! `DemoController` fakes a player with sample data so the UI runs without a browser.

mod demo;
mod mpris;

pub use demo::DemoController;
pub use mpris::{list_players, soundcloud_tab_open, MprisController};

use crate::model::PlayerState;

pub trait Controller {
    /// Pull the latest state to render. Called every frame.
    fn refresh(&mut self) -> PlayerState;
    fn toggle(&mut self);
    fn next(&mut self);
    fn prev(&mut self);
    /// Absolute volume in 0.0..=1.0.
    fn set_volume(&mut self, v: f64);
    /// Jump to `secs` into the current track.
    fn seek_to(&mut self, secs: u64);
    /// Play the track at `idx` in the queue (no-op where the queue isn't controllable).
    fn play_index(&mut self, idx: usize);
    /// Stop playback (called on quit).
    fn stop(&mut self);
}
