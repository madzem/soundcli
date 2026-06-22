//! Ratatui rendering of soundcli — centered, symmetric: header, now-playing + ASCII
//! progress bar, keypress-feedback controls, optional volume, upcoming queue, footer.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::model::PlayerState;
use crate::theme::*;

/// Which control the user just pressed — flashed briefly for feedback.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flash {
    Prev,
    Toggle,
    Next,
}

pub fn render(f: &mut Frame, st: &PlayerState, selected: usize, flash: Option<Flash>) {
    let full = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), full);

    let show_volume = st.volume.is_some();
    let total = st.tracks.len();
    let show_queue = total > 1;

    // One row per section, so the layout can never overflow into overlapping rows.
    #[derive(Clone, Copy)]
    enum Sec {
        Header,
        Rule(Color),
        Blank,
        Status,
        Progress,
        Controls,
        Volume,
        QueueHdr,
        QueueRow(usize), // absolute index into `st.tracks`
        Note,
        Footer,
    }

    // Size a window of the tracklist that keeps `selected` (an absolute index) on screen.
    let max_inner = full.height.saturating_sub(2) as usize;
    // Fixed rows: 9 core/trailing + 2 for volume + 3 for the queue header block.
    let non_queue = 9 + if show_volume { 2 } else { 0 } + if show_queue { 3 } else { 0 };
    let queue_cap = max_inner.saturating_sub(non_queue);
    let n = if show_queue { queue_cap.min(total) } else { 0 };
    let start = if n == 0 || n >= total {
        0
    } else {
        selected.saturating_sub(n / 2).min(total - n)
    };

    let mut secs = vec![
        Sec::Header,
        Sec::Rule(SEP_ORANGE),
        Sec::Blank,
        Sec::Status,
        Sec::Progress,
        Sec::Blank,
        Sec::Controls,
    ];
    if show_volume {
        secs.push(Sec::Blank);
        secs.push(Sec::Volume);
    }
    if show_queue && n > 0 {
        secs.push(Sec::Blank);
        secs.push(Sec::Rule(SEP_WHITE));
        secs.push(Sec::QueueHdr);
        for i in start..start + n {
            secs.push(Sec::QueueRow(i));
        }
    } else if st.queue_note.is_some() {
        secs.push(Sec::Blank);
        secs.push(Sec::Note);
    }
    secs.push(Sec::Rule(SEP_ORANGE));
    secs.push(Sec::Footer);

    let width = full.width.min(86);
    let height = (secs.len() as u16 + 2).min(full.height);
    let area = center(full, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ORANGE))
        .style(Style::default().bg(PANEL));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Drop rows by ascending importance until they fit, so the core always survives.
    let h = inner.height as usize;
    while secs.len() > h {
        let idx = secs
            .iter()
            .rposition(|s| matches!(s, Sec::QueueRow(_)))
            .or_else(|| secs.iter().rposition(|s| matches!(s, Sec::Blank)))
            .or_else(|| secs.iter().position(|s| matches!(s, Sec::Volume)))
            .or_else(|| secs.iter().position(|s| matches!(s, Sec::QueueHdr)))
            .or_else(|| secs.iter().rposition(|s| matches!(s, Sec::Rule(_))))
            .or_else(|| secs.iter().position(|s| matches!(s, Sec::Header)));
        match idx {
            Some(i) => {
                secs.remove(i);
            }
            None => {
                secs.truncate(h);
                break;
            }
        }
    }

    let rows = Layout::vertical(vec![Constraint::Length(1); secs.len()]).split(inner);
    for (i, s) in secs.iter().enumerate() {
        let r = rows[i];
        match s {
            Sec::Header => header(f, r, st),
            Sec::Rule(c) => rule(f, r, *c),
            Sec::Blank => {}
            Sec::Status => status(f, r, st),
            Sec::Progress => progress(f, r, st),
            Sec::Controls => controls(f, r, st, flash),
            Sec::Volume => volume(f, r, st),
            Sec::QueueHdr => queue_header(f, r),
            Sec::QueueRow(abs) => queue_row(f, r, st, &st.tracks[*abs], *abs, selected),
            Sec::Note => note(f, r, st),
            Sec::Footer => footer(f, r),
        }
    }
}

fn center(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

fn rule(f: &mut Frame, area: Rect, color: Color) {
    let line = "─".repeat(area.width as usize);
    f.render_widget(
        Paragraph::new(Span::styled(line, Style::default().fg(color))),
        area,
    );
}

fn header(f: &mut Frame, area: Rect, st: &PlayerState) {
    let mut spans = vec![Span::styled(
        "soundcli",
        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
    )];
    let name = st.playlist_name.trim();
    if !name.is_empty() && name != "—" {
        spans.push(Span::styled("  ·  ", Style::default().fg(DIM2)));
        spans.push(Span::styled(
            name.to_string(),
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ));
        if !st.queue_partial {
            spans.push(Span::styled(
                format!("  ·  {} tracks", st.tracks.len()),
                Style::default().fg(DIM),
            ));
        }
    }
    f.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .style(Style::default().bg(HEADER_BG)),
        area,
    );
}

fn status(f: &mut Frame, area: Rect, st: &PlayerState) {
    let mut spans = Vec::new();
    if !st.connected {
        spans.push(Span::styled(
            "WAITING FOR SOUNDCLOUD TAB",
            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "   press play once in your browser",
            Style::default().fg(DIM),
        ));
        f.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );
        return;
    }
    let label_w = if st.playing {
        "● NOW PLAYING".width()
    } else {
        "PAUSED".width()
    };
    if st.playing {
        spans.push(Span::styled("● ", Style::default().fg(ORANGE)));
        spans.push(Span::styled(
            "NOW PLAYING",
            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            "PAUSED",
            Style::default().fg(DIM).add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::raw("   "));
    if let Some(cur) = st.current() {
        // Budget the remaining width so a long (ad/station) title can't overflow the box.
        let budget = (area.width as usize).saturating_sub(label_w + 3 + 2);
        let sep_artist = if cur.artist.is_empty() {
            0
        } else {
            5 + cur.artist.width()
        };
        let title_room = budget.saturating_sub(sep_artist);
        let (title, show_artist) = if title_room >= 4 {
            (truncate(&cur.title, title_room), !cur.artist.is_empty())
        } else {
            (truncate(&cur.title, budget.max(4)), false)
        };
        spans.push(Span::styled(
            title,
            Style::default()
                .fg(TEXT_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ));
        if show_artist {
            spans.push(Span::styled("  —  ", Style::default().fg(DIM)));
            spans.push(Span::styled(cur.artist.clone(), Style::default().fg(TEXT)));
        }
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn progress(f: &mut Frame, area: Rect, st: &PlayerState) {
    let (elapsed, dur) = match st.current() {
        Some(c) => (st.elapsed.min(c.dur.max(1)), c.dur),
        None => (0, 0),
    };
    let pct = if dur > 0 {
        (elapsed as f64 / dur as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let left = fmt(elapsed);
    let right = format!("-{} / {}", fmt(dur.saturating_sub(elapsed)), fmt(dur));
    let side = left.width().max(right.width());
    let reserved = 1 + side + 1 + 1 + 1 + side + 1;
    let w = (area.width as usize).saturating_sub(reserved).clamp(10, 80);
    let fill = if dur > 0 {
        ((pct * w as f64).round() as usize).clamp(1, w)
    } else {
        0
    };
    let filled = "=".repeat(fill.saturating_sub(1));
    let head = if fill > 0 { ">" } else { "" };
    let empty = "-".repeat(w - fill);

    let spans = vec![
        Span::styled(format!("{left:>side$}"), Style::default().fg(ORANGE_LT)),
        Span::styled(" [", Style::default().fg(DIM2)),
        Span::styled(filled, Style::default().fg(ORANGE)),
        Span::styled(head.to_string(), Style::default().fg(ORANGE_HEAD)),
        Span::styled(empty, Style::default().fg(BAR_EMPTY)),
        Span::styled("] ", Style::default().fg(DIM2)),
        Span::styled(format!("{right:<side$}"), Style::default().fg(ORANGE_LT)),
    ];
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn controls(f: &mut Frame, area: Rect, st: &PlayerState, flash: Option<Flash>) {
    let pressed = |this: Flash, base: Style| -> Style {
        if flash == Some(this) {
            Style::default()
                .fg(TOGGLE_FG)
                .bg(ORANGE)
                .add_modifier(Modifier::BOLD)
        } else {
            base
        }
    };
    let side = Style::default().fg(DIM);
    let primary = Style::default().fg(ORANGE).add_modifier(Modifier::BOLD);
    let toggle_label = if st.playing {
        "  ⏸  pause  "
    } else {
        "  ▶  play  "
    };

    let spans = vec![
        Span::styled("  ⏮  prev  ", pressed(Flash::Prev, side)),
        Span::raw("      "),
        Span::styled(toggle_label, pressed(Flash::Toggle, primary)),
        Span::raw("      "),
        Span::styled("  next  ⏭  ", pressed(Flash::Next, side)),
    ];
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn volume(f: &mut Frame, area: Rect, st: &PlayerState) {
    let v = st.volume.unwrap_or(0.0);
    let pct = (v * 100.0).round() as i64;
    let cells = 12usize;
    let filled = (v * cells as f64).round() as usize;
    let bar_on: String = "▆".repeat(filled);
    let bar_off: String = "▆".repeat(cells.saturating_sub(filled));
    let spans = vec![
        Span::styled("vol ", Style::default().fg(DIM2)),
        Span::styled(bar_on, Style::default().fg(ORANGE)),
        Span::styled(bar_off, Style::default().fg(BAR_EMPTY)),
        Span::styled(format!(" {pct}%"), Style::default().fg(DIM2)),
    ];
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn note(f: &mut Frame, area: Rect, st: &PlayerState) {
    if let Some(n) = &st.queue_note {
        f.render_widget(
            Paragraph::new(Span::styled(n.clone(), Style::default().fg(DIM)))
                .alignment(Alignment::Center),
            area,
        );
    }
}

fn queue_header(f: &mut Frame, area: Rect) {
    f.render_widget(
        Paragraph::new(Span::styled("QUEUE", Style::default().fg(DIM)))
            .alignment(Alignment::Center),
        area,
    );
}

fn queue_row(
    f: &mut Frame,
    area: Rect,
    st: &PlayerState,
    t: &crate::model::Track,
    abs: usize,
    selected: usize,
) {
    let is_sel = abs == selected;
    let is_current = abs == st.current_idx;
    let bg = if is_sel { SELECT_BG } else { PANEL };
    // ▸ selection cursor, ● now playing.
    let marker = if is_sel {
        "▸"
    } else if is_current {
        "●"
    } else {
        " "
    };
    let num = format!("{:02}", abs + 1);
    let dur = fmt(t.dur);

    let head = vec![
        Span::raw("  "),
        Span::styled(format!("{marker} "), Style::default().fg(ORANGE)),
        Span::styled(num, Style::default().fg(DIM2)),
        Span::raw("  "),
    ];
    let tail = vec![
        Span::styled(truncate(&t.artist, 22), Style::default().fg(DIM)),
        Span::raw("  "),
        Span::styled(dur, Style::default().fg(DIM2)),
        Span::raw("  "),
    ];
    let avail = (area.width as usize).saturating_sub(width_of(&head) + width_of(&tail));
    let title = truncate(&t.title, avail.max(4));
    let title_style = if is_sel || is_current {
        Style::default().fg(TEXT_BRIGHT)
    } else {
        Style::default().fg(TEXT)
    };
    let mut spans = head;
    spans.push(Span::styled(format!("{title:<avail$}"), title_style));
    spans.extend(tail);
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
        area,
    );
}

fn footer(f: &mut Frame, area: Rect) {
    let key = |k: &str| Span::styled(k.to_string(), Style::default().fg(ACCENT));
    let txt = |t: &str| Span::styled(t.to_string(), Style::default().fg(FOOT));
    let spans = vec![
        key("space"),
        txt(" play   "),
        key("↑↓"),
        txt(" select   "),
        key("enter"),
        txt(" jump   "),
        key("←→"),
        txt(" seek   "),
        key("+−"),
        txt(" vol   "),
        key("q"),
        txt(" quit"),
    ];
    f.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .style(Style::default().bg(HEADER_BG)),
        area,
    );
}

fn width_of(spans: &[Span]) -> usize {
    spans.iter().map(|s| s.content.width()).sum()
}

fn fmt(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

fn truncate(s: &str, max: usize) -> String {
    if s.width() <= max {
        return s.to_string();
    }
    if max <= 1 {
        return "…".to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = ch.to_string().width();
        if w + cw > max - 1 {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::{Controller, DemoController};
    use ratatui::{backend::TestBackend, Terminal};

    fn render_to_text(w: u16, h: u16) -> String {
        let st = DemoController::new().refresh();
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| render(f, &st, 0, Some(Flash::Next))).unwrap();
        let buf = term.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn renders_demo_layout() {
        let text = render_to_text(86, 22);
        if let Ok(p) = std::env::var("SOUNDCLI_FRAME_OUT") {
            let _ = std::fs::write(p, &text);
        }
        for needle in [
            "soundcli",
            "Night Drive",
            "NOW PLAYING",
            "pause",
            "QUEUE",
            "select",
        ] {
            assert!(text.contains(needle), "missing {needle:?} in frame");
        }
        assert!(
            text.contains("Midnight City Lights"),
            "current track missing from queue"
        );
        assert!(
            text.contains("Analog Hearts"),
            "deep track missing from queue"
        );
    }

    fn render_state_sel(st: &PlayerState, w: u16, h: u16, sel: usize) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| render(f, st, sel, None)).unwrap();
        let buf = term.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    /// A tracklist taller than the box must scroll the selection into view, not clip it.
    #[test]
    fn long_queue_scrolls_to_selection() {
        use crate::model::Track;
        let tracks: Vec<Track> = (0..40)
            .map(|i| Track::new(&format!("Song {i:02}"), "Artist", 180))
            .collect();
        let st = PlayerState {
            playlist_name: "Big Set".into(),
            tracks,
            current_idx: 0,
            elapsed: 0,
            playing: true,
            volume: None,
            queue_partial: false,
            source: "mpris".into(),
            connected: true,
            queue_note: None,
        };
        let text = render_state_sel(&st, 86, 18, 33);
        assert!(
            text.contains("Song 33"),
            "selected deep track not scrolled into view:\n{text}"
        );
        assert_eq!(text.matches("quit").count(), 1);
        for line in text.lines() {
            assert!(line.chars().count() <= 86, "row wider than box");
        }
    }

    fn render_state(st: &PlayerState, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| render(f, st, 0, None)).unwrap();
        let buf = term.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    /// A long title on a short terminal must not overflow into overlapping rows.
    #[test]
    fn short_terminal_no_overlap() {
        use crate::model::Track;
        let st = PlayerState {
            playlist_name: "—".into(),
            tracks: vec![Track::new(
                "Stream Shemrooni | Listen to Lofi playlist online for free on SoundCloud",
                "",
                0,
            )],
            current_idx: 0,
            elapsed: 0,
            playing: true,
            volume: Some(1.0),
            queue_partial: true,
            source: "mpris".into(),
            connected: true,
            queue_note: None,
        };
        for h in [8u16, 10, 14, 24] {
            let text = render_state(&st, 86, h);
            assert_eq!(
                text.matches("quit").count(),
                1,
                "footer dup at h={h}\n{text}"
            );
            assert!(text.contains("NOW PLAYING"), "no status at h={h}");
            for line in text.lines() {
                assert!(line.chars().count() <= 86, "row wider than box at h={h}");
            }
        }
    }
}
