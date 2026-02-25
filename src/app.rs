use std::path::PathBuf;

use ratatui::widgets::ListState;

// ── Argument model ────────────────────────────────────────────────────────────

/// A single argument declared by a script via `# @arg:`.
#[derive(Clone, Debug)]
pub struct ScriptArg {
    /// Shell variable name (no spaces), e.g. `output_dir`.
    pub name: String,
    /// Human-readable label shown in the form, e.g. `Output Directory`.
    pub label: String,
    /// Whether the field must be non-empty before the script can run.
    pub required: bool,
    /// Pre-filled default value (may be empty).
    pub default: String,
}

// ── Script entry ──────────────────────────────────────────────────────────────

/// A single script discovered on disk.
#[derive(Clone)]
pub struct ScriptEntry {
    pub name: String,
    pub description: String,
    pub category: String,
    /// Absolute path to the script file on disk.
    pub path: PathBuf,
    /// Arguments declared in the script header.
    pub args: Vec<ScriptArg>,
}

// ── Dialog state ──────────────────────────────────────────────────────────────

/// Which dialog (if any) is currently open.
pub enum DialogState {
    /// No dialog open.
    None,
    /// Simple yes/no confirm (scripts with no declared arguments).
    Confirm,
    /// Argument-input form.
    ArgsForm {
        /// Index of the currently focused field (0 = first arg).
        focused_field: usize,
        /// One entry per `ScriptEntry::args`, containing the current text.
        values: Vec<String>,
    },
}

// ── App ───────────────────────────────────────────────────────────────────────

/// Top-level application state.
pub struct App {
    pub scripts: Vec<ScriptEntry>,
    /// Selection index into `scripts` (excludes category-header rows).
    pub list_state: ListState,
    pub dialog: DialogState,
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
            dialog: DialogState::None,
            last_message: None,
            load_warnings,
            scripts_dir,
        }
    }

    /// Open the appropriate dialog for the currently selected script.
    pub fn open_dialog(&mut self) {
        if let Some(script) = self.selected_script() {
            if script.args.is_empty() {
                self.dialog = DialogState::Confirm;
            } else {
                let values = script
                    .args
                    .iter()
                    .map(|a| a.default.clone())
                    .collect();
                self.dialog = DialogState::ArgsForm {
                    focused_field: 0,
                    values,
                };
            }
        }
    }

    pub fn close_dialog(&mut self) {
        self.dialog = DialogState::None;
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
