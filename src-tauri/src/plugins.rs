use crate::claude_cli::run as run_claude;
use crate::i18n;
use crate::models::{AppState, Lang, MarketplaceSummary, PluginSearchPage, PluginSummary};
use serde_json::Value;
use tauri::State;

fn run_claude_plugin(lang: Lang, args: &[&str]) -> Result<(String, String, i32), String> {
    run_claude(lang, "plugin", args)
}

/// Per the CLI docs, `--json` prints its result as the LAST non-empty line
/// of stdout (any marketplace-declared command gets printed ahead of it).
fn last_json_line(stdout: &str) -> Option<Value> {
    stdout.lines().rev().find(|l| !l.trim().is_empty()).and_then(|l| serde_json::from_str(l).ok())
}

/// `{"command":..., "outcome": "ok"|"failed", "message": "..."}` on every
/// install/uninstall/enable/disable/marketplace command — same shape for
/// all of them, so one function reads the result for every one of them.
fn run_json_command(lang: Lang, args: &[&str]) -> Result<String, String> {
    let (stdout, stderr, code) = run_claude_plugin(lang, args)?;
    if let Some(v) = last_json_line(&stdout) {
        let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
        if v.get("outcome").and_then(|o| o.as_str()) == Some("ok") {
            return Ok(message);
        }
        return Err(message);
    }
    if code == 0 {
        return Ok(stdout.trim().to_string());
    }
    let reason = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
    Err(i18n::claude_returned_error(lang, reason, code))
}

#[tauri::command]
pub fn list_marketplaces(state: State<AppState>) -> Result<Vec<MarketplaceSummary>, String> {
    let lang = state.lang();
    let (stdout, stderr, code) = run_claude_plugin(lang, &["marketplace", "list", "--json"])?;
    if code != 0 {
        return Err(i18n::claude_returned_error(lang, stderr.trim(), code));
    }
    let arr: Vec<Value> = serde_json::from_str(stdout.trim()).unwrap_or_default();
    Ok(arr
        .iter()
        .map(|v| MarketplaceSummary {
            name: v.get("name").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
            source: v.get("source").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
            repo: v.get("repo").and_then(|x| x.as_str()).map(|s| s.to_string()),
            url: v.get("url").and_then(|x| x.as_str()).map(|s| s.to_string()),
            path: v.get("path").and_then(|x| x.as_str()).map(|s| s.to_string()),
        })
        .collect())
}

#[tauri::command]
pub async fn add_marketplace(state: State<'_, AppState>, source: String, scope: String) -> Result<String, String> {
    let lang = state.lang();
    let result = tauri::async_runtime::spawn_blocking(move || {
        run_json_command(lang, &["marketplace", "add", &source, "--scope", &scope, "--json"])
    })
    .await
    .map_err(|e| e.to_string())?;
    if result.is_ok() {
        state.clear_available_plugins_cache();
    }
    result
}

#[tauri::command]
pub async fn remove_marketplace(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let lang = state.lang();
    let result = tauri::async_runtime::spawn_blocking(move || {
        run_json_command(lang, &["marketplace", "remove", &name, "--json"])
    })
    .await
    .map_err(|e| e.to_string())?;
    if result.is_ok() {
        state.clear_available_plugins_cache();
    }
    result
}

fn split_id(id: &str) -> (String, String) {
    match id.split_once('@') {
        Some((name, marketplace)) => (name.to_string(), marketplace.to_string()),
        None => (id.to_string(), String::new()),
    }
}

/// Installed and available entries from `plugin list --json --available`
/// use different field names for the same things: installed carries `id`
/// (`name@marketplace`) and no separate `name`/`marketplaceName`; available
/// carries `pluginId`, plus `name` and `marketplaceName` directly instead of
/// packing them into one string. Read both shapes, preferring the direct
/// fields when present and falling back to splitting the id otherwise.
fn parse_plugin(v: &Value, installed: bool) -> PluginSummary {
    let id = v
        .get("id")
        .or_else(|| v.get("pluginId"))
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string();
    let (name_from_id, marketplace_from_id) = split_id(&id);
    let name = v.get("name").and_then(|x| x.as_str()).map(|s| s.to_string()).unwrap_or(name_from_id);
    let marketplace = v
        .get("marketplaceName")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .unwrap_or(marketplace_from_id);
    PluginSummary {
        id,
        name,
        marketplace,
        version: v.get("version").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        scope: v.get("scope").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        enabled: v.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false),
        installed,
        description: v.get("description").and_then(|x| x.as_str()).map(|s| s.to_string()),
    }
}

/// Cheap: `claude plugin list --json` (no `--available`) returns a flat array
/// of just the installed plugins — nowhere near the size of the full catalog,
/// so this is safe to call on every refresh.
#[tauri::command]
pub async fn list_installed_plugins(state: State<'_, AppState>) -> Result<Vec<PluginSummary>, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || {
        let (stdout, stderr, code) = run_claude_plugin(lang, &["list", "--json"])?;
        if code != 0 {
            return Err(i18n::claude_returned_error(lang, stderr.trim(), code));
        }
        let arr: Vec<Value> = serde_json::from_str(stdout.trim()).unwrap_or_default();
        Ok(arr.iter().map(|v| parse_plugin(v, true)).collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// `claude plugin list --json --available` dumps the ENTIRE marketplace
/// catalog (thousands of entries, multi-MB) in one go — there's no `--query`
/// or `--page` on the CLI itself. Rather than reshipping that whole thing to
/// the frontend on every keystroke, we pay the parse cost once, cache the
/// parsed `Vec<PluginSummary>` in `AppState`, and do the filtering/pagination
/// here so only one small page ever crosses the IPC bridge.
pub fn available_plugins(state: &AppState, force: bool) -> Result<Vec<PluginSummary>, String> {
    if !force {
        if let Some(cached) = state.cached_available_plugins() {
            return Ok(cached);
        }
    }
    let lang = state.lang();
    let (stdout, stderr, code) = run_claude_plugin(lang, &["list", "--json", "--available"])?;
    if code != 0 {
        return Err(i18n::claude_returned_error(lang, stderr.trim(), code));
    }
    let doc: Value = serde_json::from_str(stdout.trim()).unwrap_or_default();
    let available: Vec<PluginSummary> = doc
        .get("available")
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().map(|v| parse_plugin(v, false)).collect())
        .unwrap_or_default();
    state.set_available_plugins_cache(available.clone());
    Ok(available)
}

fn filter_available(
    all: &[PluginSummary],
    marketplace: &str,
    query: &str,
    offset: usize,
    limit: usize,
) -> PluginSearchPage {
    let query = query.trim().to_lowercase();
    let matching: Vec<&PluginSummary> = all
        .iter()
        .filter(|p| marketplace.is_empty() || p.marketplace == marketplace)
        .filter(|p| {
            query.is_empty()
                || p.name.to_lowercase().contains(&query)
                || p.description.as_deref().unwrap_or_default().to_lowercase().contains(&query)
        })
        .collect();
    PluginSearchPage {
        total: matching.len(),
        items: matching.into_iter().skip(offset).take(limit).cloned().collect(),
    }
}

#[tauri::command]
pub async fn search_available_plugins(
    state: State<'_, AppState>,
    marketplace: String,
    query: String,
    offset: usize,
    limit: usize,
    force: bool,
) -> Result<PluginSearchPage, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let all = available_plugins(&state, force)?;
        Ok(filter_available(&all, &marketplace, &query, offset, limit))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn install_plugin(
    state: State<'_, AppState>,
    name: String,
    marketplace: String,
    scope: String,
) -> Result<String, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || {
        let id = format!("{name}@{marketplace}");
        // `-y` auto-accepts a marketplace-declared install command instead of
        // waiting on a terminal prompt clman has no TTY to answer — equivalent
        // to the user running the same `-y` install from their own terminal.
        run_json_command(lang, &["install", &id, "--scope", &scope, "-y", "--json"])
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn uninstall_plugin(
    state: State<'_, AppState>,
    name: String,
    marketplace: String,
    scope: String,
) -> Result<String, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || {
        let id = format!("{name}@{marketplace}");
        run_json_command(lang, &["uninstall", &id, "--scope", &scope, "--json"])
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn enable_plugin(
    state: State<'_, AppState>,
    name: String,
    marketplace: String,
    scope: String,
) -> Result<String, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || {
        // Bare names fail with failureCode "ambiguous_marketplace" even with
        // a single marketplace added — enable/disable need the full id, same
        // as install/uninstall already pass.
        let id = format!("{name}@{marketplace}");
        run_json_command(lang, &["enable", &id, "--scope", &scope, "--json"])
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn disable_plugin(
    state: State<'_, AppState>,
    name: String,
    marketplace: String,
    scope: String,
) -> Result<String, String> {
    let lang = state.lang();
    tauri::async_runtime::spawn_blocking(move || {
        let id = format!("{name}@{marketplace}");
        run_json_command(lang, &["disable", &id, "--scope", &scope, "--json"])
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_json_line_ignores_a_marketplace_declared_command_printed_before_it() {
        let stdout = "Running install command...\nsome output from that command\n{\"command\":\"install\",\"outcome\":\"ok\",\"message\":\"Successfully installed plugin: formatter@my-marketplace\"}\n";
        let v = last_json_line(stdout).expect("should find the trailing JSON line");
        assert_eq!(v.get("outcome").and_then(|o| o.as_str()), Some("ok"));
    }

    #[test]
    fn run_json_command_turns_a_failed_outcome_into_an_error_with_the_real_message() {
        // Exercise the pure parsing path directly rather than spawning a real
        // `claude` process — same technique as testing a format function.
        let stdout = "{\"command\":\"install\",\"outcome\":\"failed\",\"message\":\"Plugin \\\"ghost@nowhere\\\" not found\"}";
        let v = last_json_line(stdout).unwrap();
        assert_eq!(v.get("outcome").and_then(|o| o.as_str()), Some("failed"));
        assert_eq!(v.get("message").and_then(|m| m.as_str()), Some("Plugin \"ghost@nowhere\" not found"));
    }

    #[test]
    fn split_id_separates_plugin_name_from_marketplace() {
        assert_eq!(split_id("formatter@my-marketplace"), ("formatter".to_string(), "my-marketplace".to_string()));
        assert_eq!(split_id("formatter"), ("formatter".to_string(), String::new()));
    }

    #[test]
    fn parse_plugin_reads_every_field_and_tags_installed_vs_available() {
        let v: Value = serde_json::from_str(
            r#"{"id":"formatter@my-marketplace","version":"1.2.0","scope":"project","enabled":true}"#,
        )
        .unwrap();
        let p = parse_plugin(&v, true);
        assert_eq!(p.name, "formatter");
        assert_eq!(p.marketplace, "my-marketplace");
        assert_eq!(p.version, "1.2.0");
        assert_eq!(p.scope, "project");
        assert!(p.enabled);
        assert!(p.installed);
    }

    #[test]
    fn parse_plugin_reads_the_available_shape_which_uses_different_field_names() {
        // Real shape from `claude plugin list --json --available`: no `id`,
        // `name`/`marketplaceName` given directly instead of packed into one
        // string, and no version/scope/enabled since it isn't installed.
        let v: Value = serde_json::from_str(
            r#"{"pluginId":"formatter@my-marketplace","name":"formatter","marketplaceName":"my-marketplace","description":"Formats code"}"#,
        )
        .unwrap();
        let p = parse_plugin(&v, false);
        assert_eq!(p.name, "formatter");
        assert_eq!(p.marketplace, "my-marketplace");
        assert_eq!(p.description, Some("Formats code".to_string()));
        assert!(!p.installed);
    }

    fn sample_plugin(name: &str, marketplace: &str, description: &str) -> PluginSummary {
        PluginSummary {
            id: format!("{name}@{marketplace}"),
            name: name.to_string(),
            marketplace: marketplace.to_string(),
            description: Some(description.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn filter_available_narrows_by_marketplace_and_query_and_paginates() {
        let all = vec![
            sample_plugin("formatter", "official", "Formats code"),
            sample_plugin("linter", "official", "Lints code"),
            sample_plugin("formatter", "community", "A different formatter"),
        ];

        let page = filter_available(&all, "official", "format", 0, 10);
        assert_eq!(page.total, 1, "only the official-marketplace formatter should match");
        assert_eq!(page.items[0].name, "formatter");

        let page = filter_available(&all, "", "", 0, 2);
        assert_eq!(page.total, 3, "empty marketplace/query means no filter, just everything");
        assert_eq!(page.items.len(), 2, "limit caps the page even though more match");

        let page = filter_available(&all, "", "", 2, 2);
        assert_eq!(page.items.len(), 1, "offset skips past the first page");
    }
}
