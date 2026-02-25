use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::{
    app::App,
    theme::{DOS_BLACK, DOS_BLUE, DOS_CYAN, DOS_DARK_GRAY, DOS_WHITE, DOS_YELLOW},
};

pub(super) fn render_script_list(f: &mut Frame, app: &mut App, area: Rect) {
    let script_count = app.scripts.len();
    let title_text = format!(" Scripts ({script_count}) ");

    let block = Block::default()
        .title(Span::styled(
            title_text,
            Style::default()
                .fg(DOS_YELLOW)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DOS_CYAN).bg(DOS_BLUE))
        .style(Style::default().bg(DOS_BLUE));

    if app.scripts.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No scripts found.",
                Style::default().fg(DOS_DARK_GRAY),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Add scripts to scripts.d/",
                Style::default().fg(DOS_WHITE),
            )),
        ])
        .block(block);
        f.render_widget(msg, area);
        return;
    }

    // Build rendered items, inserting category headers between groups.
    // Also build a mapping: script_index → rendered_row_index.
    let mut rendered: Vec<ListItem> = Vec::new();
    let mut script_to_row: Vec<usize> = Vec::new();
    let mut current_category = String::new();

    for script in &app.scripts {
        if script.category != current_category {
            current_category = script.category.clone();
            rendered.push(ListItem::new(Line::from(vec![Span::styled(
                format!(" ── {} ", script.category),
                Style::default()
                    .fg(DOS_YELLOW)
                    .add_modifier(Modifier::BOLD),
            )])));
        }
        script_to_row.push(rendered.len());
        rendered.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!("► {}", script.name),
                Style::default().fg(DOS_WHITE),
            ),
        ])));
    }

    let mut adjusted = ListState::default();
    if let Some(sel) = app.list_state.selected() {
        adjusted.select(script_to_row.get(sel).copied());
    }

    let list = List::new(rendered)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(DOS_CYAN)
                .fg(DOS_BLACK)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");

    f.render_stateful_widget(list, area, &mut adjusted);
}
