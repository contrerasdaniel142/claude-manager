use crate::fsutil::{build_list_frontmatter, extract_list_field, join_frontmatter, split_frontmatter};
use crate::i18n;
use crate::models::{AppState, SaveSkillInput, Scope, SkillFile, SkillFrontmatter, SkillSummary};
use crate::validate::{slugify, valid_slug, validate_skill};
use std::fs;
use tauri::State;

fn skills_dir(state: &AppState, scope: Scope) -> std::path::PathBuf {
    state.claude_dir(scope).join("skills")
}

fn extra_files(dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != "SKILL.md" {
                out.push(name);
            }
        }
    }
    out.sort();
    out
}

fn parse_skill_folder(id: &str, scope: Scope, dir: &std::path::Path) -> Option<SkillFile> {
    let skill_md = dir.join("SKILL.md");
    if !skill_md.exists() {
        return None;
    }
    let raw = fs::read_to_string(&skill_md).ok()?;
    let (yaml, body) = split_frontmatter(&raw);
    let fm: SkillFrontmatter = yaml
        .as_deref()
        .and_then(|y| serde_yaml::from_str(y).ok())
        .unwrap_or_default();
    let paths = yaml.as_deref().map(|y| extract_list_field(y, "paths", ',')).unwrap_or_default();
    Some(SkillFile {
        id: id.to_string(),
        scope,
        name: fm.name.unwrap_or_else(|| id.to_string()),
        description: fm.description.unwrap_or_default(),
        when_to_use: fm.when_to_use,
        allowed_tools: fm.allowed_tools,
        paths,
        argument_hint: fm.argument_hint,
        arguments: fm.arguments,
        disable_model_invocation: fm.disable_model_invocation,
        user_invocable: fm.user_invocable,
        disallowed_tools: fm.disallowed_tools,
        model: fm.model,
        effort: fm.effort,
        context: fm.context,
        agent: fm.agent,
        background: fm.background,
        hooks: fm.hooks,
        shell: fm.shell,
        metadata: fm.metadata,
        license: fm.license,
        compatibility: fm.compatibility,
        body: body.trim_start_matches('\n').to_string(),
        extra_files: extra_files(dir),
    })
}

#[tauri::command]
pub fn list_skills(state: State<AppState>, scope: Scope) -> Result<Vec<SkillSummary>, String> {
    let dir = skills_dir(&state, scope);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = path.file_name().unwrap().to_string_lossy().to_string();
        if let Some(skill) = parse_skill_folder(&id, scope, &path) {
            let errors = validate_skill(state.lang(), &skill.name, &skill.description);
            out.push(SkillSummary {
                id: id.clone(),
                name: skill.name,
                description: skill.description,
                scope,
                valid: errors.is_empty(),
                errors,
                extra_files: skill.extra_files,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[tauri::command]
pub fn read_skill(state: State<AppState>, scope: Scope, id: String) -> Result<SkillFile, String> {
    let dir = skills_dir(&state, scope).join(&id);
    parse_skill_folder(&id, scope, &dir).ok_or_else(|| i18n::no_skill_md_found(state.lang(), &id))
}

#[tauri::command]
pub fn save_skill(state: State<AppState>, input: SaveSkillInput) -> Result<SkillSummary, String> {
    save_skill_impl(&state, input)
}

fn save_skill_impl(state: &AppState, input: SaveSkillInput) -> Result<SkillSummary, String> {
    let SaveSkillInput {
        scope,
        original_id,
        name,
        description,
        when_to_use,
        allowed_tools,
        paths,
        argument_hint,
        arguments,
        disable_model_invocation,
        user_invocable,
        disallowed_tools,
        model,
        effort,
        context,
        agent,
        background,
        hooks,
        shell,
        metadata,
        license,
        compatibility,
        body,
    } = input;

    let errors = validate_skill(state.lang(), &name, &description);
    if !errors.is_empty() {
        return Err(errors.join(" "));
    }
    let base = skills_dir(state, scope);
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;

    let slug = if name.trim().is_empty() {
        original_id.clone().unwrap_or_else(|| "nueva-skill".into())
    } else if valid_slug(&name) {
        name.clone()
    } else {
        slugify(&name)
    };
    let new_dir = base.join(&slug);

    let is_same_item = original_id.as_deref() == Some(slug.as_str());
    if new_dir.exists() && !is_same_item {
        return Err(i18n::skill_already_exists(state.lang(), &slug));
    }

    if let Some(old_id) = &original_id {
        if old_id != &slug {
            let old_dir = base.join(old_id);
            if old_dir.exists() {
                fs::rename(&old_dir, &new_dir).map_err(|e| e.to_string())?;
            }
        }
    }
    fs::create_dir_all(&new_dir).map_err(|e| e.to_string())?;

    let fm = SkillFrontmatter {
        name: if name.trim().is_empty() { None } else { Some(slug.clone()) },
        description: Some(description.clone()),
        when_to_use: when_to_use.filter(|s| !s.trim().is_empty()),
        allowed_tools: allowed_tools.filter(|s| !s.trim().is_empty()),
        argument_hint: argument_hint.filter(|s| !s.trim().is_empty()),
        arguments: arguments.filter(|s| !s.trim().is_empty()),
        disable_model_invocation,
        user_invocable,
        disallowed_tools: disallowed_tools.filter(|s| !s.trim().is_empty()),
        model: model.filter(|s| !s.trim().is_empty()),
        effort: effort.filter(|s| !s.trim().is_empty()),
        context: context.filter(|s| !s.trim().is_empty()),
        agent: agent.filter(|s| !s.trim().is_empty()),
        background,
        hooks,
        shell: shell.filter(|s| !s.trim().is_empty()),
        metadata,
        license: license.filter(|s| !s.trim().is_empty()),
        compatibility: compatibility.filter(|s| !s.trim().is_empty()),
    };
    let mut yaml = serde_yaml::to_string(&fm).map_err(|e| e.to_string())?;
    // "paths" vive aparte de SkillFrontmatter (ver fsutil::build_list_frontmatter):
    // necesita poder leerse como lista YAML o string con comas, algo que un
    // campo tipado del struct no puede representar, así que se concatena.
    if let Some(paths_yaml) = build_list_frontmatter("paths", &paths) {
        yaml.push('\n');
        yaml.push_str(&paths_yaml);
    }
    let contents = join_frontmatter(&yaml, &body);
    fs::write(new_dir.join("SKILL.md"), contents).map_err(|e| e.to_string())?;

    Ok(SkillSummary {
        id: slug.clone(),
        name: slug,
        description,
        scope,
        valid: true,
        errors: vec![],
        extra_files: extra_files(&new_dir),
    })
}

#[tauri::command]
pub fn delete_skill(state: State<AppState>, scope: Scope, id: String) -> Result<(), String> {
    let dir = skills_dir(&state, scope).join(&id);
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())
}

/// Moves the whole skill folder — not just SKILL.md — to the other scope,
/// so extra reference files survive the move instead of being silently
/// dropped (save-to-new-scope-then-delete would lose them, since deleting
/// removes the entire folder).
#[tauri::command]
pub fn move_skill(state: State<AppState>, from_scope: Scope, to_scope: Scope, id: String) -> Result<SkillSummary, String> {
    move_skill_impl(&state, from_scope, to_scope, id)
}

fn move_skill_impl(state: &AppState, from_scope: Scope, to_scope: Scope, id: String) -> Result<SkillSummary, String> {
    let lang = state.lang();
    let src = skills_dir(state, from_scope).join(&id);
    if !src.join("SKILL.md").exists() {
        return Err(i18n::read_file_failed(lang, &id, "SKILL.md not found"));
    }
    let dest_base = skills_dir(state, to_scope);
    fs::create_dir_all(&dest_base).map_err(|e| e.to_string())?;
    let dest = dest_base.join(&id);
    if dest.exists() {
        return Err(i18n::skill_already_exists(lang, &id));
    }

    crate::skill_import::copy_dir_recursive(&src, &dest).map_err(|e| e.to_string())?;
    fs::remove_dir_all(&src).map_err(|e| e.to_string())?;

    let skill = parse_skill_folder(&id, to_scope, &dest).ok_or_else(|| i18n::read_file_failed(lang, &id, "SKILL.md not found"))?;
    let errors = validate_skill(lang, &skill.name, &skill.description);
    Ok(SkillSummary {
        id,
        name: skill.name,
        description: skill.description,
        scope: to_scope,
        valid: errors.is_empty(),
        errors,
        extra_files: skill.extra_files,
    })
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

    fn save(state: &AppState, original_id: Option<&str>, name: &str, description: &str, body: &str) -> Result<SkillSummary, String> {
        save_skill_impl(
            state,
            SaveSkillInput {
                scope: Scope::Project,
                original_id: original_id.map(|s| s.to_string()),
                name: name.to_string(),
                description: description.to_string(),
                when_to_use: None,
                allowed_tools: None,
                paths: vec![],
                argument_hint: None,
                arguments: None,
                disable_model_invocation: None,
                user_invocable: None,
                disallowed_tools: None,
                model: None,
                effort: None,
                context: None,
                agent: None,
                background: None,
                hooks: None,
                shell: None,
                metadata: None,
                license: None,
                compatibility: None,
                body: body.to_string(),
            },
        )
    }

    #[test]
    fn creating_a_new_skill_with_a_colliding_name_is_rejected_not_overwritten() {
        let (_g, state) = fake_state();
        save(&state, None, "api-design", "original description", "original body")
            .expect("first create should succeed");

        let err = save(&state, None, "api-design", "a totally different description", "overwritten body")
            .expect_err("creating a second skill with the same name must be rejected");
        assert!(err.contains("already exists"));

        // The original must survive untouched.
        let on_disk = fs::read_to_string(
            skills_dir(&state, Scope::Project).join("api-design").join("SKILL.md"),
        )
        .unwrap();
        assert!(on_disk.contains("original description"));
        assert!(on_disk.contains("original body"));
    }

    #[test]
    fn editing_a_skill_and_keeping_its_name_still_works() {
        let (_g, state) = fake_state();
        save(&state, None, "api-design", "v1", "v1 body").unwrap();
        save(&state, Some("api-design"), "api-design", "v2", "v2 body")
            .expect("editing the same skill in place must succeed");

        let on_disk = fs::read_to_string(
            skills_dir(&state, Scope::Project).join("api-design").join("SKILL.md"),
        )
        .unwrap();
        assert!(on_disk.contains("v2 body"));
    }

    #[test]
    fn renaming_onto_another_existing_skill_is_rejected() {
        let (_g, state) = fake_state();
        save(&state, None, "alpha", "a", "a-body").unwrap();
        save(&state, None, "beta", "b", "b-body").unwrap();

        let err = save(&state, Some("beta"), "alpha", "b renamed onto a", "should not land")
            .expect_err("renaming beta to the name alpha already uses must be rejected");
        assert!(err.contains("already exists"));

        let alpha_on_disk = fs::read_to_string(
            skills_dir(&state, Scope::Project).join("alpha").join("SKILL.md"),
        )
        .unwrap();
        assert!(alpha_on_disk.contains("a-body"), "alpha must be untouched");
    }

    #[test]
    fn round_trips_paths_and_new_extended_fields() {
        let (_g, state) = fake_state();
        save_skill_impl(
            &state,
            SaveSkillInput {
                scope: Scope::Project,
                original_id: None,
                name: "deploy".into(),
                description: "Deploy the application".into(),
                when_to_use: None,
                allowed_tools: None,
                paths: vec!["src/**/*.ts".into(), "lib/**/*.ts".into()],
                argument_hint: Some("[env]".into()),
                arguments: Some("env".into()),
                disable_model_invocation: Some(true),
                user_invocable: Some(false),
                disallowed_tools: Some("Write".into()),
                model: Some("sonnet".into()),
                effort: Some("high".into()),
                context: Some("fork".into()),
                agent: Some("Explore".into()),
                background: Some(false),
                hooks: Some(serde_json::json!({"PreToolUse": []})),
                shell: Some("powershell".into()),
                metadata: Some(serde_json::json!({"team": "platform"})),
                license: Some("MIT".into()),
                compatibility: Some("Requires Node 18+".into()),
                body: "Deploy steps.".into(),
            },
        )
        .expect("save with every extended field should succeed");

        let parsed = parse_skill_folder("deploy", Scope::Project, &skills_dir(&state, Scope::Project).join("deploy"))
            .expect("should parse back");
        assert_eq!(parsed.paths, vec!["src/**/*.ts".to_string(), "lib/**/*.ts".to_string()]);
        assert_eq!(parsed.argument_hint.as_deref(), Some("[env]"));
        assert_eq!(parsed.arguments.as_deref(), Some("env"));
        assert_eq!(parsed.disable_model_invocation, Some(true));
        assert_eq!(parsed.user_invocable, Some(false));
        assert_eq!(parsed.disallowed_tools.as_deref(), Some("Write"));
        assert_eq!(parsed.model.as_deref(), Some("sonnet"));
        assert_eq!(parsed.effort.as_deref(), Some("high"));
        assert_eq!(parsed.context.as_deref(), Some("fork"));
        assert_eq!(parsed.agent.as_deref(), Some("Explore"));
        assert_eq!(parsed.background, Some(false));
        assert!(parsed.hooks.is_some());
        assert_eq!(parsed.shell.as_deref(), Some("powershell"));
        assert_eq!(
            parsed.metadata.and_then(|v| v.get("team").cloned()),
            Some(serde_json::json!("platform"))
        );
        assert_eq!(parsed.license.as_deref(), Some("MIT"));
        assert_eq!(parsed.compatibility.as_deref(), Some("Requires Node 18+"));
    }

    #[test]
    fn moving_a_skill_carries_its_extra_files_and_removes_it_from_the_old_scope() {
        let (_g, state) = fake_state();
        save(&state, None, "with-reference", "has an extra file", "body").unwrap();
        let dir = skills_dir(&state, Scope::Project).join("with-reference");
        fs::write(dir.join("reference.md"), "extra content").unwrap();

        let moved = move_skill_impl(&state, Scope::Project, Scope::Global, "with-reference".into()).unwrap();
        assert_eq!(moved.scope, Scope::Global);

        assert!(!skills_dir(&state, Scope::Project).join("with-reference").exists());
        let new_dir = skills_dir(&state, Scope::Global).join("with-reference");
        assert!(new_dir.join("SKILL.md").exists());
        assert_eq!(fs::read_to_string(new_dir.join("reference.md")).unwrap(), "extra content");
    }

    #[test]
    fn moving_onto_an_existing_skill_in_the_target_scope_is_rejected() {
        let (_g, state) = fake_state();
        save(&state, None, "dup", "desc", "body").unwrap();
        save_skill_impl(
            &state,
            SaveSkillInput {
                scope: Scope::Global,
                original_id: None,
                name: "dup".into(),
                description: "desc".into(),
                when_to_use: None,
                allowed_tools: None,
                paths: vec![],
                argument_hint: None,
                arguments: None,
                disable_model_invocation: None,
                user_invocable: None,
                disallowed_tools: None,
                model: None,
                effort: None,
                context: None,
                agent: None,
                background: None,
                hooks: None,
                shell: None,
                metadata: None,
                license: None,
                compatibility: None,
                body: "body".into(),
            },
        )
        .unwrap();

        let err = move_skill_impl(&state, Scope::Project, Scope::Global, "dup".into())
            .expect_err("moving onto an existing skill in the target scope must be rejected");
        assert!(err.contains("already exists") || err.contains("Ya existe"));
    }
}
