mod detail_panel;
mod run_dialog;
mod script_list;
mod status_bar;
mod title_bar;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
    Frame,
};

use crate::{app::App, theme::DOS_BLUE};

/// Top-level UI entry point — called every frame.
pub fn ui(f: &mut Frame, app: &mut App) {
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

    title_bar::render_title_bar(f, chunks[0]);
    render_content(f, app, chunks[1]);
    status_bar::render_status_bar(f, app, chunks[2]);

    if app.show_run_dialog {
        run_dialog::render_run_dialog(f, app, size);
    }
}

fn render_content(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    script_list::render_script_list(f, app, cols[0]);
    detail_panel::render_detail_panel(f, app, cols[1]);
}
