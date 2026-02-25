mod app;
mod loader;
mod theme;
mod ui;

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, DialogState};
use loader::{find_scripts_dir, load_scripts};
use ui::ui;

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

    // Always restore the terminal, even on error.
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

            match &app.dialog {
                DialogState::None => match key.code {
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
                        app.open_dialog();
                    }
                    _ => {}
                },

                DialogState::Confirm => match key.code {
                    KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                        launch_script(app);
                        app.close_dialog();
                    }
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.close_dialog();
                    }
                    _ => {}
                },

                DialogState::ArgsForm { .. } => handle_arg_form_key(app, key.code),
            }
        }
    }
}

fn handle_arg_form_key(app: &mut App, code: KeyCode) {
    let DialogState::ArgsForm {
        focused_field,
        values,
    } = &mut app.dialog
    else {
        return;
    };

    let n_fields = values.len();
    if n_fields == 0 {
        return;
    }

    match code {
        KeyCode::Tab | KeyCode::Down => {
            *focused_field = (*focused_field + 1) % n_fields;
        }
        KeyCode::BackTab | KeyCode::Up => {
            *focused_field = if *focused_field == 0 {
                n_fields - 1
            } else {
                *focused_field - 1
            };
        }
        KeyCode::Char(c) => {
            if let Some(v) = values.get_mut(*focused_field) {
                v.push(c);
            }
        }
        KeyCode::Backspace => {
            if let Some(v) = values.get_mut(*focused_field) {
                v.pop();
            }
        }
        KeyCode::Enter | KeyCode::F(5) => {
            let required_unfilled = app
                .selected_script()
                .map(|s| {
                    let DialogState::ArgsForm { values, .. } = &app.dialog else {
                        return false;
                    };
                    s.args.iter().enumerate().any(|(i, arg)| {
                        arg.required && values.get(i).map(|v| v.is_empty()).unwrap_or(true)
                    })
                })
                .unwrap_or(false);

            if !required_unfilled {
                launch_script(app);
                app.close_dialog();
            }
        }
        KeyCode::Esc => {
            app.close_dialog();
        }
        _ => {}
    }
}

fn launch_script(app: &mut App) {
    let Some(script) = app.selected_script() else {
        return;
    };
    let filename = script
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let arg_summary = if let DialogState::ArgsForm { values, .. } = &app.dialog {
        if values.iter().any(|v| !v.is_empty()) {
            let pairs: Vec<String> = script
                .args
                .iter()
                .zip(values.iter())
                .filter(|(_, v)| !v.is_empty())
                .map(|(a, v)| format!("{}={}", a.name, v))
                .collect();
            format!(" [{}]", pairs.join(", "))
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    app.last_message = Some(format!("Launched: {filename}{arg_summary}"));
}
