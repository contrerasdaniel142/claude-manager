use crate::claude_cli::run_text_command_in;
use crate::git::clone_repo;
use crate::i18n;
use crate::mcp_servers::{cli_dir, cli_scope};
use crate::models::{AppState, Lang, McpServerSummary, Scope};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMcpServer {
    /// Path to the `.mcp.json` that defines this entry, relative to the
    /// repo root — a monorepo or example repo can have more than one.
    pub rel_path: String,
    pub name: String,
    pub server_type: String,
}

#[tauri::command]
pub async fn list_remote_mcp_servers(
    state: State<'_, AppState>,
    repo_url: String,
) -> Result<Vec<RemoteMcpServer>, String> {
    // Same reasoning as list_remote_skills: git clone + a directory walk
    // block for real, so this has to run off the command-dispatch thread.
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || list_remote_mcp_servers_impl(lang, &repo_url))
        .await
        .map_err(|e| e.to_string())?
}

fn list_remote_mcp_servers_impl(lang: Lang, repo_url: &str) -> Result<Vec<RemoteMcpServer>, String> {
    let tmp = clone_repo(lang, repo_url)?;
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(&tmp)
        .into_iter()
        // Skip dotted directories (.git, .claude-plugin, tool-specific config
        // dirs) but never skip a dotted FILE — ".mcp.json" is itself one.
        .filter_entry(|e| {
            e.depth() == 0 || e.file_type().is_file() || !e.file_name().to_string_lossy().starts_with('.')
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() || entry.file_name() != ".mcp.json" {
            continue;
        }
        let rel_path = entry.path().strip_prefix(&tmp).unwrap_or(entry.path()).to_string_lossy().replace('\\', "/");
        let Ok(raw) = fs::read_to_string(entry.path()) else { continue };
        let Ok(Value::Object(doc)) = serde_json::from_str::<Value>(&raw) else { continue };
        let Some(servers) = doc.get("mcpServers").and_then(|v| v.as_object()) else { continue };
        for (name, v) in servers {
            out.push(RemoteMcpServer {
                rel_path: rel_path.clone(),
                name: name.clone(),
                server_type: v.get("type").and_then(|t| t.as_str()).unwrap_or("stdio").to_string(),
            });
        }
    }
    let _ = fs::remove_dir_all(&tmp);
    out.sort_by(|a, b| a.name.cmp(&b.name));
    if out.is_empty() {
        return Err(i18n::no_mcp_json_in_repo(lang));
    }
    Ok(out)
}

#[tauri::command]
pub async fn import_mcp_server(
    state: State<'_, AppState>,
    repo_url: String,
    rel_path: String,
    name: String,
    scope: Scope,
) -> Result<McpServerSummary, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || import_mcp_server_impl(&state, repo_url, rel_path, name, scope))
        .await
        .map_err(|e| e.to_string())?
}

fn import_mcp_server_impl(
    state: &AppState,
    repo_url: String,
    rel_path: String,
    name: String,
    scope: Scope,
) -> Result<McpServerSummary, String> {
    let lang = state.lang();
    let tmp = clone_repo(lang, &repo_url)?;
    let result = (|| {
        let raw = fs::read_to_string(tmp.join(&rel_path)).map_err(|_| i18n::mcp_json_disappeared(lang, &rel_path))?;
        let doc: Value = serde_json::from_str(&raw).map_err(|_| i18n::mcp_json_disappeared(lang, &rel_path))?;
        let entry = doc
            .get("mcpServers")
            .and_then(|v| v.get(&name))
            .cloned()
            .ok_or_else(|| i18n::mcp_json_disappeared(lang, &rel_path))?;

        let server_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("stdio").to_string();
        let entry_json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
        let dir = cli_dir(state, scope);
        run_text_command_in(lang, dir.as_deref(), "mcp", &["add-json", &name, &entry_json, "--scope", cli_scope(scope)])?;

        Ok(McpServerSummary { name, scope, server_type })
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().join("project");
        let global_root = dir.path().join("home").join(".claude");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&global_root).unwrap();
        (dir, AppState::new(project_root, global_root))
    }

    fn fake_repo_with_mcp_json(doc: &str) -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();
        fs::write(repo.path().join(".mcp.json"), doc).unwrap();
        std::process::Command::new("git").args(["init", "--quiet"]).current_dir(repo.path()).output().unwrap();
        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-c", "user.email=t@t.com", "-c", "user.name=t", "commit", "--quiet", "-m", "init"])
            .current_dir(repo.path())
            .output()
            .unwrap();
        repo
    }

    const DOC: &str = r#"{"mcpServers": {"github": {"type": "stdio", "command": "npx", "args": ["-y", "gh-mcp"]}, "docs": {"type": "http", "url": "https://example.com/mcp"}}}"#;

    #[test]
    fn finds_every_server_defined_in_the_repos_mcp_json() {
        let repo = fake_repo_with_mcp_json(DOC);
        let servers = list_remote_mcp_servers_impl(Lang::En, &repo.path().to_string_lossy()).unwrap();
        assert_eq!(servers.len(), 2);
        let names: Vec<&str> = servers.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"github"));
        assert!(names.contains(&"docs"));
        let github = servers.iter().find(|s| s.name == "github").unwrap();
        assert_eq!(github.server_type, "stdio");
        assert_eq!(github.rel_path, ".mcp.json");
    }

    #[test]
    fn importing_a_server_writes_it_into_the_targets_mcp_json() {
        let (_g, state) = fake_state();
        let repo = fake_repo_with_mcp_json(DOC);
        let summary = import_mcp_server_impl(
            &state,
            repo.path().to_string_lossy().to_string(),
            ".mcp.json".into(),
            "github".into(),
            Scope::Project,
        )
        .unwrap();
        assert_eq!(summary.name, "github");

        let raw = fs::read_to_string(state.project_root().join(".mcp.json")).unwrap();
        let doc: Value = serde_json::from_str(&raw).unwrap();
        let entry = doc.get("mcpServers").and_then(|v| v.get("github")).unwrap();
        assert_eq!(entry.get("command").and_then(|v| v.as_str()), Some("npx"));
    }

    #[test]
    fn importing_onto_a_colliding_name_is_rejected() {
        let (_g, state) = fake_state();
        let repo = fake_repo_with_mcp_json(DOC);
        import_mcp_server_impl(
            &state,
            repo.path().to_string_lossy().to_string(),
            ".mcp.json".into(),
            "github".into(),
            Scope::Project,
        )
        .unwrap();

        let err = import_mcp_server_impl(
            &state,
            repo.path().to_string_lossy().to_string(),
            ".mcp.json".into(),
            "github".into(),
            Scope::Project,
        )
        .expect_err("importing onto an existing server name must be rejected");
        assert!(err.contains("already exists"));
    }

    #[test]
    fn repo_without_any_mcp_json_is_reported_as_empty() {
        let repo = tempfile::tempdir().unwrap();
        std::process::Command::new("git").args(["init", "--quiet"]).current_dir(repo.path()).output().unwrap();
        fs::write(repo.path().join("README.md"), "nothing here").unwrap();
        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-c", "user.email=t@t.com", "-c", "user.name=t", "commit", "--quiet", "-m", "init"])
            .current_dir(repo.path())
            .output()
            .unwrap();

        let err = list_remote_mcp_servers_impl(Lang::En, &repo.path().to_string_lossy())
            .expect_err("a repo with no .mcp.json must error, not return an empty list silently");
        assert!(err.to_lowercase().contains("mcp.json") || err.to_lowercase().contains(".mcp"));
    }
}
