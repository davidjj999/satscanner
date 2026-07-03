pub mod help;
pub mod log_panel;
pub mod widgets;

use crate::app::App;
use crate::views;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.size());

    match app.current_view {
        views::View::Overhead => views::overhead::draw(f, app, chunks[0]),
        views::View::Sky => views::globe_scale::draw(f, app, chunks[0]),
        views::View::GlobeBands => views::globe_bands::draw(f, chunks[0]),
    }

    widgets::draw_status_bar(f, app, chunks[1]);

    // Render help overlay on top if active
    if app.show_help {
        help::draw_help_overlay(f, f.size());
    }

    // Render log panel on top if active
    if app.show_log {
        log_panel::draw_log_panel(f, f.size(), &app.log);
    }
}
