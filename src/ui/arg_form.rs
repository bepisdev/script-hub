use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, DialogState},
    theme::{DOS_BLACK, DOS_CYAN, DOS_DARK_GRAY, DOS_RED, DOS_WHITE, DOS_YELLOW},
};

pub(super) fn render_arg_form(f: &mut Frame, app: &App, area: Rect) {
    let DialogState::ArgsForm {
        focused_field,
        values,
    } = &app.dialog
    else {
        return;
    };
    let Some(script) = app.selected_script() else {
        return;
    };

    let n_args = script.args.len() as u16;
    // 2 rows (blank + script info) + (2 rows per arg) + 2 rows (blank + keys) + optional warn row
    let has_required = script.args.iter().any(|a| a.required);
    let inner_rows = 2 + n_args * 2 + 2 + if has_required { 1 } else { 0 };
    let h = (inner_rows + 2).max(10).min(area.height.saturating_sub(2)); // +2 for borders
    let w: u16 = 66;

    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let dialog_area = Rect::new(x, y, w.min(area.width), h.min(area.height));

    f.render_widget(Clear, dialog_area);

    let title = format!(" {} — Arguments ", script.name);
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(DOS_YELLOW)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DOS_WHITE).bg(DOS_BLACK))
        .style(Style::default().bg(DOS_BLACK));

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Category : ", Style::default().fg(DOS_DARK_GRAY)),
            Span::styled(script.category.clone(), Style::default().fg(DOS_CYAN)),
        ]),
    ];

    // ── Field rows ────────────────────────────────────────────────────────────
    let field_width: usize = 36;

    for (i, arg) in script.args.iter().enumerate() {
        let is_focused = i == *focused_field;
        let value = values.get(i).map(String::as_str).unwrap_or("");

        lines.push(Line::from(""));

        // Label row
        let req_mark = if arg.required {
            Span::styled(" *", Style::default().fg(DOS_RED))
        } else {
            Span::raw("  ")
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{:<20}", arg.label),
                if is_focused {
                    Style::default()
                        .fg(DOS_YELLOW)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(DOS_DARK_GRAY)
                },
            ),
            req_mark,
        ]));

        // Input box row — truncate to last `field_width` chars so cursor stays visible
        let cursor_str = if is_focused { "\u{258c}" } else { "" }; // ▌
        let display_raw = format!("{value}{cursor_str}");
        let char_count = display_raw.chars().count();
        let display: String = if char_count > field_width {
            display_raw
                .chars()
                .skip(char_count - field_width)
                .collect()
        } else {
            format!("{display_raw:<field_width$}")
        };

        let input_style = if is_focused {
            Style::default().bg(DOS_WHITE).fg(DOS_BLACK)
        } else {
            Style::default().bg(DOS_DARK_GRAY).fg(DOS_WHITE)
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!(" {display} "), input_style),
        ]));
    }

    lines.push(Line::from(""));

    // ── Validation row ────────────────────────────────────────────────────────
    let unfilled_required = script.args.iter().enumerate().any(|(i, arg)| {
        arg.required && values.get(i).map(|v| v.is_empty()).unwrap_or(true)
    });
    if unfilled_required {
        lines.push(Line::from(Span::styled(
            "  * Required fields must be filled",
            Style::default().fg(DOS_RED),
        )));
    }

    // ── Key hints row ─────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "  [Tab/↓↑] Navigate   [Enter] Run   [Esc] Cancel",
        Style::default().fg(DOS_WHITE),
    )));

    f.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        dialog_area,
    );
}
