use crate::fsutil::SKIP_DIRS;
use crate::i18n;
use crate::models::{AppState, Lang, Scope, SteeringFile, SteeringSummary};
use std::fs;
use std::path::PathBuf;
use tauri::State;
use walkdir::WalkDir;

fn title_from_content(lang: Lang, content: &str) -> String {
    for line in content.lines() {
        let line = line.trim();
        if let Some(h) = line.strip_prefix("# ") {
            return h.trim().to_string();
        }
        if !line.is_empty() {
            return line.chars().take(80).collect();
        }
    }
    i18n::empty_placeholder(lang)
}

#[tauri::command]
pub fn list_steering(state: State<AppState>, scope: Scope) -> Result<Vec<SteeringSummary>, String> {
    match scope {
        Scope::Global => {
            let path = state.global_root.join("CLAUDE.md");
            if !path.exists() {
                return Ok(vec![]);
            }
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            Ok(vec![SteeringSummary {
                rel_path: "CLAUDE.md".into(),
                scope,
                title: title_from_content(state.lang(), &content),
                size: meta.len(),
            }])
        }
        Scope::Project => {
            let root = state.project_root();
            let mut out = Vec::new();
            let walker = WalkDir::new(&root).into_iter().filter_entry(|e| {
                if e.depth() == 0 {
                    return true;
                }
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    return !SKIP_DIRS.contains(&name.as_ref()) && !name.starts_with('.');
                }
                true
            });
            for entry in walker.filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && entry.file_name() == "CLAUDE.md" {
                    let rel = entry
                        .path()
                        .strip_prefix(&root)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .replace('\\', "/");
                    let content = fs::read_to_string(entry.path()).unwrap_or_default();
                    let meta = entry.metadata().map_err(|e| e.to_string())?;
                    out.push(SteeringSummary {
                        rel_path: rel,
                        scope,
                        title: title_from_content(state.lang(), &content),
                        size: meta.len(),
                    });
                }
            }
            out.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
            Ok(out)
        }
    }
}

fn resolve_path(state: &AppState, scope: Scope, rel_path: &str) -> Result<PathBuf, String> {
    let base: PathBuf = match scope {
        Scope::Global => state.global_root.clone(),
        Scope::Project => state.project_root(),
    };
    let lang = state.lang();
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    let candidate = base.join(rel_path);
    let file_name = candidate
        .file_name()
        .ok_or_else(|| i18n::invalid_path(lang))?
        .to_owned();
    let parent = candidate
        .parent()
        .ok_or_else(|| i18n::invalid_path(lang))?;
    fs::create_dir_all(parent).map_err(|e| i18n::create_dir_failed(lang, &parent.display().to_string(), &e.to_string()))?;

    // Canonicalize two directories that are guaranteed to exist (base and the
    // target's parent) instead of the target file itself, which may not
    // exist yet on a `create` — canonicalize() errors on missing paths.
    let canon_base = base
        .canonicalize()
        .map_err(|e| i18n::resolve_path_failed(lang, &base.display().to_string(), &e.to_string()))?;
    let canon_parent = parent
        .canonicalize()
        .map_err(|e| i18n::resolve_path_failed(lang, &parent.display().to_string(), &e.to_string()))?;
    if !canon_parent.starts_with(&canon_base) {
        return Err(i18n::path_outside_allowed_dir(lang));
    }
    Ok(canon_parent.join(file_name))
}

#[tauri::command]
pub fn read_steering(
    state: State<AppState>,
    scope: Scope,
    rel_path: String,
) -> Result<SteeringFile, String> {
    let path = resolve_path(&state, scope, &rel_path)?;
    let content = fs::read_to_string(&path).map_err(|e| i18n::read_file_failed(state.lang(), &rel_path, &e.to_string()))?;
    Ok(SteeringFile { rel_path, scope, content })
}

#[tauri::command]
pub fn save_steering(
    state: State<AppState>,
    scope: Scope,
    rel_path: String,
    content: String,
) -> Result<SteeringFile, String> {
    let path = resolve_path(&state, scope, &rel_path)?;
    fs::write(&path, &content).map_err(|e| e.to_string())?;
    Ok(SteeringFile { rel_path, scope, content })
}

#[tauri::command]
pub fn delete_steering(
    state: State<AppState>,
    scope: Scope,
    rel_path: String,
) -> Result<(), String> {
    let path = resolve_path(&state, scope, &rel_path)?;
    fs::remove_file(&path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().join("project");
        let global_root = dir.path().join("global");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&global_root).unwrap();
        let state = AppState::new(project_root, global_root);
        (dir, state)
    }

    #[test]
    fn creates_root_claude_md() {
        let (_guard, state) = fake_state();
        let path = resolve_path(&state, Scope::Project, "CLAUDE.md").expect("root create should work");
        assert_eq!(path.file_name().unwrap(), "CLAUDE.md");
        assert!(path.starts_with(state.project_root().canonicalize().unwrap()));
    }

    #[test]
    fn creates_nested_claude_md_in_new_subfolder() {
        let (_guard, state) = fake_state();
        let path = resolve_path(&state, Scope::Project, "services/backend/CLAUDE.md")
            .expect("nested create should work even though the folder doesn't exist yet");
        assert!(path.ends_with("CLAUDE.md"));
        assert!(path.parent().unwrap().is_dir());
    }

    #[test]
    fn creates_a_second_distinct_file_after_the_first() {
        let (_guard, state) = fake_state();
        let first = resolve_path(&state, Scope::Project, "CLAUDE.md").unwrap();
        fs::write(&first, "uno").unwrap();
        let second = resolve_path(&state, Scope::Project, "frontend/CLAUDE.md")
            .expect("creating a second file at a different path must not fail");
        fs::write(&second, "dos").unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read_to_string(&first).unwrap(), "uno");
        assert_eq!(fs::read_to_string(&second).unwrap(), "dos");
    }

    #[test]
    fn rejects_path_traversal_outside_the_project_root() {
        let (_guard, state) = fake_state();
        let result = resolve_path(&state, Scope::Project, "../../evil.md");
        assert!(result.is_err(), "escaping the project root must be rejected");
    }

    #[test]
    fn global_scope_always_resolves_to_global_root_claude_md() {
        let (_guard, state) = fake_state();
        let path = resolve_path(&state, Scope::Global, "CLAUDE.md").unwrap();
        assert!(path.starts_with(state.global_root.canonicalize().unwrap()));
    }
}
