use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Project,
    Global,
}

/// Idioma de la UI — el frontend lo manda una vez al arrancar (según lo que
/// tenga guardado en localStorage) y de nuevo cada vez que el usuario lo
/// cambia. Los mensajes de error que arma el backend lo leen de acá para
/// no tener que mandar el idioma en cada llamada por separado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    Es,
}

#[derive(Clone)]
pub struct AppState {
    /// Mutable so the project selector can switch it at runtime without
    /// restarting the app — every command already reads this fresh on each
    /// call via `project_root()`/`claude_dir()`, so nothing else has to
    /// change for a switch to take effect immediately.
    project_root: Arc<Mutex<PathBuf>>,
    pub global_root: PathBuf,
    pub lang: Arc<Mutex<Lang>>,
    /// Bumped by `watcher::start` every time it's called — lets an older
    /// watcher thread notice (next time its channel wakes it) that a newer
    /// one has taken over and stop itself instead of running forever.
    watcher_generation: Arc<Mutex<u64>>,
    /// Parsed once per "cold" fetch from `claude plugin list --json --available`
    /// (which can be a multi-MB catalog) and reused across every keystroke of
    /// the Available-plugins search instead of re-running that CLI call and
    /// re-shipping the whole thing to the frontend each time. Cleared whenever
    /// a marketplace is added/removed, since that changes what's available.
    available_plugins_cache: Arc<Mutex<Option<Vec<PluginSummary>>>>,
}

impl AppState {
    pub fn new(project_root: PathBuf, global_root: PathBuf) -> Self {
        AppState {
            project_root: Arc::new(Mutex::new(project_root)),
            global_root,
            lang: Default::default(),
            watcher_generation: Arc::new(Mutex::new(0)),
            available_plugins_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn cached_available_plugins(&self) -> Option<Vec<PluginSummary>> {
        self.available_plugins_cache.lock().unwrap().clone()
    }

    pub fn set_available_plugins_cache(&self, plugins: Vec<PluginSummary>) {
        *self.available_plugins_cache.lock().unwrap() = Some(plugins);
    }

    pub fn clear_available_plugins_cache(&self) {
        *self.available_plugins_cache.lock().unwrap() = None;
    }

    pub fn project_root(&self) -> PathBuf {
        self.project_root.lock().unwrap().clone()
    }

    pub fn set_project_root(&self, path: PathBuf) {
        *self.project_root.lock().unwrap() = path;
    }

    pub fn next_watcher_generation(&self) -> u64 {
        let mut g = self.watcher_generation.lock().unwrap();
        *g += 1;
        *g
    }

    pub fn watcher_generation(&self) -> u64 {
        *self.watcher_generation.lock().unwrap()
    }

    /// `.claude` directory for the given scope (created on demand by callers, not here).
    pub fn claude_dir(&self, scope: Scope) -> PathBuf {
        match scope {
            Scope::Project => self.project_root().join(".claude"),
            Scope::Global => self.global_root.clone(),
        }
    }

    pub fn lang(&self) -> Lang {
        *self.lang.lock().unwrap()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, rename = "disallowedTools", skip_serializing_if = "Option::is_none")]
    pub disallowed_tools: Option<String>,
    #[serde(default, rename = "permissionMode", skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, rename = "maxTurns", skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(default, rename = "omitClaudeMd", skip_serializing_if = "Option::is_none")]
    pub omit_claude_md: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isolation: Option<String>,
    #[serde(default, rename = "initialPrompt", skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    /// Passthrough — hoy el único campo documentado es `cacheTtl`, pero se
    /// guarda el mapa entero para no perder nada si Anthropic agrega más.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
    #[serde(default, rename = "mcpServers", skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<serde_json::Value>,
    /// Hooks propios del agente (mismo formato que settings.json) — passthrough,
    /// clman no los edita campo por campo, se edita como bloque YAML/JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,
}

/// Input de `save_agent` empaquetado en un solo struct — con 18 campos,
/// seguir agregando parámetros sueltos a la función ya no es razonable.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAgentInput {
    pub scope: Scope,
    pub original_id: Option<String>,
    pub name: String,
    pub description: String,
    pub tools: Option<String>,
    pub model: Option<String>,
    pub disallowed_tools: Option<String>,
    pub permission_mode: Option<String>,
    pub color: Option<String>,
    pub max_turns: Option<u32>,
    pub skills: Option<Vec<String>>,
    pub memory: Option<String>,
    pub background: Option<bool>,
    pub omit_claude_md: Option<bool>,
    pub effort: Option<String>,
    pub isolation: Option<String>,
    pub initial_prompt: Option<String>,
    pub experimental: Option<serde_json::Value>,
    pub mcp_servers: Option<serde_json::Value>,
    pub hooks: Option<serde_json::Value>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: Scope,
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFile {
    pub id: String,
    pub scope: Scope,
    pub name: String,
    pub description: String,
    pub tools: Option<String>,
    pub model: Option<String>,
    pub disallowed_tools: Option<String>,
    pub permission_mode: Option<String>,
    pub color: Option<String>,
    pub max_turns: Option<u32>,
    pub skills: Option<Vec<String>>,
    pub memory: Option<String>,
    pub background: Option<bool>,
    pub omit_claude_md: Option<bool>,
    pub effort: Option<String>,
    pub isolation: Option<String>,
    pub initial_prompt: Option<String>,
    pub experimental: Option<serde_json::Value>,
    pub mcp_servers: Option<serde_json::Value>,
    pub hooks: Option<serde_json::Value>,
    pub body: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when_to_use: Option<String>,
    #[serde(
        default,
        rename = "allowed-tools",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_tools: Option<String>,
    // "paths" se maneja a mano (ver fsutil::extract_list_field), no vive en
    // este struct: igual que en Rules, acepta lista YAML o string con comas,
    // algo que el derive de serde no puede representar en un solo campo tipado.
    #[serde(default, rename = "argument-hint", skip_serializing_if = "Option::is_none")]
    pub argument_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(default, rename = "disable-model-invocation", skip_serializing_if = "Option::is_none")]
    pub disable_model_invocation: Option<bool>,
    #[serde(default, rename = "user-invocable", skip_serializing_if = "Option::is_none")]
    pub user_invocable: Option<bool>,
    #[serde(default, rename = "disallowed-tools", skip_serializing_if = "Option::is_none")]
    pub disallowed_tools: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSkillInput {
    pub scope: Scope,
    pub original_id: Option<String>,
    pub name: String,
    pub description: String,
    pub when_to_use: Option<String>,
    pub allowed_tools: Option<String>,
    pub paths: Vec<String>,
    pub argument_hint: Option<String>,
    pub arguments: Option<String>,
    pub disable_model_invocation: Option<bool>,
    pub user_invocable: Option<bool>,
    pub disallowed_tools: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub context: Option<String>,
    pub agent: Option<String>,
    pub background: Option<bool>,
    pub hooks: Option<serde_json::Value>,
    pub shell: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: Scope,
    pub valid: bool,
    pub errors: Vec<String>,
    pub extra_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFile {
    pub id: String,
    pub scope: Scope,
    pub name: String,
    pub description: String,
    pub when_to_use: Option<String>,
    pub allowed_tools: Option<String>,
    pub paths: Vec<String>,
    pub argument_hint: Option<String>,
    pub arguments: Option<String>,
    pub disable_model_invocation: Option<bool>,
    pub user_invocable: Option<bool>,
    pub disallowed_tools: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub context: Option<String>,
    pub agent: Option<String>,
    pub background: Option<bool>,
    pub hooks: Option<serde_json::Value>,
    pub shell: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub body: String,
    pub extra_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRule {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub event: String,
    #[serde(default)]
    pub matcher: String,
    /// Tipo real de Claude Code: "command" | "http" | "mcp_tool" | "prompt" | "agent".
    #[serde(default = "default_hook_type")]
    pub hook_type: String,
    /// Dentro de hook_type == "command": "command" (corre un comando real) o
    /// "message" (le agrega texto al contexto sin ejecutar nada) — abstracción
    /// propia de clman, no existe como tal en Claude Code.
    #[serde(default = "default_action")]
    pub action: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(default)]
    pub is_async: bool,
    /// Patrones de archivo (ej "*.java") separados por "|". Vacío = sin filtro.
    /// No existe en Claude Code: lo resolvemos generando un script intermedio.
    #[serde(default)]
    pub file_glob: String,
    // Comunes a cualquier hook_type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub if_filter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    // hook_type == "command", extras sobre action/command/timeout/is_async.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_rewake: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_failure: Option<String>,
    // hook_type == "http"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Líneas "Clave: Valor" — se parsean a un objeto al guardar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_env_vars: Option<Vec<String>>,
    // hook_type == "mcp_tool"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    // hook_type == "prompt" | "agent"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_model: Option<String>,
}

fn default_action() -> String {
    "command".to_string()
}

fn default_hook_type() -> String {
    "command".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksDocument {
    pub rules: Vec<HookRule>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringSummary {
    pub rel_path: String,
    pub scope: Scope,
    pub title: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringFile {
    pub rel_path: String,
    pub scope: Scope,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSummary {
    pub id: String,
    pub scope: Scope,
    pub title: String,
    pub paths: Vec<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFile {
    pub id: String,
    pub scope: Scope,
    pub paths: Vec<String>,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputStyleFrontmatter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, rename = "keep-coding-instructions", skip_serializing_if = "Option::is_none")]
    pub keep_coding_instructions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStyleSummary {
    pub id: String,
    pub scope: Scope,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsSettings {
    pub allow: Vec<String>,
    pub ask: Vec<String>,
    pub deny: Vec<String>,
    pub additional_directories: Vec<String>,
    pub default_mode: Option<String>,
    pub disable_bypass_permissions_mode: bool,
    pub disable_auto_mode: bool,
    pub block_reads_outside_working_directories: bool,
    /// These four live under the top-level `attribution`/`includeGitInstructions`
    /// keys, siblings of `permissions` in settings.json — not nested under
    /// it — but share this one settings screen since they're the same kind
    /// of "how Claude is allowed to behave" setting.
    pub attribution_commit: Option<String>,
    pub attribution_pr: Option<String>,
    pub attribution_session_url: Option<bool>,
    pub include_git_instructions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerSummary {
    pub name: String,
    pub scope: Scope,
    pub server_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerFile {
    pub name: String,
    pub scope: Scope,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: Option<String>,
    pub url: Option<String>,
    pub headers: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMcpServerInput {
    pub scope: Scope,
    pub original_name: Option<String>,
    pub name: String,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: Option<String>,
    pub url: Option<String>,
    pub headers: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketplaceSummary {
    pub name: String,
    pub source: String,
    pub repo: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginSummary {
    pub id: String,
    pub name: String,
    pub marketplace: String,
    pub version: String,
    pub scope: String,
    pub enabled: bool,
    pub installed: bool,
    pub description: Option<String>,
}

/// One page of a server-side-filtered `available` search — never the whole
/// catalog, so neither the IPC bridge nor the frontend's React state ever
/// holds more than `limit` entries at a time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginSearchPage {
    pub items: Vec<PluginSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstallResult {
    pub status: String,
    pub message: String,
    pub command: Option<String>,
    pub kind: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStyleFile {
    pub id: String,
    pub scope: Scope,
    pub name: String,
    pub description: String,
    pub keep_coding_instructions: bool,
    pub body: String,
}
