use crate::fsutil::{build_list_frontmatter, extract_list_field, join_frontmatter, split_frontmatter};
use crate::i18n;
use crate::models::{AppState, Lang, RuleFile, RuleSummary, Scope};
use crate::validate::slugify;
use std::fs;
use std::path::PathBuf;
use tauri::State;

fn rules_dir(state: &AppState, scope: Scope) -> PathBuf {
    state.claude_dir(scope).join("rules")
}

fn title_from_body(lang: Lang, body: &str) -> String {
    for line in body.lines() {
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

fn parse_rule_file(lang: Lang, id: &str, scope: Scope, raw: &str) -> RuleFile {
    let (yaml, body) = split_frontmatter(raw);
    let paths = yaml.as_deref().map(|y| extract_list_field(y, "paths", ',')).unwrap_or_default();
    let body = body.trim_start_matches('\n').to_string();
    RuleFile {
        id: id.to_string(),
        scope,
        paths,
        title: title_from_body(lang, &body),
        body,
    }
}

#[tauri::command]
pub fn list_rules(state: State<AppState>, scope: Scope) -> Result<Vec<RuleSummary>, String> {
    let dir = rules_dir(&state, scope);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let id = entry
            .path()
            .strip_prefix(&dir)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        let raw = fs::read_to_string(entry.path()).unwrap_or_default();
        let rule = parse_rule_file(state.lang(), &id, scope, &raw);
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        out.push(RuleSummary {
            id,
            scope,
            title: rule.title,
            paths: rule.paths,
            size: meta.len(),
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[tauri::command]
pub fn read_rule(state: State<AppState>, scope: Scope, id: String) -> Result<RuleFile, String> {
    let path = rules_dir(&state, scope).join(&id);
    let raw = fs::read_to_string(&path).map_err(|e| i18n::read_file_failed(state.lang(), &id, &e.to_string()))?;
    Ok(parse_rule_file(state.lang(), &id, scope, &raw))
}

#[tauri::command]
pub fn save_rule(
    state: State<AppState>,
    scope: Scope,
    original_id: Option<String>,
    id: String,
    paths: Vec<String>,
    body: String,
) -> Result<RuleSummary, String> {
    save_rule_impl(&state, scope, original_id, id, paths, body)
}

fn save_rule_impl(
    state: &AppState,
    scope: Scope,
    original_id: Option<String>,
    id: String,
    paths: Vec<String>,
    body: String,
) -> Result<RuleSummary, String> {
    let dir = rules_dir(state, scope);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let trimmed = id.trim();
    let final_id = if trimmed.is_empty() {
        original_id.clone().unwrap_or_else(|| "nueva-regla.md".into())
    } else if trimmed.ends_with(".md") {
        trimmed.to_string()
    } else {
        format!("{}.md", slugify(trimmed))
    };

    let new_path = dir.join(&final_id);
    let canon_dir = dir.canonicalize().map_err(|e| e.to_string())?;
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let is_same_item = original_id.as_deref() == Some(final_id.as_str());
    if new_path.exists() && !is_same_item {
        return Err(i18n::rule_already_exists(state.lang(), &final_id));
    }

    if let Some(old_id) = &original_id {
        if old_id != &final_id {
            let old_path = dir.join(old_id);
            if old_path.exists() {
                fs::remove_file(&old_path).map_err(|e| e.to_string())?;
            }
        }
    }

    let canon_parent = new_path
        .parent()
        .unwrap()
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !canon_parent.starts_with(&canon_dir) {
        return Err(i18n::path_outside_rules_dir(state.lang()));
    }

    let contents = match build_list_frontmatter("paths", &paths) {
        Some(yaml) => join_frontmatter(&yaml, &body),
        None => format!("{}\n", body.trim_end()),
    };
    fs::write(&new_path, contents).map_err(|e| e.to_string())?;

    Ok(RuleSummary {
        id: final_id,
        scope,
        title: title_from_body(state.lang(), &body),
        paths,
        size: fs::metadata(&new_path).map(|m| m.len()).unwrap_or(0),
    })
}

#[tauri::command]
pub fn delete_rule(state: State<AppState>, scope: Scope, id: String) -> Result<(), String> {
    let path = rules_dir(&state, scope).join(&id);
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
        (dir, AppState::new(project_root, global_root))
    }

    #[test]
    fn creating_a_new_rule_with_a_colliding_name_is_rejected_not_overwritten() {
        let (_g, state) = fake_state();
        save_rule_impl(&state, Scope::Project, None, "arch".into(), vec![], "original body".into())
            .unwrap();

        let err = save_rule_impl(&state, Scope::Project, None, "arch".into(), vec![], "overwritten".into())
            .expect_err("creating a second rule with the same name must be rejected");
        assert!(err.contains("already exists"));

        let on_disk = fs::read_to_string(rules_dir(&state, Scope::Project).join("arch.md")).unwrap();
        assert!(on_disk.contains("original body"));
    }

    #[test]
    fn editing_a_rule_and_keeping_its_name_still_works() {
        let (_g, state) = fake_state();
        save_rule_impl(&state, Scope::Project, None, "arch".into(), vec![], "v1".into()).unwrap();
        save_rule_impl(&state, Scope::Project, Some("arch.md".into()), "arch".into(), vec![], "v2".into())
            .expect("editing the same rule in place must succeed");
        let on_disk = fs::read_to_string(rules_dir(&state, Scope::Project).join("arch.md")).unwrap();
        assert!(on_disk.contains("v2"));
    }

    #[test]
    fn round_trips_paths_frontmatter_as_yaml_list() {
        let (_g, state) = fake_state();
        let raw = "---\npaths:\n  - \"src/api/**/*.ts\"\n  - \"src/web/**/*.tsx\"\n---\n\n# API rules\nUse Zod.\n";
        let parsed = parse_rule_file(Lang::En, "api.md", Scope::Project, raw);
        assert_eq!(parsed.paths, vec!["src/api/**/*.ts", "src/web/**/*.tsx"]);
        assert_eq!(parsed.title, "API rules");
        let _ = state;
    }

    #[test]
    fn round_trips_paths_frontmatter_as_comma_string() {
        let raw = "---\npaths: \"*.java, *.ts\"\n---\n\nhexagonal arch\n";
        let parsed = parse_rule_file(Lang::En, "arch.md", Scope::Project, raw);
        assert_eq!(parsed.paths, vec!["*.java", "*.ts"]);
    }

    #[test]
    fn rule_without_paths_loads_unconditionally() {
        let raw = "# Always loaded\nno frontmatter here\n";
        let parsed = parse_rule_file(Lang::En, "global.md", Scope::Project, raw);
        assert!(parsed.paths.is_empty());
    }

    #[test]
    fn save_then_list_round_trip() {
        let (_g, state) = fake_state();
        let dir = rules_dir(&state, Scope::Project);
        fs::create_dir_all(&dir).unwrap();
        let contents = join_frontmatter("paths:\n  - \"**/*.java\"", "# Hexagonal\nDomain never imports infra.\n");
        fs::write(dir.join("hexagonal.md"), contents).unwrap();

        let raw = fs::read_to_string(dir.join("hexagonal.md")).unwrap();
        let parsed = parse_rule_file(Lang::En, "hexagonal.md", Scope::Project, &raw);
        assert_eq!(parsed.paths, vec!["**/*.java"]);
        assert!(parsed.body.contains("Domain never imports infra."));
    }
}
