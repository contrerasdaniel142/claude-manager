use crate::i18n;
use crate::models::{AppState, Lang, Scope};
use serde_json::{Map, Value};

/// Splits a markdown file with optional YAML frontmatter into (yaml_block, body).
/// Frontmatter must start at byte 0 with a line that is exactly `---`.
pub fn split_frontmatter(raw: &str) -> (Option<String>, String) {
    let normalized = raw.replace("\r\n", "\n");
    if let Some(rest) = normalized.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            let yaml = rest[..end].to_string();
            let body = rest[end + "\n---\n".len()..].to_string();
            return (Some(yaml), body);
        }
        // Handle frontmatter that is the entire file (closing --- as last line, no trailing newline)
        if let Some(end) = rest.find("\n---") {
            if rest[end + 1..].trim() == "---" {
                let yaml = rest[..end].to_string();
                return (Some(yaml), String::new());
            }
        }
    }
    (None, normalized)
}

pub fn join_frontmatter(yaml: &str, body: &str) -> String {
    let trimmed_body = body.trim_start_matches('\n');
    format!("---\n{}\n---\n\n{}\n", yaml.trim_end(), trimmed_body.trim_end())
}

/// Reads a frontmatter field that Claude Code accepts as either a YAML list
/// or a delimiter-separated string — shared by `paths` on both rules and
/// skills, since the docs specify the exact same two accepted shapes.
pub fn extract_list_field(yaml: &str, key: &str, string_sep: char) -> Vec<String> {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(yaml) else {
        return vec![];
    };
    let Some(field_value) = value.get(key) else {
        return vec![];
    };
    match field_value {
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        serde_yaml::Value::String(s) => s
            .split(string_sep)
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect(),
        _ => vec![],
    }
}

/// Builds the YAML block for a list-shaped frontmatter field, always as a
/// YAML list regardless of which form it was read in — `None` when empty,
/// so the caller can omit the key entirely rather than write `key: []`.
pub fn build_list_frontmatter(key: &str, items: &[String]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let mut yaml = format!("{key}:");
    for item in items {
        yaml.push_str(&format!("\n  - {}", serde_yaml::to_string(item).unwrap_or_default().trim_end()));
    }
    Some(yaml)
}

pub fn settings_path(state: &AppState, scope: Scope) -> std::path::PathBuf {
    state.claude_dir(scope).join("settings.json")
}

/// Loads `settings.json` as a generic map so callers can read/write a single
/// top-level key (`hooks`, `permissions`, ...) without disturbing the rest
/// of the file — every other key the user or another tool wrote stays intact.
pub fn load_settings(lang: Lang, path: &std::path::Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    if raw.trim().is_empty() {
        return Ok(Map::new());
    }
    let value: Value = serde_json::from_str(&raw)
        .map_err(|e| i18n::settings_json_invalid(lang, &path.display().to_string(), &e.to_string()))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(i18n::settings_json_not_object(lang, &path.display().to_string())),
    }
}

/// Builds a `Command` for an external program the way every production
/// call site should — on Windows, the shipped release build has no
/// console of its own (see `main.rs`'s windows_subsystem), so spawning a
/// console program (claude, git, python) without this flag briefly flashes
/// a new console window per spawn. Invisible in `cargo run` dev builds
/// (which already have a console to share), only visible in the real
/// installed .exe.
pub fn silent_command(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}


/// Directory names we never want to recurse into when scanning a project for
/// CLAUDE.md files: dependency/build/VCS noise that can be huge or irrelevant.
pub const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".venv",
    "venv",
    "__pycache__",
    ".next",
    ".turbo",
];
