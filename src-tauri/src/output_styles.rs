use crate::fsutil::{join_frontmatter, split_frontmatter};
use crate::i18n;
use crate::models::{AppState, OutputStyleFile, OutputStyleFrontmatter, OutputStyleSummary, Scope};
use crate::validate::slugify;
use std::fs;
use std::path::PathBuf;
use tauri::State;

fn output_styles_dir(state: &AppState, scope: Scope) -> PathBuf {
    state.claude_dir(scope).join("output-styles")
}

fn parse_output_style_file(id: &str, scope: Scope, raw: &str) -> OutputStyleFile {
    let (yaml, body) = split_frontmatter(raw);
    let fm: OutputStyleFrontmatter = yaml
        .as_deref()
        .and_then(|y| serde_yaml::from_str(y).ok())
        .unwrap_or_default();
    OutputStyleFile {
        id: id.to_string(),
        scope,
        name: fm.name.unwrap_or_else(|| id.trim_end_matches(".md").to_string()),
        description: fm.description.unwrap_or_default(),
        keep_coding_instructions: fm.keep_coding_instructions.unwrap_or(false),
        body: body.trim_start_matches('\n').to_string(),
    }
}

fn build_frontmatter(fm: &OutputStyleFrontmatter) -> Option<String> {
    if fm.name.is_none() && fm.description.is_none() && fm.keep_coding_instructions.is_none() {
        return None;
    }
    serde_yaml::to_string(fm).ok()
}

#[tauri::command]
pub fn list_output_styles(state: State<AppState>, scope: Scope) -> Result<Vec<OutputStyleSummary>, String> {
    let dir = output_styles_dir(&state, scope);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let id = path.file_name().unwrap().to_string_lossy().to_string();
        let raw = fs::read_to_string(&path).unwrap_or_default();
        let style = parse_output_style_file(&id, scope, &raw);
        out.push(OutputStyleSummary {
            id,
            scope,
            name: style.name,
            description: style.description,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[tauri::command]
pub fn read_output_style(state: State<AppState>, scope: Scope, id: String) -> Result<OutputStyleFile, String> {
    let path = output_styles_dir(&state, scope).join(&id);
    let raw = fs::read_to_string(&path).map_err(|e| i18n::read_file_failed(state.lang(), &id, &e.to_string()))?;
    Ok(parse_output_style_file(&id, scope, &raw))
}

#[tauri::command]
pub fn save_output_style(
    state: State<AppState>,
    scope: Scope,
    original_id: Option<String>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    keep_coding_instructions: bool,
    body: String,
) -> Result<OutputStyleSummary, String> {
    save_output_style_impl(&state, scope, original_id, id, name, description, keep_coding_instructions, body)
}

fn save_output_style_impl(
    state: &AppState,
    scope: Scope,
    original_id: Option<String>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    keep_coding_instructions: bool,
    body: String,
) -> Result<OutputStyleSummary, String> {
    let dir = output_styles_dir(state, scope);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let trimmed = id.trim();
    let final_id = if trimmed.is_empty() {
        original_id.clone().unwrap_or_else(|| "nuevo-estilo.md".into())
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
        return Err(i18n::output_style_already_exists(state.lang(), &final_id));
    }

    if let Some(old_id) = &original_id {
        if old_id != &final_id {
            let old_path = dir.join(old_id);
            if old_path.exists() {
                fs::remove_file(&old_path).map_err(|e| e.to_string())?;
            }
        }
    }

    let canon_parent = new_path.parent().unwrap().canonicalize().map_err(|e| e.to_string())?;
    if !canon_parent.starts_with(&canon_dir) {
        return Err(i18n::path_outside_output_styles_dir(state.lang()));
    }

    let fm = OutputStyleFrontmatter {
        name: name.filter(|s| !s.trim().is_empty()),
        description: description.filter(|s| !s.trim().is_empty()),
        keep_coding_instructions: if keep_coding_instructions { Some(true) } else { None },
    };
    let contents = match build_frontmatter(&fm) {
        Some(yaml) => join_frontmatter(&yaml, &body),
        None => format!("{}\n", body.trim_end()),
    };
    fs::write(&new_path, contents).map_err(|e| e.to_string())?;

    Ok(OutputStyleSummary {
        id: final_id.clone(),
        scope,
        name: fm.name.unwrap_or_else(|| final_id.trim_end_matches(".md").to_string()),
        description: fm.description.unwrap_or_default(),
    })
}

#[tauri::command]
pub fn delete_output_style(state: State<AppState>, scope: Scope, id: String) -> Result<(), String> {
    let path = output_styles_dir(&state, scope).join(&id);
    fs::remove_file(&path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Lang;

    fn fake_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().join("project");
        let global_root = dir.path().join("global");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&global_root).unwrap();
        (dir, AppState::new(project_root, global_root))
    }

    #[test]
    fn creating_a_new_style_with_a_colliding_name_is_rejected_not_overwritten() {
        let (_g, state) = fake_state();
        save_output_style_impl(&state, Scope::Project, None, "concise2".into(), None, None, false, "v1".into())
            .unwrap();

        let err = save_output_style_impl(
            &state,
            Scope::Project,
            None,
            "concise2".into(),
            None,
            None,
            false,
            "v2".into(),
        )
        .expect_err("creating a second style with the same name must be rejected");
        assert!(err.contains("already exists"));
    }

    #[test]
    fn editing_a_style_and_keeping_its_name_still_works() {
        let (_g, state) = fake_state();
        save_output_style_impl(&state, Scope::Project, None, "diagrams".into(), None, None, false, "v1".into())
            .unwrap();
        save_output_style_impl(
            &state,
            Scope::Project,
            Some("diagrams.md".into()),
            "diagrams".into(),
            Some("Diagrams first".into()),
            Some("Lead with a diagram".into()),
            true,
            "v2".into(),
        )
        .expect("editing the same style in place must succeed");

        let on_disk = fs::read_to_string(output_styles_dir(&state, Scope::Project).join("diagrams.md")).unwrap();
        assert!(on_disk.contains("v2"));
        assert!(on_disk.contains("keep-coding-instructions: true"));
    }

    #[test]
    fn round_trips_name_description_and_keep_coding_instructions() {
        let raw = "---\nname: Diagrams first\ndescription: Lead every explanation with a diagram\nkeep-coding-instructions: true\n---\n\nBody text here.\n";
        let parsed = parse_output_style_file("diagrams.md", Scope::Project, raw);
        assert_eq!(parsed.name, "Diagrams first");
        assert_eq!(parsed.description, "Lead every explanation with a diagram");
        assert!(parsed.keep_coding_instructions);
        assert_eq!(parsed.body, "Body text here.\n");
    }

    #[test]
    fn style_without_frontmatter_falls_back_to_filename_as_name() {
        let raw = "Just instructions, no frontmatter.\n";
        let parsed = parse_output_style_file("terse.md", Scope::Project, raw);
        assert_eq!(parsed.name, "terse");
        assert_eq!(parsed.description, "");
        assert!(!parsed.keep_coding_instructions);
        let _ = Lang::En;
    }

    #[test]
    fn save_then_list_round_trip() {
        let (_g, state) = fake_state();
        save_output_style_impl(
            &state,
            Scope::Project,
            None,
            "explanatory2".into(),
            Some("Explanatory 2".into()),
            Some("More insights".into()),
            false,
            "Explain everything.".into(),
        )
        .unwrap();

        let dir = output_styles_dir(&state, Scope::Project);
        let raw = fs::read_to_string(dir.join("explanatory2.md")).unwrap();
        let parsed = parse_output_style_file("explanatory2.md", Scope::Project, &raw);
        assert_eq!(parsed.name, "Explanatory 2");
        assert_eq!(parsed.description, "More insights");
        assert!(parsed.body.contains("Explain everything."));
    }
}
