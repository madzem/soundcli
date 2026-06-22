//! soundcli color palette.

use ratatui::style::Color;

pub const BG: Color = Color::Rgb(0x1b, 0x20, 0x30); // page background
pub const PANEL: Color = Color::Rgb(0x1d, 0x23, 0x34); // interface box
pub const ORANGE: Color = Color::Rgb(0xff, 0x55, 0x00); // SoundCloud orange / border
pub const ORANGE_LT: Color = Color::Rgb(0xff, 0x7a, 0x33);
pub const ORANGE_HEAD: Color = Color::Rgb(0xff, 0xb2, 0x7a); // progress bar head
pub const ACCENT: Color = Color::Rgb(0xe0, 0x82, 0x3d); // footer key hints

pub const TEXT: Color = Color::Rgb(0xc5, 0xc8, 0xd6);
pub const TEXT_BRIGHT: Color = Color::Rgb(0xf3, 0xec, 0xe2);
pub const PURPLE: Color = Color::Rgb(0xc8, 0xb9, 0xf0); // playlist name
pub const DIM: Color = Color::Rgb(0x6b, 0x71, 0x86);
pub const DIM2: Color = Color::Rgb(0x5f, 0x64, 0x78);
pub const FOOT: Color = Color::Rgb(0x8b, 0x90, 0xa6);

pub const BAR_EMPTY: Color = Color::Rgb(0x3a, 0x40, 0x55); // unplayed progress
pub const TOGGLE_FG: Color = Color::Rgb(0x1a, 0x0d, 0x04); // text on orange (flash)

// Section tints (panel blended with orange) — approximations of the rgba overlays.
pub const HEADER_BG: Color = Color::Rgb(0x2a, 0x23, 0x30); // orange @ ~6%
pub const SELECT_BG: Color = Color::Rgb(0x23, 0x2a, 0x3d); // cursor row (white @ ~4%)

pub const SEP_ORANGE: Color = Color::Rgb(0x6e, 0x39, 0x1c); // header/footer rule
pub const SEP_WHITE: Color = Color::Rgb(0x2a, 0x2f, 0x3d); // inner rules
