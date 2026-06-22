//! Platform-agnostic playback state. A `Controller` produces this; the UI renders it.

#[derive(Clone, Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub dur: u64, // seconds
}

impl Track {
    pub fn new(title: &str, artist: &str, dur: u64) -> Self {
        Track {
            title: title.into(),
            artist: artist.into(),
            dur,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub playlist_name: String,
    pub tracks: Vec<Track>,
    pub current_idx: usize,
    pub elapsed: u64,
    pub playing: bool,
    pub volume: Option<f64>, // 0.0..=1.0; None when not controllable
    /// True when only the current track is known (no full queue).
    pub queue_partial: bool,
    pub source: String,
    pub connected: bool,
    /// Message shown in place of an empty queue (e.g. a private set needs a token).
    pub queue_note: Option<String>,
}

impl PlayerState {
    pub fn current(&self) -> Option<&Track> {
        self.tracks.get(self.current_idx)
    }
}
