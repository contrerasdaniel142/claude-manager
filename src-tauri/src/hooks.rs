use crate::fsutil::{load_settings, settings_path, silent_command};
use crate::i18n;
use crate::models::{AppState, HookRule, HooksDocument, Scope};
use crate::validate::{validate_hook, KNOWN_HOOK_EVENTS};
use serde_json::{Map, Value};
use std::fs;
use std::sync::OnceLock;
use tauri::State;

fn generated_dir(state: &AppState, scope: Scope) -> std::path::PathBuf {
    state.claude_dir(scope).join("hooks").join("_clman")
}

static PYTHON_BIN: OnceLock<String> = OnceLock::new();

/// Finds a working Python interpreter on this machine. Cached for the process
/// lifetime since it only depends on the local environment, not on any rule.
///
/// On Windows, `python3`/`python` can resolve to a non-functional "App
/// Execution Alias" stub when Python isn't actually installed — it still
/// spawns and exits (just with a Microsoft Store nag and a non-zero code),
/// so checking `.is_ok()` alone isn't enough: we need the exit code AND the
/// output to actually look like a Python version string.
fn detect_python() -> &'static str {
    PYTHON_BIN.get_or_init(|| {
        for candidate in ["python3", "python", "py"] {
            if let Ok(output) = silent_command(candidate).arg("--version").output() {
                if output.status.success() {
                    let combined = format!(
                        "{}{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    if combined.trim().starts_with("Python ") {
                        return candidate.to_string();
                    }
                }
            }
        }
        "python3".to_string()
    })
}

#[tauri::command]
pub fn list_hook_events() -> Vec<&'static str> {
    KNOWN_HOOK_EVENTS.to_vec()
}

fn str_field(hook: &Value, key: &str) -> String {
    hook.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn opt_str_field(hook: &Value, key: &str) -> Option<String> {
    hook.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn opt_str_list_field(hook: &Value, key: &str) -> Option<Vec<String>> {
    hook.get(key).and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect()
    })
}

/// `headers`/`input` llegan como objetos JSON reales — se muestran como
/// texto editable ("Clave: Valor" por línea, o JSON crudo para `input`).
fn headers_to_lines(hook: &Value) -> Option<String> {
    let obj = hook.get("headers")?.as_object()?;
    if obj.is_empty() {
        return None;
    }
    Some(
        obj.iter()
            .map(|(k, v)| format!("{k}: {}", v.as_str().unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn json_field_to_string(hook: &Value, key: &str) -> Option<String> {
    let v = hook.get(key)?;
    if v.is_null() {
        return None;
    }
    serde_json::to_string_pretty(v).ok()
}

#[tauri::command]
pub fn read_hooks(state: State<AppState>, scope: Scope) -> Result<HooksDocument, String> {
    read_hooks_impl(&state, scope)
}

fn read_hooks_impl(state: &AppState, scope: Scope) -> Result<HooksDocument, String> {
    let lang = state.lang();
    let path = settings_path(state, scope);
    let settings = load_settings(lang, &path)?;
    let mut rules = Vec::new();
    let mut errors = Vec::new();

    if let Some(Value::Object(hooks_obj)) = settings.get("hooks") {
        for (event, groups) in hooks_obj {
            let Some(groups) = groups.as_array() else {
                errors.push(i18n::hook_group_not_array(lang, event));
                continue;
            };
            for group in groups {
                let matcher = group
                    .get("matcher")
                    .and_then(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let Some(hook_list) = group.get("hooks").and_then(|h| h.as_array()) else {
                    errors.push(i18n::hook_group_missing_hooks(lang, event));
                    continue;
                };
                for hook in hook_list {
                    let hook_type = {
                        let t = str_field(hook, "type");
                        if t.is_empty() { "command".to_string() } else { t }
                    };
                    let common = HookRule {
                        title: str_field(hook, "_clmanTitle"),
                        description: str_field(hook, "_clmanDescription"),
                        event: event.clone(),
                        matcher: matcher.clone(),
                        hook_type: hook_type.clone(),
                        action: "command".to_string(),
                        command: String::new(),
                        message: String::new(),
                        timeout: hook.get("timeout").and_then(|t| t.as_u64()).map(|t| t as u32),
                        is_async: hook.get("async").and_then(|a| a.as_bool()).unwrap_or(false),
                        file_glob: String::new(),
                        if_filter: opt_str_field(hook, "if"),
                        status_message: opt_str_field(hook, "statusMessage"),
                        args: opt_str_list_field(hook, "args"),
                        async_rewake: hook.get("asyncRewake").and_then(|v| v.as_bool()),
                        shell: None,
                        on_failure: opt_str_field(hook, "onFailure"),
                        url: None,
                        headers: None,
                        allowed_env_vars: None,
                        server: None,
                        tool: None,
                        input: None,
                        prompt: None,
                        prompt_model: None,
                    };

                    rules.push(match hook_type.as_str() {
                        "http" => HookRule {
                            url: opt_str_field(hook, "url"),
                            headers: headers_to_lines(hook),
                            allowed_env_vars: opt_str_list_field(hook, "allowedEnvVars"),
                            ..common
                        },
                        "mcp_tool" => HookRule {
                            server: opt_str_field(hook, "server"),
                            tool: opt_str_field(hook, "tool"),
                            input: json_field_to_string(hook, "input"),
                            ..common
                        },
                        "prompt" | "agent" => HookRule {
                            prompt: opt_str_field(hook, "prompt"),
                            prompt_model: opt_str_field(hook, "model"),
                            ..common
                        },
                        _ => {
                            let action = {
                                let a = str_field(hook, "_clmanAction");
                                if a.is_empty() { "command".to_string() } else { a }
                            };
                            let inner_command = hook.get("_clmanInnerCommand").and_then(|c| c.as_str());
                            let command = inner_command
                                .map(|s| s.to_string())
                                .or_else(|| hook.get("command").and_then(|c| c.as_str()).map(|s| s.to_string()))
                                .unwrap_or_default();
                            let uses_wrapper = inner_command.is_some() || action == "message";
                            HookRule {
                                action: action.clone(),
                                command: if action == "message" { String::new() } else { command },
                                message: str_field(hook, "_clmanMessage"),
                                file_glob: str_field(hook, "_clmanFileGlob"),
                                // args/shell/onFailure solo se exponen cuando no hay wrapper
                                // de clman de por medio (ver save_hooks).
                                args: if uses_wrapper { None } else { opt_str_list_field(hook, "args") },
                                shell: if uses_wrapper { None } else { opt_str_field(hook, "shell") },
                                ..common
                            }
                        }
                    });
                }
            }
        }
    }

    Ok(HooksDocument { rules, errors })
}

fn py_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

fn py_list(items: &[String]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
}

fn write_wrapper(
    path: &std::path::Path,
    patterns: &[String],
    action: &str,
    inner_command: &str,
    message: &str,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let script = format!(
        r#"# Generado por clman. Se reescribe entero cada vez que guardás este hook: no lo edites a mano.
import sys, json, fnmatch, subprocess

PATTERNS = {patterns}
ACTION = {action}
INNER = {inner}
MESSAGE = {message}


def main():
    raw = sys.stdin.read()
    try:
        data = json.loads(raw) if raw.strip() else {{}}
    except Exception:
        data = {{}}
    tool_input = data.get("tool_input") or {{}}
    path = tool_input.get("file_path") or tool_input.get("path") or data.get("file_path") or ""

    if PATTERNS and path and not any(fnmatch.fnmatch(path, p) for p in PATTERNS):
        sys.exit(0)  # no matchea el patron de archivo: no hacemos nada

    if ACTION == "message":
        # systemMessage solo se le muestra al humano en la transcripcion —
        # Claude mismo nunca lo lee. Lo que si llega al contexto del modelo
        # es hookSpecificOutput.additionalContext, por eso va siempre junto.
        event_name = data.get("hook_event_name") or "PostToolUse"
        print(json.dumps({{
            "systemMessage": MESSAGE,
            "hookSpecificOutput": {{"hookEventName": event_name, "additionalContext": MESSAGE}},
        }}))
        sys.exit(0)

    if not INNER.strip():
        sys.exit(0)

    proc = subprocess.run(INNER, shell=True, input=raw, text=True)
    sys.exit(proc.returncode)


if __name__ == "__main__":
    main()
"#,
        patterns = py_list(patterns),
        action = py_str(action),
        inner = py_str(inner_command),
        message = py_str(message),
    );
    fs::write(path, script).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_hooks(
    state: State<AppState>,
    scope: Scope,
    rules: Vec<HookRule>,
) -> Result<HooksDocument, String> {
    save_hooks_impl(&state, scope, rules)
}

fn save_hooks_impl(state: &AppState, scope: Scope, rules: Vec<HookRule>) -> Result<HooksDocument, String> {
    let lang = state.lang();
    let mut all_errors = Vec::new();
    for r in &rules {
        let errs = validate_hook(lang, r);
        if !errs.is_empty() {
            let label = if r.title.trim().is_empty() { &r.event } else { &r.title };
            all_errors.push(format!("[{label}] {}", errs.join(" ")));
        }
    }
    if !all_errors.is_empty() {
        return Err(all_errors.join("\n"));
    }

    let path = settings_path(state, scope);
    let mut settings = load_settings(lang, &path)?;

    // Wipe and regenerate wrapper scripts fresh on every save so removed /
    // edited rules don't leave stale scripts behind.
    let gen_dir = generated_dir(state, scope);
    if gen_dir.exists() {
        fs::remove_dir_all(&gen_dir).map_err(|e| e.to_string())?;
    }

    // Group rules by event, then by matcher, preserving first-seen order.
    let mut event_order: Vec<String> = Vec::new();
    let mut by_event: std::collections::HashMap<String, Vec<(String, Vec<&HookRule>)>> =
        std::collections::HashMap::new();

    for rule in &rules {
        if !event_order.contains(&rule.event) {
            event_order.push(rule.event.clone());
        }
        let groups = by_event.entry(rule.event.clone()).or_default();
        if let Some(group) = groups.iter_mut().find(|(m, _)| m == &rule.matcher) {
            group.1.push(rule);
        } else {
            groups.push((rule.matcher.clone(), vec![rule]));
        }
    }

    let mut hooks_obj = Map::new();
    let mut wrapper_idx = 0usize;
    for event in &event_order {
        let groups = &by_event[event];
        let mut group_values = Vec::new();
        for (matcher, hook_rules) in groups {
            let hooks_array: Vec<Value> = hook_rules
                .iter()
                .map(|r| -> Result<Value, String> {
                    let hook_type = r.hook_type.as_str();
                    let mut obj = Map::new();
                    obj.insert("type".into(), Value::String(hook_type.to_string()));

                    match hook_type {
                        "http" => {
                            obj.insert("url".into(), Value::String(r.url.clone().unwrap_or_default()));
                            if let Some(headers) = r.headers.as_deref().filter(|s| !s.trim().is_empty()) {
                                let mut map = Map::new();
                                for line in headers.lines() {
                                    if let Some((k, v)) = line.split_once(':') {
                                        map.insert(k.trim().to_string(), Value::String(v.trim().to_string()));
                                    }
                                }
                                obj.insert("headers".into(), Value::Object(map));
                            }
                            if let Some(vars) = r.allowed_env_vars.as_ref().filter(|v| !v.is_empty()) {
                                obj.insert(
                                    "allowedEnvVars".into(),
                                    Value::Array(vars.iter().cloned().map(Value::String).collect()),
                                );
                            }
                            if let Some(of) = r.on_failure.as_deref().filter(|s| !s.trim().is_empty()) {
                                obj.insert("onFailure".into(), Value::String(of.to_string()));
                            }
                        }
                        "mcp_tool" => {
                            obj.insert("server".into(), Value::String(r.server.clone().unwrap_or_default()));
                            obj.insert("tool".into(), Value::String(r.tool.clone().unwrap_or_default()));
                            if let Some(input) = r.input.as_deref().filter(|s| !s.trim().is_empty()) {
                                let parsed: Value = serde_json::from_str(input)
                                    .map_err(|e| i18n::hook_input_invalid_json(lang, &r.title, &e.to_string()))?;
                                obj.insert("input".into(), parsed);
                            }
                        }
                        "prompt" | "agent" => {
                            obj.insert("prompt".into(), Value::String(r.prompt.clone().unwrap_or_default()));
                            if let Some(m) = r.prompt_model.as_deref().filter(|s| !s.trim().is_empty()) {
                                obj.insert("model".into(), Value::String(m.to_string()));
                            }
                        }
                        _ => {
                            let glob = r.file_glob.trim();
                            let needs_wrapper = r.action == "message" || !glob.is_empty();

                            if !needs_wrapper {
                                obj.insert("command".into(), Value::String(r.command.clone()));
                                if let Some(args) = r.args.as_ref().filter(|a| !a.is_empty()) {
                                    obj.insert(
                                        "args".into(),
                                        Value::Array(args.iter().cloned().map(Value::String).collect()),
                                    );
                                }
                                if let Some(shell) = r.shell.as_deref().filter(|s| !s.trim().is_empty()) {
                                    obj.insert("shell".into(), Value::String(shell.to_string()));
                                }
                                if let Some(ar) = r.async_rewake {
                                    obj.insert("asyncRewake".into(), Value::Bool(ar));
                                }
                                if let Some(of) = r.on_failure.as_deref().filter(|s| !s.trim().is_empty()) {
                                    obj.insert("onFailure".into(), Value::String(of.to_string()));
                                }
                            } else {
                                wrapper_idx += 1;
                                let patterns: Vec<String> = glob
                                    .split('|')
                                    .map(|p| p.trim().to_string())
                                    .filter(|p| !p.is_empty())
                                    .collect();
                                let wrapper_path = gen_dir.join(format!("hook-{wrapper_idx}.py"));
                                write_wrapper(&wrapper_path, &patterns, &r.action, &r.command, &r.message)?;
                                let python = detect_python();
                                obj.insert(
                                    "command".into(),
                                    Value::String(format!("{python} \"{}\"", wrapper_path.display())),
                                );
                                if r.action == "command" {
                                    obj.insert("_clmanInnerCommand".into(), Value::String(r.command.clone()));
                                }
                                if !glob.is_empty() {
                                    obj.insert("_clmanFileGlob".into(), Value::String(glob.to_string()));
                                }
                            }

                            obj.insert("_clmanAction".into(), Value::String(r.action.clone()));
                            if r.action == "message" {
                                obj.insert("_clmanMessage".into(), Value::String(r.message.clone()));
                            }
                            if r.is_async {
                                obj.insert("async".into(), Value::Bool(true));
                            }
                        }
                    }

                    if let Some(f) = r.if_filter.as_deref().filter(|s| !s.trim().is_empty()) {
                        obj.insert("if".into(), Value::String(f.to_string()));
                    }
                    if let Some(sm) = r.status_message.as_deref().filter(|s| !s.trim().is_empty()) {
                        obj.insert("statusMessage".into(), Value::String(sm.to_string()));
                    }
                    if !r.title.trim().is_empty() {
                        obj.insert("_clmanTitle".into(), Value::String(r.title.clone()));
                    }
                    if !r.description.trim().is_empty() {
                        obj.insert("_clmanDescription".into(), Value::String(r.description.clone()));
                    }
                    if let Some(t) = r.timeout {
                        obj.insert("timeout".into(), Value::from(t));
                    }
                    Ok(Value::Object(obj))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut group_obj = Map::new();
            if !matcher.is_empty() {
                group_obj.insert("matcher".into(), Value::String(matcher.clone()));
            }
            group_obj.insert("hooks".into(), Value::Array(hooks_array));
            group_values.push(Value::Object(group_obj));
        }
        hooks_obj.insert(event.clone(), Value::Array(group_values));
    }

    if hooks_obj.is_empty() {
        settings.remove("hooks");
    } else {
        settings.insert("hooks".into(), Value::Object(hooks_obj));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&Value::Object(settings)).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| e.to_string())?;

    read_hooks_impl(state, scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::process::{Command, Stdio};

    fn run_wrapper(script_path: &std::path::Path, stdin_json: &str) -> (i32, String) {
        let python = detect_python();
        let mut child = Command::new(python)
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("python should be available (checked by detect_python)");
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin_json.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        let code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !output.stderr.is_empty() {
            eprintln!("wrapper stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        (code, stdout)
    }

    #[test]
    fn file_glob_filter_skips_non_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hook.py");
        write_wrapper(
            &script,
            &["*.java".to_string()],
            "command",
            "echo RAN_INNER_COMMAND",
            "",
        )
        .unwrap();

        let (code, stdout) = run_wrapper(
            &script,
            r#"{"tool_input": {"file_path": "C:\\proj\\src\\Thing.txt"}}"#,
        );
        assert_eq!(code, 0, "non-matching file must exit 0 without running the command");
        assert!(stdout.is_empty(), "non-matching file must not run the inner command, got: {stdout}");
    }

    #[test]
    fn file_glob_filter_runs_inner_command_on_match() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hook.py");
        write_wrapper(
            &script,
            &["*.java".to_string()],
            "command",
            "echo RAN_INNER_COMMAND",
            "",
        )
        .unwrap();

        let (code, stdout) = run_wrapper(
            &script,
            r#"{"tool_input": {"file_path": "C:\\proj\\src\\Thing.java"}}"#,
        );
        assert_eq!(code, 0);
        assert!(
            stdout.contains("RAN_INNER_COMMAND"),
            "matching .java file must run the inner command, got: {stdout}"
        );
    }

    #[test]
    fn message_action_prints_system_message_json_without_running_anything() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hook.py");
        write_wrapper(&script, &[], "message", "", "Seguí la arquitectura hexagonal").unwrap();

        let (code, stdout) = run_wrapper(&script, r#"{"tool_name": "Write", "hook_event_name": "PostToolUse"}"#);
        assert_eq!(code, 0);
        let parsed: Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("stdout should be valid JSON, got {stdout:?}: {e}"));
        assert_eq!(
            parsed.get("systemMessage").and_then(|v| v.as_str()),
            Some("Seguí la arquitectura hexagonal"),
            "systemMessage sigue yendo para que el humano lo vea en la transcripción"
        );
        assert_eq!(
            parsed
                .pointer("/hookSpecificOutput/additionalContext")
                .and_then(|v| v.as_str()),
            Some("Seguí la arquitectura hexagonal"),
            "additionalContext es lo que de verdad llega al contexto de Claude — systemMessage solo es visual"
        );
        assert_eq!(
            parsed.pointer("/hookSpecificOutput/hookEventName").and_then(|v| v.as_str()),
            Some("PostToolUse")
        );
    }

    #[test]
    fn message_action_falls_back_to_post_tool_use_when_event_name_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hook.py");
        write_wrapper(&script, &[], "message", "", "hola").unwrap();

        let (code, stdout) = run_wrapper(&script, "{}");
        assert_eq!(code, 0);
        let parsed: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(
            parsed.pointer("/hookSpecificOutput/hookEventName").and_then(|v| v.as_str()),
            Some("PostToolUse")
        );
    }

    #[test]
    fn message_with_quotes_and_newlines_does_not_break_the_generated_script() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hook.py");
        let tricky = "Decí \"hola\"\ny esto\tcon tab";
        write_wrapper(&script, &[], "message", "", tricky).unwrap();

        let (code, stdout) = run_wrapper(&script, "{}");
        assert_eq!(code, 0, "a message with quotes/newlines must not break the script");
        let parsed: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(parsed.get("systemMessage").and_then(|v| v.as_str()), Some(tricky));
    }

    #[test]
    fn command_with_no_file_glob_runs_unconditionally() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hook.py");
        write_wrapper(&script, &[], "command", "echo ALWAYS_RUNS", "").unwrap();

        let (code, stdout) = run_wrapper(&script, "{}");
        assert_eq!(code, 0);
        assert!(stdout.contains("ALWAYS_RUNS"));
    }

    fn fake_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path().join("project");
        let global_root = dir.path().join("global");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&global_root).unwrap();
        (dir, AppState::new(project_root, global_root))
    }

    fn empty_hook(event: &str, hook_type: &str) -> HookRule {
        HookRule {
            title: String::new(),
            description: String::new(),
            event: event.to_string(),
            matcher: String::new(),
            hook_type: hook_type.to_string(),
            action: "command".to_string(),
            command: String::new(),
            message: String::new(),
            timeout: None,
            is_async: false,
            file_glob: String::new(),
            if_filter: None,
            status_message: None,
            args: None,
            async_rewake: None,
            shell: None,
            on_failure: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            server: None,
            tool: None,
            input: None,
            prompt: None,
            prompt_model: None,
        }
    }

    #[test]
    fn http_hook_round_trips_through_save_and_read() {
        let (_g, state) = fake_state();
        let hook = HookRule {
            title: "Avisar al webhook".into(),
            url: Some("https://example.com/hook".into()),
            headers: Some("Authorization: Bearer abc\nX-Team: platform".into()),
            allowed_env_vars: Some(vec!["GITHUB_TOKEN".into()]),
            on_failure: Some("block".into()),
            if_filter: Some("Bash(git push *)".into()),
            status_message: Some("Avisando...".into()),
            ..empty_hook("PreToolUse", "http")
        };
        save_hooks_impl(&state, Scope::Project, vec![hook]).expect("http hook should save");

        let doc = read_hooks_impl(&state, Scope::Project).unwrap();
        assert_eq!(doc.rules.len(), 1);
        let r = &doc.rules[0];
        assert_eq!(r.hook_type, "http");
        assert_eq!(r.url.as_deref(), Some("https://example.com/hook"));
        assert_eq!(r.headers.as_deref(), Some("Authorization: Bearer abc\nX-Team: platform"));
        assert_eq!(r.allowed_env_vars, Some(vec!["GITHUB_TOKEN".to_string()]));
        assert_eq!(r.on_failure.as_deref(), Some("block"));
        assert_eq!(r.if_filter.as_deref(), Some("Bash(git push *)"));
        assert_eq!(r.status_message.as_deref(), Some("Avisando..."));
    }

    #[test]
    fn mcp_tool_hook_round_trips_through_save_and_read() {
        let (_g, state) = fake_state();
        let hook = HookRule {
            server: Some("github".into()),
            tool: Some("create_issue".into()),
            input: Some(r#"{"title": "${tool_input.file_path}"}"#.into()),
            ..empty_hook("PostToolUse", "mcp_tool")
        };
        save_hooks_impl(&state, Scope::Project, vec![hook]).expect("mcp_tool hook should save");

        let doc = read_hooks_impl(&state, Scope::Project).unwrap();
        let r = &doc.rules[0];
        assert_eq!(r.hook_type, "mcp_tool");
        assert_eq!(r.server.as_deref(), Some("github"));
        assert_eq!(r.tool.as_deref(), Some("create_issue"));
        assert!(r.input.as_deref().unwrap().contains("tool_input.file_path"));
    }

    #[test]
    fn prompt_hook_round_trips_through_save_and_read() {
        let (_g, state) = fake_state();
        let hook = HookRule {
            prompt: Some("¿Este comando es seguro? $ARGUMENTS".into()),
            prompt_model: Some("haiku".into()),
            ..empty_hook("PreToolUse", "prompt")
        };
        save_hooks_impl(&state, Scope::Project, vec![hook]).expect("prompt hook should save");

        let doc = read_hooks_impl(&state, Scope::Project).unwrap();
        let r = &doc.rules[0];
        assert_eq!(r.hook_type, "prompt");
        assert_eq!(r.prompt.as_deref(), Some("¿Este comando es seguro? $ARGUMENTS"));
        assert_eq!(r.prompt_model.as_deref(), Some("haiku"));
    }

    #[test]
    fn command_hook_with_args_shell_and_on_failure_round_trips() {
        let (_g, state) = fake_state();
        let hook = HookRule {
            command: "./check.sh".into(),
            args: Some(vec!["--strict".into()]),
            shell: Some("powershell".into()),
            on_failure: Some("block".into()),
            async_rewake: Some(true),
            is_async: true,
            ..empty_hook("PostToolUse", "command")
        };
        save_hooks_impl(&state, Scope::Project, vec![hook]).expect("command hook should save");

        let doc = read_hooks_impl(&state, Scope::Project).unwrap();
        let r = &doc.rules[0];
        assert_eq!(r.hook_type, "command");
        assert_eq!(r.command, "./check.sh");
        assert_eq!(r.args, Some(vec!["--strict".to_string()]));
        assert_eq!(r.shell.as_deref(), Some("powershell"));
        assert_eq!(r.on_failure.as_deref(), Some("block"));
        assert_eq!(r.async_rewake, Some(true));
        assert!(r.is_async);
    }

    #[test]
    fn http_hook_without_url_is_rejected() {
        let (_g, state) = fake_state();
        let hook = empty_hook("PreToolUse", "http");
        let err = save_hooks_impl(&state, Scope::Project, vec![hook])
            .expect_err("http hook without a url must be rejected");
        assert!(err.contains("url"));
    }
}
