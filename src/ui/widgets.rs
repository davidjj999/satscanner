use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.current_view {
        crate::views::View::Overhead => "Overhead Map",
        crate::views::View::GlobeScale => "To-Scale Globe",
        crate::views::View::GlobeBands => "Altitude-Banded Globe",
    };

    let tle_status = if app.is_fetching_tles {
        "Fetching TLEs..."
    } else {
        "Data Ready"
    };

    let text = format!(" View: {} | Status: {} | Sats: {} ", view_name, tle_status, app.loaded_tles);

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(paragraph, area);
}
