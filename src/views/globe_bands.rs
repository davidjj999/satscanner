use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Globe Bands View ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));
    let text = Paragraph::new("Log-scaled globe goes here...").block(block);
    f.render_widget(text, area);
}
