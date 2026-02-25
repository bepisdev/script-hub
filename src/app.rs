use std::path::PathBuf;

use ratatui::widgets::ListState;

/// A single script discovered on disk.
#[derive(Clone)]
pub struct ScriptEntry {
    pub name: String,
    pub description: String,
    pub category: String,
    /// Absolute path to the script file on disk.
    pub path: PathBuf,
}

/// Top-level application state.
pub struct App {
    pub scripts: Vec<ScriptEntry>,
    /// Selection index into `scripts` (excludes category-header rows).
    pub list_state: ListState,
    pub show_run_dialog: bool,
    pub last_message: Option<String>,
    /// Non-fatal warnings accumulated while loading scripts.
    pub load_warnings: Vec<String>,
    /// The directory that was scanned.
    pub scripts_dir: PathBuf,
}

impl App {
    pub fn new(
        scripts: Vec<ScriptEntry>,
        load_warnings: Vec<String>,
        scripts_dir: PathBuf,
    ) -> Self {
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

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn selected_script(&self) -> Option<&ScriptEntry> {
        self.list_state.selected().and_then(|i| self.scripts.get(i))
    }
}
