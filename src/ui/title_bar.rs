use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::theme::{DOS_BLUE, DOS_CYAN, DOS_DARK_GRAY, DOS_WHITE, DOS_YELLOW};

pub(super) fn render_title_bar(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DOS_YELLOW).bg(DOS_BLUE))
        .style(Style::default().bg(DOS_BLUE));

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "SCRIPT",
            Style::default()
                .fg(DOS_YELLOW)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "-HUB",
            Style::default()
                .fg(DOS_WHITE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  v0.1.0", Style::default().fg(DOS_DARK_GRAY)),
        Span::raw("          "),
        Span::styled(
            "[ Automation Script Launcher ]",
            Style::default().fg(DOS_CYAN),
        ),
    ]))
    .block(block)
    .alignment(Alignment::Left);

    f.render_widget(title, area);
}
