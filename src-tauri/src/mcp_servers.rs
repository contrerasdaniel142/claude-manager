use crate::claude_cli::run_text_command_in;
use crate::fsutil::load_settings;
use crate::models::{AppState, McpServerFile, McpServerSummary, SaveMcpServerInput, Scope};
use crate::validate::validate_mcp_server;
use serde_json::{Map, Value};
use std::path::PathBuf;
use tauri::State;

/// Project scope reads the team-shared `.mcp.json` at the project root.
/// Global scope reads the `mcpServers` key of `~/.claude.json`, the same
/// file Claude Code itself uses for user-scoped servers.
///
/// Reads go straight to the file because `claude mcp list`/`get` have no
/// `--json` flag — nothing to machine-parse there. Writes (below) instead
/// shell out to `claude mcp add-json`/`remove`, the same lesson applied to
/// Plugins: let the real CLI own collision checks and the exact on-disk
/// shape instead of clman reimplementing them.
pub(crate) fn mcp_file_path(state: &AppState, scope: Scope) -> PathBuf {
    match scope {
        Scope::Project => state.project_root().join(".mcp.json"),
        Scope::Global => state
            .global_root
            .parent()
            .map(|home| home.join(".claude.json"))
            .unwrap_or_else(|| PathBuf::from(".claude.json")),
    }
}

pub(crate) const SERVERS_KEY: &str = "mcpServers";

fn obj_to_lines(v: Option<&Value>, sep: &str) -> Option<String> {
    let obj = v?.as_object()?;
    if obj.is_empty() {
        return None;
    }
    Some(
        obj.iter()
            .map(|(k, v)| format!("{k}{sep}{}", v.as_str().unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn lines_to_obj(text: Option<&str>, sep: char) -> Option<Value> {
    let text = text?.trim();
    if text.is_empty() {
        return None;
    }
    let mut map = Map::new();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once(sep) {
            map.insert(k.trim().to_string(), Value::String(v.trim().to_string()));
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(Value::Object(map))
    }
}

fn parse_entry(name: &str, scope: Scope, v: &Value) -> McpServerFile {
    McpServerFile {
        name: name.to_string(),
        scope,
        server_type: v.get("type").and_then(|t| t.as_str()).unwrap_or("stdio").to_string(),
        command: v.get("command").and_then(|c| c.as_str()).map(|s| s.to_string()),
        args: v
            .get("args")
            .and_then(|a| a.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        env: obj_to_lines(v.get("env"), "="),
        url: v.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()),
        headers: obj_to_lines(v.get("headers"), ": "),
    }
}

/// The bare server entry (no outer `{"mcpServers": {...}}` wrapper) —
/// exactly the JSON `claude mcp add-json <name> <json>` expects.
fn build_entry_json(input: &SaveMcpServerInput) -> Value {
    let mut obj = Map::new();
    obj.insert("type".into(), Value::String(input.server_type.clone()));
    match input.server_type.as_str() {
        "http" | "sse" | "streamable-http" | "ws" => {
            obj.insert("url".into(), Value::String(input.url.clone().unwrap_or_default()));
            if let Some(headers) = lines_to_obj(input.headers.as_deref(), ':') {
                obj.insert("headers".into(), headers);
            }
        }
        _ => {
            obj.insert("command".into(), Value::String(input.command.clone().unwrap_or_default()));
            if !input.args.is_empty() {
                obj.insert("args".into(), Value::Array(input.args.iter().cloned().map(Value::String).collect()));
            }
            if let Some(env) = lines_to_obj(input.env.as_deref(), '=') {
                obj.insert("env".into(), env);
            }
        }
    }
    Value::Object(obj)
}

pub(crate) fn servers_map(settings: &Map<String, Value>) -> Map<String, Value> {
    settings.get(SERVERS_KEY).and_then(|v| v.as_object()).cloned().unwrap_or_default()
}

/// `claude mcp`'s own scope names — "user" is clman's "Global".
pub(crate) fn cli_scope(scope: Scope) -> &'static str {
    match scope {
        Scope::Project => "project",
        Scope::Global => "user",
    }
}

/// Project-scope commands resolve "this project" from the process's cwd, not
/// from any flag — so we have to launch `claude` rooted there explicitly.
/// User scope always resolves to the real `~/.claude.json` regardless of cwd.
pub(crate) fn cli_dir(state: &AppState, scope: Scope) -> Option<std::path::PathBuf> {
    match scope {
        Scope::Project => Some(state.project_root()),
        Scope::Global => None,
    }
}

#[tauri::command]
pub fn list_mcp_servers(state: State<AppState>, scope: Scope) -> Result<Vec<McpServerSummary>, String> {
    let path = mcp_file_path(&state, scope);
    let settings = load_settings(state.lang(), &path)?;
    let mut out: Vec<McpServerSummary> = servers_map(&settings)
        .iter()
        .map(|(name, v)| McpServerSummary {
            name: name.clone(),
            scope,
            server_type: v.get("type").and_then(|t| t.as_str()).unwrap_or("stdio").to_string(),
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[tauri::command]
pub fn read_mcp_server(state: State<AppState>, scope: Scope, name: String) -> Result<McpServerFile, String> {
    let path = mcp_file_path(&state, scope);
    let settings = load_settings(state.lang(), &path)?;
    let entry = servers_map(&settings)
        .get(&name)
        .cloned()
        .ok_or_else(|| crate::i18n::read_file_failed(state.lang(), &name, "not found"))?;
    Ok(parse_entry(&name, scope, &entry))
}

#[tauri::command]
pub async fn save_mcp_server(state: State<'_, AppState>, input: SaveMcpServerInput) -> Result<McpServerSummary, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || save_mcp_server_impl(&state, input))
        .await
        .map_err(|e| e.to_string())?
}

fn save_mcp_server_impl(state: &AppState, input: SaveMcpServerInput) -> Result<McpServerSummary, String> {
    let lang = state.lang();
    let name = input.name.trim().to_string();
    let errors = validate_mcp_server(
        lang,
        &McpServerFile {
            name: name.clone(),
            scope: input.scope,
            server_type: input.server_type.clone(),
            command: input.command.clone(),
            args: input.args.clone(),
            env: input.env.clone(),
            url: input.url.clone(),
            headers: input.headers.clone(),
        },
    );
    if !errors.is_empty() {
        return Err(errors.join(" "));
    }

    let scope_str = cli_scope(input.scope);
    let dir = cli_dir(state, input.scope);

    // Editing in place (same name) or renaming both need the old entry gone
    // first — `add-json` refuses to overwrite a name that already exists.
    // Best-effort: nothing to remove yet on a brand-new server is not an error.
    if let Some(old_name) = &input.original_name {
        let _ = run_text_command_in(lang, dir.as_deref(), "mcp", &["remove", old_name, "--scope", scope_str]);
    }

    let json = serde_json::to_string(&build_entry_json(&input)).map_err(|e| e.to_string())?;
    run_text_command_in(lang, dir.as_deref(), "mcp", &["add-json", &name, &json, "--scope", scope_str])?;

    Ok(McpServerSummary { name, scope: input.scope, server_type: input.server_type })
}

#[tauri::command]
pub async fn delete_mcp_server(state: State<'_, AppState>, scope: Scope, name: String) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || delete_mcp_server_impl(&state, scope, name))
        .await
        .map_err(|e| e.to_string())?
}

fn delete_mcp_server_impl(state: &AppState, scope: Scope, name: String) -> Result<(), String> {
    let lang = state.lang();
    let dir = cli_dir(state, scope);
    run_text_command_in(lang, dir.as_deref(), "mcp", &["remove", &name, "--scope", cli_scope(scope)])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fake_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().join("project");
        let global_root = dir.path().join("home").join(".claude");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&global_root).unwrap();
        (dir, AppState::new(project_root, global_root))
    }

    fn stdio_input(scope: Scope, name: &str) -> SaveMcpServerInput {
        SaveMcpServerInput {
            scope,
            original_name: None,
            name: name.into(),
            server_type: "stdio".into(),
            command: Some("npx".into()),
            args: vec!["-y".into(), "@scope/pkg".into()],
            env: Some("LOG_LEVEL=info".into()),
            url: None,
            headers: None,
        }
    }

    // Project scope is fully sandboxed (rooted at a temp dir via cwd), so
    // these exercise the real `claude mcp add-json`/`remove` CLI, same as
    // skill_import's tests exercise a real `git clone`. Global/user scope
    // can't be sandboxed the same way — it always resolves to this
    // machine's single real `~/.claude.json` regardless of any fake
    // AppState — so that path is covered by manual verification instead
    // (confirmed live: add-json/remove at local/user/project scope, schema
    // matches `parse_entry`/`build_entry_json` exactly) plus the
    // scope-agnostic validation tests below.

    #[test]
    fn stdio_server_round_trips_through_project_scope_via_the_real_cli() {
        let (_g, state) = fake_state();
        save_mcp_server_impl(&state, stdio_input(Scope::Project, "database-tools")).unwrap();

        let path = mcp_file_path(&state, Scope::Project);
        assert!(path.ends_with(".mcp.json"));
        let settings = load_settings(state.lang(), &path).unwrap();
        let entry = parse_entry("database-tools", Scope::Project, servers_map(&settings).get("database-tools").unwrap());
        assert_eq!(entry.command, Some("npx".into()));
        assert_eq!(entry.args, vec!["-y".to_string(), "@scope/pkg".to_string()]);
        assert_eq!(entry.env, Some("LOG_LEVEL=info".into()));
    }

    #[test]
    fn http_server_without_url_is_rejected_before_touching_the_cli() {
        let (_g, state) = fake_state();
        let mut input = stdio_input(Scope::Project, "broken");
        input.server_type = "http".into();
        input.url = None;
        let err = save_mcp_server_impl(&state, input).expect_err("http server without url must be rejected");
        assert!(err.contains("URL"));
    }

    #[test]
    fn stdio_server_without_command_is_rejected_before_touching_the_cli() {
        let (_g, state) = fake_state();
        let mut input = stdio_input(Scope::Project, "broken");
        input.command = None;
        let err = save_mcp_server_impl(&state, input).expect_err("stdio server without command must be rejected");
        assert!(err.contains("command"));
    }

    #[test]
    fn creating_a_new_server_with_a_colliding_name_is_rejected_by_the_real_cli() {
        let (_g, state) = fake_state();
        save_mcp_server_impl(&state, stdio_input(Scope::Project, "github")).unwrap();
        save_mcp_server_impl(&state, stdio_input(Scope::Project, "github"))
            .expect_err("creating a second server with the same name must be rejected");
    }

    #[test]
    fn renaming_a_server_removes_the_old_key() {
        let (_g, state) = fake_state();
        save_mcp_server_impl(&state, stdio_input(Scope::Project, "old-name")).unwrap();

        let mut rename = stdio_input(Scope::Project, "new-name");
        rename.original_name = Some("old-name".into());
        save_mcp_server_impl(&state, rename).unwrap();

        let path = mcp_file_path(&state, Scope::Project);
        let settings = load_settings(state.lang(), &path).unwrap();
        let servers = servers_map(&settings);
        assert!(!servers.contains_key("old-name"));
        assert!(servers.contains_key("new-name"));
    }

    #[test]
    fn editing_a_server_in_place_keeps_only_the_new_fields() {
        let (_g, state) = fake_state();
        save_mcp_server_impl(&state, stdio_input(Scope::Project, "same-name")).unwrap();

        let mut edit = stdio_input(Scope::Project, "same-name");
        edit.original_name = Some("same-name".into());
        edit.command = Some("other-command".into());
        save_mcp_server_impl(&state, edit).unwrap();

        let path = mcp_file_path(&state, Scope::Project);
        let settings = load_settings(state.lang(), &path).unwrap();
        let entry = parse_entry("same-name", Scope::Project, servers_map(&settings).get("same-name").unwrap());
        assert_eq!(entry.command, Some("other-command".into()));
    }

    #[test]
    fn deleting_a_server_preserves_the_other_entries() {
        let (_g, state) = fake_state();
        save_mcp_server_impl(&state, stdio_input(Scope::Project, "a")).unwrap();
        save_mcp_server_impl(&state, stdio_input(Scope::Project, "b")).unwrap();

        delete_mcp_server_impl(&state, Scope::Project, "a".into()).unwrap();

        let path = mcp_file_path(&state, Scope::Project);
        let settings = load_settings(state.lang(), &path).unwrap();
        let servers = servers_map(&settings);
        assert!(!servers.contains_key("a"));
        assert!(servers.contains_key("b"));
    }
}
