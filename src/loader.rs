use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::app::{ScriptArg, ScriptEntry};

/// Resolve the `scripts.d` directory.  Searches:
///   1. Next to the running binary
///   2. Current working directory
pub fn find_scripts_dir() -> PathBuf {
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
/// Recognised tags (anywhere in the first 60 lines):
/// ```text
/// # @name:        My Script
/// # @description: What it does
/// # @category:    Category Name
/// # @arg: var_name | Human Label | required | default_value
/// ```
///
/// `@arg` format (pipe-separated, trailing fields optional):
/// - field 1: variable name  (no spaces, used as env var / positional)
/// - field 2: label          (shown in the form)
/// - field 3: `required` or `optional` (default: optional)
/// - field 4: default value  (default: empty)
///
/// Returns an error string if `@name` is absent (required).
fn parse_script_metadata(path: &Path) -> Result<ScriptEntry, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;

    let mut name: Option<String> = None;
    let mut description = String::new();
    let mut category = String::from("Uncategorized");
    let mut args: Vec<ScriptArg> = Vec::new();

    for line in content.lines().take(60) {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("# @name:") {
            name = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("# @description:") {
            description = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("# @category:") {
            category = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("# @arg:") {
            if let Some(arg) = parse_arg_tag(val) {
                args.push(arg);
            }
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
        args,
    })
}

/// Parse a single `@arg:` value string.
///
/// Format: `var_name | Human Label | required | default_value`
/// Returns `None` if the name or label field is missing/empty.
fn parse_arg_tag(raw: &str) -> Option<ScriptArg> {
    let mut parts = raw.splitn(4, '|').map(str::trim);

    let name = parts.next().filter(|s| !s.is_empty())?.to_string();
    let label = parts.next().filter(|s| !s.is_empty())?.to_string();
    let required = parts
        .next()
        .map(|s| s.eq_ignore_ascii_case("required"))
        .unwrap_or(false);
    let default = parts.next().unwrap_or("").to_string();

    Some(ScriptArg {
        name,
        label,
        required,
        default,
    })
}

/// Load all scripts from `scripts_dir`, returning the sorted list and any
/// non-fatal warnings (e.g. files skipped due to missing tags).
pub fn load_scripts(scripts_dir: &Path) -> (Vec<ScriptEntry>, Vec<String>) {
    let mut scripts: Vec<ScriptEntry> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let read_dir = match fs::read_dir(scripts_dir) {
        Ok(rd) => rd,
        Err(e) => {
            warnings.push(format!("Cannot open {}: {}", scripts_dir.display(), e));
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

    // Sort by category, then by name (case-insensitive).
    scripts.sort_by(|a, b| {
        a.category
            .to_lowercase()
            .cmp(&b.category.to_lowercase())
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    (scripts, warnings)
}
