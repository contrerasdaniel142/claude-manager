mod agents;
mod assistant;
mod claude_cli;
mod fsutil;
mod git;
mod hooks;
mod i18n;
mod mcp_import;
mod mcp_servers;
mod models;
mod output_styles;
mod permissions;
mod plugins;
mod rules;
mod search;
mod skill_import;
mod skill_marketplace;
mod skills;
mod steering;
mod validate;
mod watcher;

use models::{AppState, Lang};
use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
fn get_roots(state: tauri::State<AppState>) -> serde_json::Value {
    serde_json::json!({
        "project": state.project_root().to_string_lossy(),
        "global": state.global_root.to_string_lossy(),
    })
}

/// El frontend llama esto al arrancar (según lo que tenga en localStorage) y
/// de nuevo cada vez que el usuario cambia de idioma — así los mensajes de
/// error que arma el backend salen en el idioma correcto sin tener que
/// mandarlo en cada llamada por separado.
#[tauri::command]
fn set_language(state: tauri::State<AppState>, lang: Lang) {
    *state.lang.lock().unwrap() = lang;
}

/// Switches the active project without restarting the app — the project
/// picker's "Open Folder" and "Recent" both land here. Every Tauri command
/// already reads `state.project_root()` fresh on each call, so nothing else
/// needs to be told; only the file watcher (its own background thread,
/// started once at startup) needs to be pointed at the new folder.
#[tauri::command]
fn set_project_root(app: tauri::AppHandle, state: tauri::State<AppState>, path: String) -> Result<serde_json::Value, String> {
    let new_root = PathBuf::from(&path);
    if !new_root.is_dir() {
        return Err(i18n::not_a_directory(state.lang(), &path));
    }
    state.set_project_root(new_root);
    // Starts a fresh watcher on the new project dir; the previous one stops
    // itself on its next wake-up once it notices its generation is stale
    // (see watcher.rs) instead of lingering.
    watcher::start(app, &state);
    Ok(get_roots(state))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(project_root: PathBuf) {
    let global_root = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new(project_root, global_root))
        .setup(|app| {
            let state = app.state::<AppState>();
            watcher::start(app.handle().clone(), &state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_roots,
            set_language,
            set_project_root,
            agents::list_agents,
            agents::read_agent,
            agents::save_agent,
            agents::delete_agent,
            agents::list_permission_modes,
            agents::list_agent_colors,
            skills::list_skills,
            skills::read_skill,
            skills::save_skill,
            skills::delete_skill,
            skills::move_skill,
            hooks::list_hook_events,
            hooks::read_hooks,
            hooks::save_hooks,
            steering::list_steering,
            steering::read_steering,
            steering::save_steering,
            steering::delete_steering,
            rules::list_rules,
            rules::read_rule,
            rules::save_rule,
            rules::delete_rule,
            search::search_everything,
            output_styles::list_output_styles,
            output_styles::read_output_style,
            output_styles::save_output_style,
            output_styles::delete_output_style,
            permissions::read_permissions,
            permissions::save_permissions,
            mcp_servers::list_mcp_servers,
            mcp_servers::read_mcp_server,
            mcp_servers::save_mcp_server,
            mcp_servers::delete_mcp_server,
            mcp_import::list_remote_mcp_servers,
            mcp_import::import_mcp_server,
            plugins::list_marketplaces,
            plugins::add_marketplace,
            plugins::remove_marketplace,
            plugins::list_installed_plugins,
            plugins::search_available_plugins,
            plugins::install_plugin,
            plugins::uninstall_plugin,
            plugins::enable_plugin,
            plugins::disable_plugin,
            assistant::assistant_available,
            assistant::generate_with_ai,
            assistant::list_assistant_models,
            skill_import::list_remote_skills,
            skill_import::import_skill,
            skill_marketplace::search_marketplace_skills,
            skill_marketplace::preview_marketplace_skill,
            skill_marketplace::install_marketplace_skill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
