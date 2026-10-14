// Herramientas built-in más comunes de Claude Code. Las de servidores MCP
// tienen nombres propios (ej. mcp__nombre-server__accion) y no están acá,
// pero se pueden escribir igual a mano en el campo.
export const BUILTIN_TOOLS = [
  "Bash",
  "Read",
  "Edit",
  "Write",
  "Glob",
  "Grep",
  "WebFetch",
  "WebSearch",
  "NotebookEdit",
  "Task",
  "TodoWrite",
  "SlashCommand",
  "BashOutput",
  "KillShell",
];

// Eventos cuyo matcher filtra por nombre de herramienta (los únicos para
// los que tiene sentido nuestro filtro extra "solo si el archivo coincide
// con", porque Claude Code no sabe filtrar por extensión de archivo ahí).
const TOOL_USE_EVENTS = new Set([
  "PreToolUse",
  "PostToolUse",
  "PostToolUseFailure",
  "PermissionRequest",
  "PermissionDenied",
]);

// Verificado contra code.claude.com/docs/en/hooks (33 eventos en total).
// Los que casi nadie necesita (worktrees, cambio de modelo, elicitation de
// MCP) quedan en "Avanzado" para no saturar el dropdown.
// `key` identifica el grupo para buscar su label traducido en strings.ts
// (t.hooks.eventGroupLabels) — no es texto para mostrar directamente.
export type EventGroupKey = "tools" | "files" | "prompt" | "advanced";

export const EVENT_GROUPS: { key: EventGroupKey; events: string[] }[] = [
  { key: "tools", events: ["PreToolUse", "PostToolUse", "PostToolUseFailure", "PermissionRequest", "PermissionDenied"] },
  { key: "files", events: ["FileChanged", "CwdChanged", "DirectoryAdded"] },
  {
    key: "prompt",
    events: [
      "UserPromptSubmit",
      "UserPromptExpansion",
      "SessionStart",
      "SessionEnd",
      "Stop",
      "StopFailure",
      "SubagentStart",
      "SubagentStop",
      "Notification",
      "PreCompact",
      "PostCompact",
      "InstructionsLoaded",
    ],
  },
  {
    key: "advanced",
    events: [
      "Setup",
      "PostToolBatch",
      "MessageDisplay",
      "TaskCreated",
      "TaskCompleted",
      "TeammateIdle",
      "ConfigChange",
      "WorktreeCreate",
      "WorktreeRemove",
      "PreModelSwitch",
      "PostModelSwitch",
      "Elicitation",
      "ElicitationResult",
    ],
  },
];

export function matcherIsToolBased(event: string): boolean {
  return TOOL_USE_EVENTS.has(event);
}

// Qué tipo de valor espera el matcher de cada evento, para dar la ayuda y
// las sugerencias correctas en vez de un genérico "nombre de herramienta".
export type MatcherKind = "tool" | "literal-file" | "keyword" | "none";

const KEYWORD_MATCHERS: Record<string, string[]> = {
  SessionStart: ["startup", "resume", "clear", "compact", "fork"],
  Setup: ["init", "maintenance"],
  UserPromptExpansion: [], // nombre del slash command, no hay lista fija
  SubagentStart: [],
  SubagentStop: [],
  Notification: ["permission_prompt", "idle_prompt"],
  StopFailure: ["rate_limit"],
  InstructionsLoaded: ["session_start", "path_glob_match"],
  ConfigChange: ["user_settings", "project_settings", "local_settings", "policy_settings", "skills"],
  DirectoryAdded: ["slash_command", "register_repo_root"],
  SessionEnd: ["clear", "resume", "logout", "prompt_input_exit", "other"],
  PreCompact: ["manual", "auto"],
  PostCompact: ["manual", "auto"],
  PreModelSwitch: [],
  PostModelSwitch: [],
  Elicitation: [],
  ElicitationResult: [],
};

const NO_MATCHER_EVENTS = new Set([
  "UserPromptSubmit",
  "MessageDisplay",
  "PostToolBatch",
  "TaskCreated",
  "TaskCompleted",
  "TeammateIdle",
  "Stop",
  "CwdChanged",
  "WorktreeCreate",
  "WorktreeRemove",
]);

export function matcherKind(event: string): MatcherKind {
  if (TOOL_USE_EVENTS.has(event)) return "tool";
  if (event === "FileChanged") return "literal-file";
  if (NO_MATCHER_EVENTS.has(event)) return "none";
  return "keyword";
}

// Eventos donde la doc da una tabla cerrada y completa de valores válidos
// (sin "ej."/"e.g." que sugiera que hay más) — ahí no tiene sentido dejar
// escribir cualquier cosa: un valor no eres de la lista nunca va a matchear
// nada, y puede meter el matcher en modo regex por accidente. Se muestran
// como chips para tocar, no como texto libre.
const CLOSED_SET_EVENTS = new Set([
  "SessionStart",
  "Setup",
  "ConfigChange",
  "DirectoryAdded",
  "PreCompact",
  "PostCompact",
  "SessionEnd",
]);

export function matcherIsClosedSet(event: string): boolean {
  return CLOSED_SET_EVENTS.has(event);
}

// Las descripciones "cuándo dispara" por evento viven en strings.ts
// (t.hooks.eventDescriptions), porque son texto traducible — acá solo
// quedan datos/lógica independientes del idioma.

export function matcherSuggestions(event: string): string[] {
  // No sugerimos "*": la doc dice que "*", vacío y omitido son exactamente
  // lo mismo ("match all"), así que agregarlo como tag no suma nada —
  // dejar el matcher vacío ya significa "siempre".
  if (TOOL_USE_EVENTS.has(event)) return [...BUILTIN_TOOLS, "Agent", "Task", "Workflow"];
  return KEYWORD_MATCHERS[event] ?? [];
}

// Claude Code evalúa el matcher de dos formas distintas según qué
// caracteres tiene, y el cambio de modo es silencioso:
//   - Solo [A-Za-z0-9_-,| y espacio] -> coincidencia EXACTA (o lista con "|"/",")
//   - Cualquier otro caracter (., *, ^, $...) -> TODO el string pasa a ser
//     un regex de JavaScript sin anclar, probado con RegExp.test() — lo que
//     significa que "Edit" ahí adentro deja de ser exacto y empieza a
//     matchear como substring (ej. también dentro de "NotebookEdit").
// FileChanged y StopFailure tienen el set "plano" más angosto: ni guion,
// ni espacio, ni coma — cualquiera de esos ya los manda a modo regex.
const PLAIN_CHARS_GENERAL = /^[A-Za-z0-9_\- ,|]*$/;
const PLAIN_CHARS_NARROW = /^[A-Za-z0-9_|]*$/; // FileChanged, StopFailure

export function matcherTriggersRegexMode(tags: string[], event: string): boolean {
  const joined = tags.join("|");
  if (joined === "") return false;
  const re = event === "FileChanged" || event === "StopFailure" ? PLAIN_CHARS_NARROW : PLAIN_CHARS_GENERAL;
  return !re.test(joined);
}

export const FILE_GLOB_SUGGESTIONS = [
  "*.java",
  "*.ts",
  "*.tsx",
  "*.js",
  "*.jsx",
  "*.py",
  "*.go",
  "*.rs",
  "*.md",
  "*.json",
  "*.yaml",
  "*.yml",
];

export const LITERAL_FILE_SUGGESTIONS = [
  ".env",
  ".envrc",
  "package.json",
  "pom.xml",
  "requirements.txt",
  "go.mod",
];
