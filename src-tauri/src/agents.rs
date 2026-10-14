use crate::fsutil::{join_frontmatter, split_frontmatter};
use crate::i18n;
use crate::models::{AgentFile, AgentFrontmatter, AgentSummary, AppState, SaveAgentInput, Scope};
use crate::validate::{slugify, valid_slug, validate_agent, AGENT_COLORS, PERMISSION_MODES};
use std::fs;
use tauri::State;

fn agents_dir(state: &AppState, scope: Scope) -> std::path::PathBuf {
    state.claude_dir(scope).join("agents")
}

#[tauri::command]
pub fn list_permission_modes() -> Vec<&'static str> {
    PERMISSION_MODES.to_vec()
}

#[tauri::command]
pub fn list_agent_colors() -> Vec<&'static str> {
    AGENT_COLORS.to_vec()
}

fn parse_agent_file(id: &str, scope: Scope, raw: &str) -> AgentFile {
    let (yaml, body) = split_frontmatter(raw);
    let mut fm: AgentFrontmatter = yaml
        .as_deref()
        .and_then(|y| serde_yaml::from_str(y).ok())
        .unwrap_or_default();
    if fm.name.is_empty() {
        fm.name = id.to_string();
    }
    AgentFile {
        id: id.to_string(),
        scope,
        name: fm.name,
        description: fm.description,
        tools: fm.tools,
        model: fm.model,
        disallowed_tools: fm.disallowed_tools,
        permission_mode: fm.permission_mode,
        color: fm.color,
        max_turns: fm.max_turns,
        skills: fm.skills,
        memory: fm.memory,
        background: fm.background,
        omit_claude_md: fm.omit_claude_md,
        effort: fm.effort,
        isolation: fm.isolation,
        initial_prompt: fm.initial_prompt,
        experimental: fm.experimental,
        mcp_servers: fm.mcp_servers,
        hooks: fm.hooks,
        body: body.trim_start_matches('\n').to_string(),
    }
}

#[tauri::command]
pub fn list_agents(state: State<AppState>, scope: Scope) -> Result<Vec<AgentSummary>, String> {
    let dir = agents_dir(&state, scope);
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
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let agent = parse_agent_file(&id, scope, &raw);
        let errors = validate_agent(state.lang(), &agent.name, &agent.description);
        out.push(AgentSummary {
            id,
            name: agent.name,
            description: agent.description,
            scope,
            valid: errors.is_empty(),
            errors,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[tauri::command]
pub fn read_agent(state: State<AppState>, scope: Scope, id: String) -> Result<AgentFile, String> {
    let path = agents_dir(&state, scope).join(format!("{id}.md"));
    let raw = fs::read_to_string(&path).map_err(|e| i18n::read_file_failed(state.lang(), &format!("{id}.md"), &e.to_string()))?;
    Ok(parse_agent_file(&id, scope, &raw))
}

#[tauri::command]
pub fn save_agent(state: State<AppState>, input: SaveAgentInput) -> Result<AgentSummary, String> {
    save_agent_impl(&state, input)
}

fn save_agent_impl(state: &AppState, input: SaveAgentInput) -> Result<AgentSummary, String> {
    let SaveAgentInput {
        scope,
        original_id,
        name,
        description,
        tools,
        model,
        disallowed_tools,
        permission_mode,
        color,
        max_turns,
        skills,
        memory,
        background,
        omit_claude_md,
        effort,
        isolation,
        initial_prompt,
        experimental,
        mcp_servers,
        hooks,
        body,
    } = input;

    let errors = validate_agent(state.lang(), &name, &description);
    if !errors.is_empty() {
        return Err(errors.join(" "));
    }
    let dir = agents_dir(state, scope);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let slug = if valid_slug(&name) { name.clone() } else { slugify(&name) };
    let new_path = dir.join(format!("{slug}.md"));

    let is_same_item = original_id.as_deref() == Some(slug.as_str());
    if new_path.exists() && !is_same_item {
        return Err(i18n::agent_already_exists(state.lang(), &slug));
    }

    if let Some(old_id) = &original_id {
        if old_id != &slug {
            let old_path = dir.join(format!("{old_id}.md"));
            if old_path.exists() {
                fs::remove_file(&old_path).map_err(|e| e.to_string())?;
            }
        }
    }

    let fm = AgentFrontmatter {
        name: slug.clone(),
        description: description.clone(),
        tools: tools.filter(|t| !t.trim().is_empty()),
        model: model.filter(|m| !m.trim().is_empty()),
        disallowed_tools: disallowed_tools.filter(|t| !t.trim().is_empty()),
        permission_mode: permission_mode.filter(|p| !p.trim().is_empty()),
        color: color.filter(|c| !c.trim().is_empty()),
        max_turns,
        skills: skills.filter(|s| !s.is_empty()),
        memory: memory.filter(|m| !m.trim().is_empty()),
        background,
        omit_claude_md,
        effort: effort.filter(|e| !e.trim().is_empty()),
        isolation: isolation.filter(|i| !i.trim().is_empty()),
        initial_prompt: initial_prompt.filter(|p| !p.trim().is_empty()),
        experimental,
        mcp_servers,
        hooks,
    };
    let yaml = serde_yaml::to_string(&fm).map_err(|e| e.to_string())?;
    let contents = join_frontmatter(&yaml, &body);
    fs::write(&new_path, contents).map_err(|e| e.to_string())?;

    Ok(AgentSummary {
        id: slug.clone(),
        name: slug,
        description,
        scope,
        valid: true,
        errors: vec![],
    })
}

#[tauri::command]
pub fn delete_agent(state: State<AppState>, scope: Scope, id: String) -> Result<(), String> {
    let path = agents_dir(&state, scope).join(format!("{id}.md"));
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

    fn save(state: &AppState, original_id: Option<&str>, name: &str, description: &str, body: &str) -> Result<AgentSummary, String> {
        save_agent_impl(
            state,
            SaveAgentInput {
                scope: Scope::Project,
                original_id: original_id.map(|s| s.to_string()),
                name: name.to_string(),
                description: description.to_string(),
                tools: None,
                model: None,
                disallowed_tools: None,
                permission_mode: None,
                color: None,
                max_turns: None,
                skills: None,
                memory: None,
                background: None,
                omit_claude_md: None,
                effort: None,
                isolation: None,
                initial_prompt: None,
                experimental: None,
                mcp_servers: None,
                hooks: None,
                body: body.to_string(),
            },
        )
    }

    #[test]
    fn error_messages_switch_language_when_app_state_lang_changes() {
        let (_g, state) = fake_state();
        save(&state, None, "backend-java", "original", "original body").unwrap();

        let err_en = save(&state, None, "backend-java", "different", "x").unwrap_err();
        assert!(err_en.contains("already exists"), "default AppState lang should be English, got: {err_en}");

        *state.lang.lock().unwrap() = crate::models::Lang::Es;
        let err_es = save(&state, None, "backend-java", "different", "x").unwrap_err();
        assert!(err_es.contains("Ya existe"), "switching AppState.lang to Es should switch the message, got: {err_es}");
    }

    #[test]
    fn creating_a_new_agent_with_a_colliding_name_is_rejected_not_overwritten() {
        let (_g, state) = fake_state();
        save(&state, None, "backend-java", "original", "original body").unwrap();

        let err = save(&state, None, "backend-java", "different", "overwritten body")
            .expect_err("creating a second agent with the same name must be rejected");
        assert!(err.contains("already exists"));

        let on_disk = fs::read_to_string(agents_dir(&state, Scope::Project).join("backend-java.md")).unwrap();
        assert!(on_disk.contains("original body"));
    }

    #[test]
    fn editing_an_agent_and_keeping_its_name_still_works() {
        let (_g, state) = fake_state();
        save(&state, None, "backend-java", "v1", "v1 body").unwrap();
        save(&state, Some("backend-java"), "backend-java", "v2", "v2 body")
            .expect("editing the same agent in place must succeed");
        let on_disk = fs::read_to_string(agents_dir(&state, Scope::Project).join("backend-java.md")).unwrap();
        assert!(on_disk.contains("v2 body"));
    }

    #[test]
    fn renaming_onto_another_existing_agent_is_rejected() {
        let (_g, state) = fake_state();
        save(&state, None, "alpha", "a", "a-body").unwrap();
        save(&state, None, "beta", "b", "b-body").unwrap();

        let err = save(&state, Some("beta"), "alpha", "b renamed onto a", "should not land")
            .expect_err("renaming beta to the name alpha already uses must be rejected");
        assert!(err.contains("already exists"));

        let alpha_on_disk = fs::read_to_string(agents_dir(&state, Scope::Project).join("alpha.md")).unwrap();
        assert!(alpha_on_disk.contains("a-body"), "alpha must be untouched");
    }

    #[test]
    fn round_trips_all_extended_frontmatter_fields() {
        let fm = AgentFrontmatter {
            name: "db-reader".into(),
            description: "Execute read-only database queries".into(),
            tools: Some("Bash".into()),
            model: Some("haiku".into()),
            disallowed_tools: Some("Write, Edit".into()),
            permission_mode: Some("plan".into()),
            color: Some("cyan".into()),
            max_turns: Some(10),
            skills: Some(vec!["sql-review".into(), "schema-audit".into()]),
            memory: Some("user".into()),
            background: Some(true),
            omit_claude_md: Some(true),
            effort: Some("high".into()),
            isolation: Some("worktree".into()),
            initial_prompt: Some("Start by listing tables.".into()),
            experimental: Some(serde_json::json!({"cacheTtl": "1h"})),
            mcp_servers: Some(serde_json::json!(["github"])),
            hooks: Some(serde_json::json!({"PreToolUse": []})),
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        let raw = join_frontmatter(&yaml, "You only run SELECT queries.\n");

        let parsed = parse_agent_file("db-reader", Scope::Project, &raw);
        assert_eq!(parsed.name, "db-reader");
        assert_eq!(parsed.disallowed_tools.as_deref(), Some("Write, Edit"));
        assert_eq!(parsed.permission_mode.as_deref(), Some("plan"));
        assert_eq!(parsed.color.as_deref(), Some("cyan"));
        assert_eq!(parsed.max_turns, Some(10));
        assert_eq!(
            parsed.skills,
            Some(vec!["sql-review".to_string(), "schema-audit".to_string()])
        );
        assert_eq!(parsed.memory.as_deref(), Some("user"));
        assert_eq!(parsed.background, Some(true));
        assert_eq!(parsed.omit_claude_md, Some(true));
        assert_eq!(parsed.effort.as_deref(), Some("high"));
        assert_eq!(parsed.isolation.as_deref(), Some("worktree"));
        assert_eq!(parsed.initial_prompt.as_deref(), Some("Start by listing tables."));
        assert_eq!(
            parsed.experimental.and_then(|v| v.get("cacheTtl").cloned()),
            Some(serde_json::json!("1h"))
        );
        assert_eq!(parsed.mcp_servers, Some(serde_json::json!(["github"])));
        assert!(parsed.hooks.is_some());
        assert!(parsed.body.contains("SELECT queries"));
    }

    #[test]
    fn agent_without_extended_fields_still_parses() {
        let raw = "---\nname: simple\ndescription: Does one thing\n---\n\nBody text.\n";
        let parsed = parse_agent_file("simple", Scope::Project, raw);
        assert_eq!(parsed.name, "simple");
        assert!(parsed.disallowed_tools.is_none());
        assert!(parsed.skills.is_none());
    }
}
