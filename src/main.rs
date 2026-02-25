use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

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
const DOS_RED: Color = Color::Red;

// ── Data model ────────────────────────────────────────────────────────────────
#[derive(Clone)]
struct ScriptEntry {
    name: String,
    description: String,
    category: String,
    /// Absolute path to the script file on disk
    path: PathBuf,
}

struct App {
    scripts: Vec<ScriptEntry>,
    /// Index into `scripts` (not the rendered list which includes category headers)
    list_state: ListState,
    show_run_dialog: bool,
    last_message: Option<String>,
    /// Non-fatal warnings accumulated while loading scripts
    load_warnings: Vec<String>,
    /// The directory that was scanned
    scripts_dir: PathBuf,
}

impl App {
    fn new(scripts: Vec<ScriptEntry>, load_warnings: Vec<String>, scripts_dir: PathBuf) -> Self {
        let mut list_state = ListState::default();
        if !scripts.is_empty() {
            list_state.select(Some(0));
        }
        App {
            scripts,
            list_state,
            show_run_dialog: false,
            last_message: None,
            load_warnings,
            scripts_dir,
        }
    }

    fn next(&mut self) {
        let len = self.scripts.len();
        if len == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map(|i| if i >= len - 1 { 0 } else { i + 1 })
            .unwrap_or(0);
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let len = self.scripts.len();
        if len == 0 {
            return;
        }
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

// ── Script discovery ──────────────────────────────────────────────────────────

/// Resolve the `scripts.d` directory.  Searches:
///   1. Next to the running binary
///   2. Current working directory
fn find_scripts_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let candidate = exe
            .parent()
            .unwrap_or(Path::new("."))
            .join("scripts.d");
        if candidate.is_dir() {
            return candidate;
        }
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("scripts.d")
}

/// Extract metadata tags from a script file's header comments.
///
/// Recognised tags (anywhere in the first 40 lines):
/// ```
/// # @name:        My Script
/// # @description: What it does
/// # @category:    Category Name
/// ```
///
/// Returns `None` if `@name` is absent (required).
fn parse_script_metadata(path: &Path) -> Result<ScriptEntry, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("{}: {}", path.display(), e))?;

    let mut name: Option<String> = None;
    let mut description = String::new();
    let mut category = String::from("Uncategorized");

    for line in content.lines().take(40) {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("# @name:") {
            name = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("# @description:") {
            description = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("# @category:") {
            category = val.trim().to_string();
        }
    }

    let name = name.ok_or_else(|| {
        format!(
            "{}: missing required tag `# @name:`",
            path.file_name().unwrap_or_default().to_string_lossy()
        )
    })?;

    Ok(ScriptEntry {
        name,
        description,
        category,
        path: path.to_path_buf(),
    })
}

/// Load all scripts from `scripts_dir`, returning the list and any
/// non-fatal warnings (e.g. files that were skipped due to missing tags).
fn load_scripts(scripts_dir: &Path) -> (Vec<ScriptEntry>, Vec<String>) {
    let mut scripts: Vec<ScriptEntry> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let read_dir = match fs::read_dir(scripts_dir) {
        Ok(rd) => rd,
        Err(e) => {
            warnings.push(format!(
                "Cannot open {}: {}",
                scripts_dir.display(),
                e
            ));
            return (scripts, warnings);
        }
    };

    let mut paths: Vec<PathBuf> = read_dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    paths.sort();

    for path in &paths {
        match parse_script_metadata(path) {
            Ok(entry) => scripts.push(entry),
            Err(msg) => warnings.push(format!("Skipped — {msg}")),
        }
    }

    // Sort by category, then by name (case-insensitive)
    scripts.sort_by(|a, b| {
        a.category
            .to_lowercase()
            .cmp(&b.category.to_lowercase())
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    (scripts, warnings)
}

// ── Entry point ───────────────────────────────────────────────────────────────
fn main() -> io::Result<()> {
    let scripts_dir = find_scripts_dir();
    let (scripts, load_warnings) = load_scripts(&scripts_dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(scripts, load_warnings, scripts_dir);
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
                            let filename = script
                                .path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned();
                            app.last_message = Some(format!("Launched: {filename}"));
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
    // We also build a mapping: script_index → rendered_row_index.
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

        // Show load warnings at the bottom of the detail pane
        if !app.load_warnings.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ─── Load warnings ───────────────────────",
                Style::default().fg(DOS_DARK_GRAY),
            )));
            for w in &app.load_warnings {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(w.clone(), Style::default().fg(DOS_RED)),
                ]));
            }
        }

        Text::from(lines)
    } else {
        // No scripts loaded — show helpful info
        let dir = app.scripts_dir.display().to_string();
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No scripts found.",
                Style::default().fg(DOS_YELLOW).add_modifier(Modifier::BOLD),
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
        if !app.load_warnings.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ─── Load warnings ───────────────────────",
                Style::default().fg(DOS_DARK_GRAY),
            )));
            for w in &app.load_warnings {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(w.clone(), Style::default().fg(DOS_RED)),
                ]));
            }
        }
        Text::from(lines)
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
        Paragraph::new(content).block(block).wrap(Wrap { trim: true }),
        dialog_area,
    );
}
