use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::App,
    theme::{DOS_BLUE, DOS_CYAN, DOS_DARK_GRAY, DOS_GREEN, DOS_RED, DOS_WHITE, DOS_YELLOW},
};

pub(super) fn render_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Details ",
            Style::default()
                .fg(DOS_YELLOW)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DOS_CYAN).bg(DOS_BLUE))
        .style(Style::default().bg(DOS_BLUE));

    let content = if let Some(script) = app.selected_script() {
        let filename = script
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let full_path = script.path.display().to_string();

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name     : ", Style::default().fg(DOS_DARK_GRAY)),
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
            Line::from(vec![
                Span::styled("  Path     : ", Style::default().fg(DOS_DARK_GRAY)),
                Span::styled(full_path, Style::default().fg(DOS_DARK_GRAY)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ─────────────────────────────────────────",
                Style::default().fg(DOS_DARK_GRAY),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(script.description.clone(), Style::default().fg(DOS_WHITE)),
            ]),
        ];

        // ── Declared arguments ────────────────────────────────────────────
        if !script.args.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ─────────────────────────────────────────",
                Style::default().fg(DOS_DARK_GRAY),
            )));
            for arg in &script.args {
                let req = if arg.required { " *" } else { "  " };
                let default_hint = if arg.default.is_empty() {
                    String::new()
                } else {
                    format!("  (default: {})", arg.default)
                };
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(
                        format!("{:<20}", arg.label),
                        Style::default().fg(DOS_CYAN),
                    ),
                    Span::styled(req, Style::default().fg(DOS_RED)),
                    Span::styled(default_hint, Style::default().fg(DOS_DARK_GRAY)),
                ]));
            }
        }

        append_warnings(&mut lines, &app.load_warnings);
        Text::from(lines)
    } else {
        // No scripts loaded — show helpful info
        let dir = app.scripts_dir.display().to_string();
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No scripts found.",
                Style::default()
                    .fg(DOS_YELLOW)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Place executable scripts in:",
                Style::default().fg(DOS_WHITE),
            )),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(dir, Style::default().fg(DOS_CYAN)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  Each script must contain these header",
                Style::default().fg(DOS_DARK_GRAY),
            )),
            Line::from(Span::styled(
                "  comment tags (first 40 lines):",
                Style::default().fg(DOS_DARK_GRAY),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "    # @name:        My Script",
                Style::default().fg(DOS_GREEN),
            )),
            Line::from(Span::styled(
                "    # @description: What it does",
                Style::default().fg(DOS_GREEN),
            )),
            Line::from(Span::styled(
                "    # @category:    Category",
                Style::default().fg(DOS_GREEN),
            )),
        ];
        append_warnings(&mut lines, &app.load_warnings);
        Text::from(lines)
    };

    f.render_widget(
        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true }),
        area,
    );
}

/// Appends the load-warnings section to a line vec if there are any warnings.
fn append_warnings<'a>(lines: &mut Vec<Line<'a>>, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ─── Load warnings ───────────────────────",
        Style::default().fg(DOS_DARK_GRAY),
    )));
    for w in warnings {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(w.clone(), Style::default().fg(DOS_RED)),
        ]));
    }
}
