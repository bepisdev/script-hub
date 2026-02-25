use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{
    app::App,
    theme::{DOS_BLUE, DOS_GREEN, DOS_WHITE, DOS_YELLOW},
};

pub(super) fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DOS_YELLOW).bg(DOS_BLUE))
        .style(Style::default().bg(DOS_BLUE));

    let hint = |key: &'static str, desc: &'static str| -> Vec<Span<'static>> {
        vec![
            Span::styled(
                key,
                Style::default()
                    .fg(DOS_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {}  ", desc), Style::default().fg(DOS_WHITE)),
        ]
    };

    let mut spans: Vec<Span> = vec![Span::raw("  ")];
    spans.extend(hint("↑↓", "Navigate"));
    spans.extend(hint("Enter", "Run"));
    spans.extend(hint("F10", "Quit"));
    spans.extend(hint("Q", "Quit"));

    if let Some(msg) = &app.last_message {
        spans.push(Span::styled(
            format!("│  ✓ {msg}"),
            Style::default()
                .fg(DOS_GREEN)
                .add_modifier(Modifier::BOLD),
        ));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans))
            .block(block)
            .alignment(Alignment::Left),
        area,
    );
}
