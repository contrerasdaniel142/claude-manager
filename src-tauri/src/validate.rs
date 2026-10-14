use crate::i18n;
use crate::models::Lang;
use regex::Regex;
use std::sync::OnceLock;

static SLUG_RE: OnceLock<Regex> = OnceLock::new();

/// Claude Code requires agent/skill identifiers to be lowercase letters,
/// numbers and hyphens, no leading/trailing hyphen, no double hyphen.
pub fn valid_slug(s: &str) -> bool {
    let re = SLUG_RE.get_or_init(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());
    !s.is_empty() && re.is_match(s)
}

pub fn slugify(s: &str) -> String {
    let lowered = s.to_lowercase();
    let mut out = String::new();
    let mut last_was_dash = false;
    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !out.is_empty() {
            out.push('-');
            last_was_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

pub const PERMISSION_MODES: &[&str] = &[
    "default",
    "acceptEdits",
    "auto",
    "dontAsk",
    "bypassPermissions",
    "plan",
    "manual",
];

pub const AGENT_COLORS: &[&str] = &[
    "red", "blue", "green", "yellow", "purple", "orange", "pink", "cyan",
];

pub fn validate_agent(lang: Lang, name: &str, description: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if name.trim().is_empty() {
        errors.push(i18n::agent_needs_name(lang));
    } else if !valid_slug(name) {
        errors.push(i18n::name_slug_format_long(lang));
    } else if name.len() > 256 {
        errors.push(i18n::name_too_long(lang, 256));
    }
    if description.trim().is_empty() {
        errors.push(i18n::agent_needs_description(lang));
    }
    errors
}

pub fn validate_skill(lang: Lang, name: &str, _description: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if !name.trim().is_empty() {
        if name.len() > 64 {
            errors.push(i18n::name_too_long(lang, 64));
        }
        if !valid_slug(name) {
            errors.push(i18n::name_slug_format_short(lang));
        }
    }
    errors
}

// Los 33 eventos documentados en code.claude.com/docs/en/hooks (verificado
// contra la doc en vivo, no de memoria). Orden: ciclo de vida de sesión,
// prompt, herramientas, agentes/tareas, archivos/workspace, compactación,
// modelo, MCP.
pub const KNOWN_HOOK_EVENTS: &[&str] = &[
    "SessionStart",
    "SessionEnd",
    "Setup",
    "UserPromptSubmit",
    "UserPromptExpansion",
    "PreToolUse",
    "PermissionRequest",
    "PermissionDenied",
    "PostToolUse",
    "PostToolUseFailure",
    "PostToolBatch",
    "Notification",
    "MessageDisplay",
    "SubagentStart",
    "SubagentStop",
    "TaskCreated",
    "TaskCompleted",
    "Stop",
    "StopFailure",
    "TeammateIdle",
    "InstructionsLoaded",
    "ConfigChange",
    "CwdChanged",
    "DirectoryAdded",
    "FileChanged",
    "WorktreeCreate",
    "WorktreeRemove",
    "PreCompact",
    "PostCompact",
    "PreModelSwitch",
    "PostModelSwitch",
    "Elicitation",
    "ElicitationResult",
];

pub fn validate_hook(lang: Lang, hook: &crate::models::HookRule) -> Vec<String> {
    let mut errors = Vec::new();
    if hook.event.trim().is_empty() {
        errors.push(i18n::pick_hook_event(lang));
    } else if !KNOWN_HOOK_EVENTS.contains(&hook.event.as_str()) {
        errors.push(i18n::unknown_hook_event(lang, &hook.event));
    }
    match hook.hook_type.as_str() {
        "http" => {
            if hook.url.as_deref().unwrap_or("").trim().is_empty() {
                errors.push(i18n::http_hook_needs_url(lang));
            }
        }
        "mcp_tool" => {
            if hook.server.as_deref().unwrap_or("").trim().is_empty()
                || hook.tool.as_deref().unwrap_or("").trim().is_empty()
            {
                errors.push(i18n::mcp_tool_hook_needs_server_and_tool(lang));
            }
        }
        "prompt" | "agent" => {
            if hook.prompt.as_deref().unwrap_or("").trim().is_empty() {
                errors.push(i18n::prompt_hook_needs_prompt(lang));
            }
        }
        _ => match hook.action.as_str() {
            "message" => {
                if hook.message.trim().is_empty() {
                    errors.push(i18n::message_action_needs_message(lang));
                }
            }
            _ => {
                if hook.command.trim().is_empty() {
                    errors.push(i18n::command_action_needs_command(lang));
                }
            }
        },
    }
    errors
}

pub fn validate_mcp_server(lang: Lang, entry: &crate::models::McpServerFile) -> Vec<String> {
    let mut errors = Vec::new();
    if entry.name.trim().is_empty() {
        errors.push(i18n::mcp_server_needs_name(lang));
    }
    match entry.server_type.as_str() {
        "http" | "sse" | "streamable-http" | "ws" => {
            if entry.url.as_deref().unwrap_or("").trim().is_empty() {
                errors.push(i18n::mcp_server_needs_url(lang));
            }
        }
        _ => {
            if entry.command.as_deref().unwrap_or("").trim().is_empty() {
                errors.push(i18n::mcp_server_needs_command(lang));
            }
        }
    }
    errors
}
