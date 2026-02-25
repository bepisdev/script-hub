use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

// ── DOS colour palette ────────────────────────────────────────────────────────
const DOS_BLUE: Color = Color::Blue;
const DOS_CYAN: Color = Color::Cyan;
const DOS_YELLOW: Color = Color::Yellow;
const DOS_WHITE: Color = Color::White;
const DOS_BLACK: Color = Color::Black;
const DOS_DARK_GRAY: Color = Color::DarkGray;
const DOS_GREEN: Color = Color::Green;

// ── Data model ────────────────────────────────────────────────────────────────
#[derive(Clone)]
struct ScriptEntry {
    name: &'static str,
    description: &'static str,
    category: &'static str,
    command: &'static str,
}

struct App {
    scripts: Vec<ScriptEntry>,
    /// Index into `scripts` (not the rendered list which includes category headers)
    list_state: ListState,
    show_run_dialog: bool,
    last_message: Option<String>,
}

impl App {
    fn new() -> Self {
        let scripts = vec![
            ScriptEntry {
                name: "Backup Home",
                description: "Backs up the home directory to NAS storage.",
                category: "System",
                command: "backup_home.sh",
            },
            ScriptEntry {
                name: "Clean Temp Files",
                description: "Removes temporary files, caches and build artefacts.",
                category: "System",
                command: "clean_temp.sh",
            },
            ScriptEntry {
                name: "Update Dependencies",
                description: "Updates all project package dependencies to latest.",
                category: "Development",
                command: "update_deps.sh",
            },
            ScriptEntry {
                name: "Deploy to Staging",
                description: "Builds and deploys the current branch to the staging environment.",
                category: "Development",
                command: "deploy_staging.sh",
            },
            ScriptEntry {
                name: "DB Snapshot",
                description: "Takes a point-in-time snapshot of the production database.",
                category: "Database",
                command: "db_snapshot.sh",
            },
            ScriptEntry {
                name: "DB Restore",
                description: "Restores a previously taken database snapshot.",
                category: "Database",
                command: "db_restore.sh",
            },
            ScriptEntry {
                name: "Health Check",
                description: "Runs health checks across all registered services.",
                category: "Monitoring",
                command: "health_check.sh",
            },
            ScriptEntry {
                name: "Log Analyzer",
                description: "Parses and summarises recent application log files.",
                category: "Monitoring",
                command: "analyze_logs.sh",
            },
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        App {
            scripts,
            list_state,
            show_run_dialog: false,
            last_message: None,
        }
    }

    fn next(&mut self) {
        let len = self.scripts.len();
        let i = self
            .list_state
            .selected()
            .map(|i| if i >= len - 1 { 0 } else { i + 1 })
            .unwrap_or(0);
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let len = self.scripts.len();
        let i = self
            .list_state
            .selected()
            .map(|i| if i == 0 { len - 1 } else { i - 1 })
            .unwrap_or(0);
        self.list_state.select(Some(i));
    }

    fn selected_script(&self) -> Option<&ScriptEntry> {
        self.list_state.selected().and_then(|i| self.scripts.get(i))
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────
fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    // Always restore the terminal, even on error
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("{err:?}");
    }
    Ok(())
}

// ── Event loop ────────────────────────────────────────────────────────────────
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.show_run_dialog {
                match key.code {
                    KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(script) = app.selected_script() {
                            app.last_message =
                                Some(format!("Launched: {}", script.command));
                        }
                        app.show_run_dialog = false;
                    }
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.show_run_dialog = false;
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::F(10) => {
                        return Ok(());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.last_message = None;
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.last_message = None;
                        app.previous();
                    }
                    KeyCode::Enter | KeyCode::F(5) => {
                        if app.selected_script().is_some() {
                            app.show_run_dialog = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// ── Top-level layout ──────────────────────────────────────────────────────────
fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Blue "desktop" background
    f.render_widget(
        Block::default().style(Style::default().bg(DOS_BLUE)),
        size,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title bar
            Constraint::Min(0),    // content
            Constraint::Length(3), // status bar
        ])
        .split(size);

    render_title_bar(f, chunks[0]);
    render_content(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);

    if app.show_run_dialog {
        render_run_dialog(f, app, size);
    }
}

// ── Title bar ─────────────────────────────────────────────────────────────────
fn render_title_bar(f: &mut Frame, area: Rect) {
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
        Span::styled(
            "  v0.1.0",
            Style::default().fg(DOS_DARK_GRAY),
        ),
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

// ── Main content area (list + detail) ────────────────────────────────────────
fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    render_script_list(f, app, cols[0]);
    render_detail_panel(f, app, cols[1]);
}

// ── Script list panel ────────────────────────────────────────────────────────
fn render_script_list(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Scripts ",
            Style::default()
                .fg(DOS_YELLOW)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DOS_CYAN).bg(DOS_BLUE))
        .style(Style::default().bg(DOS_BLUE));

    // Build rendered items, inserting category headers between groups.
    // We also build a mapping: script_index → rendered_row_index.
    let mut rendered: Vec<ListItem> = Vec::new();
    let mut script_to_row: Vec<usize> = Vec::new();
    let mut current_category = "";

    for script in &app.scripts {
        if script.category != current_category {
            current_category = script.category;
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

// ── Detail panel ─────────────────────────────────────────────────────────────
fn render_detail_panel(f: &mut Frame, app: &App, area: Rect) {
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
        Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name     : ", Style::default().fg(DOS_DARK_GRAY)),
                Span::styled(
                    script.name,
                    Style::default()
                        .fg(DOS_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Category : ", Style::default().fg(DOS_DARK_GRAY)),
                Span::styled(script.category, Style::default().fg(DOS_CYAN)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Command  : ", Style::default().fg(DOS_DARK_GRAY)),
                Span::styled(script.command, Style::default().fg(DOS_WHITE)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ─────────────────────────────────────────",
                Style::default().fg(DOS_DARK_GRAY),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(script.description, Style::default().fg(DOS_WHITE)),
            ]),
        ])
    } else {
        Text::from("")
    };

    f.render_widget(
        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true }),
        area,
    );
}

// ── Status / help bar ─────────────────────────────────────────────────────────
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
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
            Span::styled(
                format!(" {}  ", desc),
                Style::default().fg(DOS_WHITE),
            ),
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

// ── Run confirmation dialog ───────────────────────────────────────────────────
fn render_run_dialog(f: &mut Frame, app: &App, area: Rect) {
    let Some(script) = app.selected_script() else {
        return;
    };

    let w: u16 = 52;
    let h: u16 = 9;
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

    let content = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Script  : ", Style::default().fg(DOS_DARK_GRAY)),
            Span::styled(
                script.name,
                Style::default()
                    .fg(DOS_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Command : ", Style::default().fg(DOS_DARK_GRAY)),
            Span::styled(script.command, Style::default().fg(DOS_CYAN)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  [Y / Enter] Run       [N / Esc] Cancel",
            Style::default().fg(DOS_WHITE),
        )),
    ]);

    f.render_widget(
        Paragraph::new(content).block(block).wrap(Wrap { trim: true }),
        dialog_area,
    );
}
