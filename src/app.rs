//! Terminal lifecycle + input loop.

use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::controller::Controller;
use crate::ui::{self, Flash};

const FLASH_MS: u128 = 220;

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn run(mut controller: Box<dyn Controller>) -> Result<()> {
    let mut term = setup()?;
    let res = event_loop(&mut term, &mut controller);
    restore(&mut term)?;
    res
}

fn event_loop(term: &mut Tui, controller: &mut Box<dyn Controller>) -> Result<()> {
    let mut selected = 0usize;
    let mut flash: Option<(Flash, Instant)> = None;
    loop {
        let state = controller.refresh();
        let total = state.tracks.len();
        if total > 0 && selected >= total {
            selected = total - 1;
        }
        let active = flash
            .filter(|(_, t)| t.elapsed().as_millis() < FLASH_MS)
            .map(|(fl, _)| fl);
        term.draw(|f| ui::render(f, &state, selected, active))?;

        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                // Raw mode swallows SIGINT, so Ctrl+C / Ctrl+D arrive as key events.
                if k.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(k.code, KeyCode::Char('c') | KeyCode::Char('d'))
                {
                    controller.stop();
                    break;
                }
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        controller.stop();
                        break;
                    }
                    KeyCode::Char(' ') => {
                        controller.toggle();
                        flash = Some((Flash::Toggle, Instant::now()));
                    }
                    KeyCode::Char('n') => {
                        controller.next();
                        flash = Some((Flash::Next, Instant::now()));
                    }
                    KeyCode::Char('p') => {
                        controller.prev();
                        flash = Some((Flash::Prev, Instant::now()));
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let cur = state.volume.unwrap_or(0.0);
                        controller.set_volume((cur + 0.05).min(1.0));
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let cur = state.volume.unwrap_or(0.0);
                        controller.set_volume((cur - 0.05).max(0.0));
                    }
                    KeyCode::Left => controller.seek_to(state.elapsed.saturating_sub(5)),
                    KeyCode::Right => {
                        let cap = state
                            .current()
                            .map(|c| c.dur.saturating_sub(1))
                            .unwrap_or(0);
                        controller.seek_to((state.elapsed + 5).min(cap));
                    }
                    KeyCode::Up | KeyCode::Char('k') if total > 0 => {
                        selected = (selected + total - 1) % total;
                    }
                    KeyCode::Down | KeyCode::Char('j') if total > 0 => {
                        selected = (selected + 1) % total;
                    }
                    KeyCode::Enter => controller.play_index(selected),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn setup() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let term = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(term)
}

fn restore(term: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    Ok(())
}
