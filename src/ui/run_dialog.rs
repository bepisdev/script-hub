use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::App,
    theme::{DOS_BLACK, DOS_CYAN, DOS_DARK_GRAY, DOS_WHITE, DOS_YELLOW},
};

pub(super) fn render_run_dialog(f: &mut Frame, app: &App, area: Rect) {
    let Some(script) = app.selected_script() else {
        return;
    };

    let w: u16 = 58;
    let h: u16 = 11;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let dialog_area = Rect::new(x, y, w.min(area.width), h.min(area.height));

    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(Span::styled(
            " Confirm Run ",
            Style::default()
                .fg(DOS_YELLOW)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DOS_WHITE).bg(DOS_BLACK))
        .style(Style::default().bg(DOS_BLACK));

    let filename = script
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let content = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Script   : ", Style::default().fg(DOS_DARK_GRAY)),
            Span::styled(
                script.name.clone(),
                Style::default()
                    .fg(DOS_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Category : ", Style::default().fg(DOS_DARK_GRAY)),
            Span::styled(script.category.clone(), Style::default().fg(DOS_CYAN)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  File     : ", Style::default().fg(DOS_DARK_GRAY)),
            Span::styled(filename, Style::default().fg(DOS_WHITE)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  [Y / Enter] Run       [N / Esc] Cancel",
            Style::default().fg(DOS_WHITE),
        )),
    ]);

    f.render_widget(
        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true }),
        dialog_area,
    );
}
