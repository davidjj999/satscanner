use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, panic, time::Duration};

mod app;
mod config;
mod satellite;
mod ui;
mod views;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing
    tracing_subscriber::fmt::init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup panic hook to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Initialize the app
    let mut app = App::new();
    
    // Trigger startup task (e.g., fetch TLEs)
    app.init();

    // Main event loop
    let tick_rate = Duration::from_millis(16); // ~60fps target
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                        KeyCode::Char('1') => app.set_view(views::View::Overhead),
                        KeyCode::Char('2') => app.set_view(views::View::GlobeScale),
                        KeyCode::Char('3') => app.set_view(views::View::GlobeBands),
                        KeyCode::Right => app.navigate_spatial(1.0, 0.0),
                        KeyCode::Left => app.navigate_spatial(-1.0, 0.0),
                        KeyCode::Up => app.navigate_spatial(0.0, 1.0),
                        KeyCode::Down => app.navigate_spatial(0.0, -1.0),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = std::time::Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
