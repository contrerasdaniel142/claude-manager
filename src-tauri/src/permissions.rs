use crate::fsutil::{load_settings, settings_path};
use crate::models::{AppState, PermissionsSettings, Scope};
use serde_json::{Map, Value};
use std::fs;
use tauri::State;

fn str_list(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str()).map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

/// `disableBypassPermissionsMode`/`disableAutoMode` are written as the
/// string `"disable"`, not a bool — any other value (including absence)
/// means "not disabled". The UI only ever needs the on/off distinction.
fn is_disable_sentinel(v: &Value, key: &str) -> bool {
    v.get(key).and_then(|v| v.as_str()) == Some("disable")
}

fn parse_permissions(settings: &Map<String, Value>) -> PermissionsSettings {
    let v = settings.get("permissions");
    let attribution = settings.get("attribution").and_then(|a| a.as_object());
    PermissionsSettings {
        allow: v.map(|v| str_list(v, "allow")).unwrap_or_default(),
        ask: v.map(|v| str_list(v, "ask")).unwrap_or_default(),
        deny: v.map(|v| str_list(v, "deny")).unwrap_or_default(),
        additional_directories: v.map(|v| str_list(v, "additionalDirectories")).unwrap_or_default(),
        default_mode: v.and_then(|v| v.get("defaultMode")).and_then(|v| v.as_str()).map(|s| s.to_string()),
        disable_bypass_permissions_mode: v.is_some_and(|v| is_disable_sentinel(v, "disableBypassPermissionsMode")),
        disable_auto_mode: v.is_some_and(|v| is_disable_sentinel(v, "disableAutoMode")),
        block_reads_outside_working_directories: v
            .and_then(|v| v.get("blockReadsOutsideWorkingDirectories"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        attribution_commit: attribution.and_then(|a| a.get("commit")).and_then(|v| v.as_str()).map(String::from),
        attribution_pr: attribution.and_then(|a| a.get("pr")).and_then(|v| v.as_str()).map(String::from),
        attribution_session_url: attribution.and_then(|a| a.get("sessionUrl")).and_then(|v| v.as_bool()),
        include_git_instructions: settings.get("includeGitInstructions").and_then(|v| v.as_bool()),
    }
}

fn build_permissions_value(p: &PermissionsSettings) -> Value {
    let mut obj = Map::new();
    if !p.allow.is_empty() {
        obj.insert("allow".into(), Value::Array(p.allow.iter().cloned().map(Value::String).collect()));
    }
    if !p.ask.is_empty() {
        obj.insert("ask".into(), Value::Array(p.ask.iter().cloned().map(Value::String).collect()));
    }
    if !p.deny.is_empty() {
        obj.insert("deny".into(), Value::Array(p.deny.iter().cloned().map(Value::String).collect()));
    }
    if !p.additional_directories.is_empty() {
        obj.insert(
            "additionalDirectories".into(),
            Value::Array(p.additional_directories.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(mode) = p.default_mode.as_deref().filter(|s| !s.is_empty()) {
        obj.insert("defaultMode".into(), Value::String(mode.to_string()));
    }
    if p.disable_bypass_permissions_mode {
        obj.insert("disableBypassPermissionsMode".into(), Value::String("disable".into()));
    }
    if p.disable_auto_mode {
        obj.insert("disableAutoMode".into(), Value::String("disable".into()));
    }
    if p.block_reads_outside_working_directories {
        obj.insert("blockReadsOutsideWorkingDirectories".into(), Value::Bool(true));
    }
    Value::Object(obj)
}

/// `attribution` is a sibling of `permissions` at the top level of
/// settings.json, not nested under it — built separately so it lands at
/// its own key when saved.
fn build_attribution_value(p: &PermissionsSettings) -> Value {
    let mut obj = Map::new();
    if let Some(commit) = &p.attribution_commit {
        obj.insert("commit".into(), Value::String(commit.clone()));
    }
    if let Some(pr) = &p.attribution_pr {
        obj.insert("pr".into(), Value::String(pr.clone()));
    }
    if let Some(session_url) = p.attribution_session_url {
        obj.insert("sessionUrl".into(), Value::Bool(session_url));
    }
    Value::Object(obj)
}

#[tauri::command]
pub fn read_permissions(state: State<AppState>, scope: Scope) -> Result<PermissionsSettings, String> {
    let path = settings_path(&state, scope);
    let settings = load_settings(state.lang(), &path)?;
    Ok(parse_permissions(&settings))
}

#[tauri::command]
pub fn save_permissions(
    state: State<AppState>,
    scope: Scope,
    input: PermissionsSettings,
) -> Result<PermissionsSettings, String> {
    save_permissions_impl(&state, scope, input)
}

fn save_permissions_impl(
    state: &AppState,
    scope: Scope,
    input: PermissionsSettings,
) -> Result<PermissionsSettings, String> {
    let path = settings_path(state, scope);
    let mut settings = load_settings(state.lang(), &path)?;

    let value = build_permissions_value(&input);
    match &value {
        Value::Object(obj) if obj.is_empty() => {
            settings.remove("permissions");
        }
        _ => {
            settings.insert("permissions".into(), value);
        }
    }

    let attribution = build_attribution_value(&input);
    match &attribution {
        Value::Object(obj) if obj.is_empty() => {
            settings.remove("attribution");
        }
        _ => {
            settings.insert("attribution".into(), attribution);
        }
    }

    match input.include_git_instructions {
        Some(v) => {
            settings.insert("includeGitInstructions".into(), Value::Bool(v));
        }
        None => {
            settings.remove("includeGitInstructions");
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&Value::Object(settings)).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| e.to_string())?;

    Ok(input)
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
    fn save_then_read_round_trips_every_field() {
        let (_g, state) = fake_state();
        let input = PermissionsSettings {
            allow: vec!["Bash(npm run test)".into(), "Read(~/.zshrc)".into()],
            ask: vec!["Bash(git push *)".into()],
            deny: vec!["Bash(curl *)".into(), "Read(./.env)".into()],
            additional_directories: vec!["../shared-lib".into()],
            default_mode: Some("acceptEdits".into()),
            disable_bypass_permissions_mode: true,
            disable_auto_mode: false,
            block_reads_outside_working_directories: true,
            attribution_commit: Some("".into()),
            attribution_pr: Some("Custom trailer".into()),
            attribution_session_url: Some(false),
            include_git_instructions: Some(false),
        };
        save_permissions_impl(&state, Scope::Project, input.clone()).unwrap();

        let path = settings_path(&state, Scope::Project);
        let settings = load_settings(state.lang(), &path).unwrap();
        let roundtripped = parse_permissions(&settings);
        assert_eq!(roundtripped, input);
    }

    #[test]
    fn attribution_is_written_as_its_own_top_level_key_not_nested_under_permissions() {
        let (_g, state) = fake_state();
        let input = PermissionsSettings {
            attribution_commit: Some("".into()),
            include_git_instructions: Some(false),
            ..Default::default()
        };
        save_permissions_impl(&state, Scope::Project, input).unwrap();

        let path = settings_path(&state, Scope::Project);
        let settings = load_settings(state.lang(), &path).unwrap();
        assert!(!settings.contains_key("permissions"));
        assert_eq!(
            settings.get("attribution").and_then(|a| a.get("commit")).and_then(|v| v.as_str()),
            Some("")
        );
        assert_eq!(settings.get("includeGitInstructions").and_then(|v| v.as_bool()), Some(false));
    }

    #[test]
    fn disable_sentinels_are_written_as_the_literal_string_disable_not_a_bool() {
        let (_g, state) = fake_state();
        let input = PermissionsSettings {
            disable_bypass_permissions_mode: true,
            ..Default::default()
        };
        save_permissions_impl(&state, Scope::Project, input).unwrap();

        let path = settings_path(&state, Scope::Project);
        let raw = fs::read_to_string(&path).unwrap();
        assert!(raw.contains("\"disableBypassPermissionsMode\": \"disable\""));
    }

    #[test]
    fn saving_all_empty_removes_the_permissions_key_entirely() {
        let (_g, state) = fake_state();
        save_permissions_impl(
            &state,
            Scope::Project,
            PermissionsSettings {
                allow: vec!["Bash".into()],
                ..Default::default()
            },
        )
        .unwrap();
        save_permissions_impl(&state, Scope::Project, PermissionsSettings::default()).unwrap();

        let path = settings_path(&state, Scope::Project);
        let settings = load_settings(state.lang(), &path).unwrap();
        assert!(!settings.contains_key("permissions"));
    }

    #[test]
    fn preserves_unrelated_top_level_keys_already_in_settings_json() {
        let (_g, state) = fake_state();
        let path = settings_path(&state, Scope::Project);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"model": "claude-opus-5-5", "env": {"FOO": "bar"}}"#).unwrap();

        save_permissions_impl(
            &state,
            Scope::Project,
            PermissionsSettings {
                allow: vec!["Bash".into()],
                ..Default::default()
            },
        )
        .unwrap();

        let settings = load_settings(state.lang(), &path).unwrap();
        assert_eq!(settings.get("model").and_then(|v| v.as_str()), Some("claude-opus-5-5"));
        assert!(settings.contains_key("env"));
        assert!(settings.contains_key("permissions"));
    }
}
