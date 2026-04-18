use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Globe Scale View ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));
    let text = Paragraph::new("To-scale globe goes here...").block(block);
    f.render_widget(text, area);
}
