//! Offline controller with a fixed sample playlist; no browser or network.

use std::time::Instant;

use super::Controller;
use crate::model::{PlayerState, Track};

pub struct DemoController {
    state: PlayerState,
    last: Instant,
    acc: f64, // sub-second accumulator so playback advances in real time
}

impl DemoController {
    pub fn new() -> Self {
        let tracks = vec![
            Track::new("Midnight City Lights", "Neon Coast", 222),
            Track::new("Sunset Overdrive", "Palm Reader", 184),
            Track::new("Lo-Fi Saturdays", "bedroom.wav", 251),
            Track::new("Drive Slow Homie", "VHS Dreams", 199),
            Track::new("Concrete Jungle Beat", "Eastside Collective", 165),
            Track::new("Analog Hearts", "Tape Deck", 233),
        ];
        let state = PlayerState {
            playlist_name: "Night Drive".into(),
            tracks,
            current_idx: 0,
            elapsed: 37,
            playing: true,
            volume: Some(0.7),
            queue_partial: false,
            source: "demo".into(),
            connected: true,
            queue_note: None,
        };
        DemoController {
            state,
            last: Instant::now(),
            acc: 0.0,
        }
    }

    fn advance_one_second(&mut self) {
        let dur = self.state.tracks[self.state.current_idx].dur;
        if self.state.elapsed + 1 >= dur {
            self.state.current_idx = (self.state.current_idx + 1) % self.state.tracks.len();
            self.state.elapsed = 0;
        } else {
            self.state.elapsed += 1;
        }
    }
}

impl Controller for DemoController {
    fn refresh(&mut self) -> PlayerState {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        if self.state.playing {
            self.acc += dt;
            while self.acc >= 1.0 {
                self.acc -= 1.0;
                self.advance_one_second();
            }
        }
        self.state.clone()
    }

    fn toggle(&mut self) {
        self.state.playing = !self.state.playing;
    }

    fn next(&mut self) {
        self.state.current_idx = (self.state.current_idx + 1) % self.state.tracks.len();
        self.state.elapsed = 0;
        self.acc = 0.0;
    }

    fn prev(&mut self) {
        let n = self.state.tracks.len();
        if self.state.elapsed > 3 {
            self.state.elapsed = 0;
        } else {
            self.state.current_idx = (self.state.current_idx + n - 1) % n;
            self.state.elapsed = 0;
        }
        self.acc = 0.0;
    }

    fn set_volume(&mut self, v: f64) {
        self.state.volume = Some(v.clamp(0.0, 1.0));
    }

    fn seek_to(&mut self, secs: u64) {
        let dur = self.state.tracks[self.state.current_idx].dur;
        self.state.elapsed = secs.min(dur.saturating_sub(1));
        self.acc = 0.0;
    }

    fn play_index(&mut self, idx: usize) {
        if idx < self.state.tracks.len() {
            self.state.current_idx = idx;
            self.state.elapsed = 0;
            self.state.playing = true;
            self.acc = 0.0;
        }
    }

    fn stop(&mut self) {
        self.state.playing = false;
        self.state.elapsed = 0;
    }
}
