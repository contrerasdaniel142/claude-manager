export type Scope = "project" | "global";

export interface AgentSummary {
  id: string;
  name: string;
  description: string;
  scope: Scope;
  valid: boolean;
  errors: string[];
}

export interface AgentFile {
  id: string;
  scope: Scope;
  name: string;
  description: string;
  tools: string | null;
  model: string | null;
  disallowed_tools: string | null;
  permission_mode: string | null;
  color: string | null;
  max_turns: number | null;
  skills: string[] | null;
  memory: string | null;
  background: boolean | null;
  omit_claude_md: boolean | null;
  effort: string | null;
  isolation: string | null;
  initial_prompt: string | null;
  experimental: Record<string, unknown> | null;
  mcp_servers: unknown[] | null;
  hooks: Record<string, unknown> | null;
  body: string;
}

export interface SaveAgentInput {
  scope: Scope;
  originalId: string | null;
  name: string;
  description: string;
  tools: string | null;
  model: string | null;
  disallowedTools: string | null;
  permissionMode: string | null;
  color: string | null;
  maxTurns: number | null;
  skills: string[] | null;
  memory: string | null;
  background: boolean | null;
  omitClaudeMd: boolean | null;
  effort: string | null;
  isolation: string | null;
  initialPrompt: string | null;
  experimental: Record<string, unknown> | null;
  mcpServers: unknown[] | null;
  hooks: Record<string, unknown> | null;
  body: string;
}

export interface SkillSummary {
  id: string;
  name: string;
  description: string;
  scope: Scope;
  valid: boolean;
  errors: string[];
  extra_files: string[];
}

export interface SkillFile {
  id: string;
  scope: Scope;
  name: string;
  description: string;
  when_to_use: string | null;
  allowed_tools: string | null;
  paths: string[];
  argument_hint: string | null;
  arguments: string | null;
  disable_model_invocation: boolean | null;
  user_invocable: boolean | null;
  disallowed_tools: string | null;
  model: string | null;
  effort: string | null;
  context: string | null;
  agent: string | null;
  background: boolean | null;
  hooks: Record<string, unknown> | null;
  shell: string | null;
  metadata: Record<string, unknown> | null;
  license: string | null;
  compatibility: string | null;
  body: string;
  extra_files: string[];
}

export interface SaveSkillInput {
  scope: Scope;
  originalId: string | null;
  name: string;
  description: string;
  whenToUse: string | null;
  allowedTools: string | null;
  paths: string[];
  argumentHint: string | null;
  arguments: string | null;
  disableModelInvocation: boolean | null;
  userInvocable: boolean | null;
  disallowedTools: string | null;
  model: string | null;
  effort: string | null;
  context: string | null;
  agent: string | null;
  background: boolean | null;
  hooks: Record<string, unknown> | null;
  shell: string | null;
  metadata: Record<string, unknown> | null;
  license: string | null;
  compatibility: string | null;
  body: string;
}

export type HookAction = "command" | "message";
export type HookType = "command" | "http" | "mcp_tool" | "prompt" | "agent";

export interface HookRule {
  title: string;
  description: string;
  event: string;
  matcher: string;
  hook_type: HookType;
  action: HookAction;
  command: string;
  message: string;
  timeout: number | null;
  is_async: boolean;
  file_glob: string;
  if_filter: string | null;
  status_message: string | null;
  args: string[] | null;
  async_rewake: boolean | null;
  shell: string | null;
  on_failure: string | null;
  url: string | null;
  headers: string | null;
  allowed_env_vars: string[] | null;
  server: string | null;
  tool: string | null;
  input: string | null;
  prompt: string | null;
  prompt_model: string | null;
}

export interface HooksDocument {
  rules: HookRule[];
  errors: string[];
}

export interface SteeringSummary {
  rel_path: string;
  scope: Scope;
  title: string;
  size: number;
}

export interface SteeringFile {
  rel_path: string;
  scope: Scope;
  content: string;
}

export interface RuleSummary {
  id: string;
  scope: Scope;
  title: string;
  paths: string[];
  size: number;
}

export interface RuleFile {
  id: string;
  scope: Scope;
  paths: string[];
  title: string;
  body: string;
}

export interface OutputStyleSummary {
  id: string;
  scope: Scope;
  name: string;
  description: string;
}

export interface OutputStyleFile {
  id: string;
  scope: Scope;
  name: string;
  description: string;
  keep_coding_instructions: boolean;
  body: string;
}

export interface PermissionsSettings {
  allow: string[];
  ask: string[];
  deny: string[];
  additionalDirectories: string[];
  defaultMode: string | null;
  disableBypassPermissionsMode: boolean;
  disableAutoMode: boolean;
  blockReadsOutsideWorkingDirectories: boolean;
  attributionCommit: string | null;
  attributionPr: string | null;
  attributionSessionUrl: boolean | null;
  includeGitInstructions: boolean | null;
}

export interface McpServerSummary {
  name: string;
  scope: Scope;
  server_type: string;
}

export interface McpServerFile {
  name: string;
  scope: Scope;
  server_type: string;
  command: string | null;
  args: string[];
  env: string | null;
  url: string | null;
  headers: string | null;
}

export interface SearchResult {
  category: Category;
  id: string;
  title: string;
  snippet: string;
}

export interface RemoteSkill {
  rel_path: string;
  name: string;
  description: string;
  when_to_use: string;
  body: string;
  extra_files: string[];
}

export interface SaveMcpServerInput {
  scope: Scope;
  originalName: string | null;
  name: string;
  serverType: string;
  command: string | null;
  args: string[];
  env: string | null;
  url: string | null;
  headers: string | null;
}

export interface MarketplaceSummary {
  name: string;
  source: string;
  repo: string | null;
  url: string | null;
  path: string | null;
}

export interface PluginSummary {
  id: string;
  name: string;
  marketplace: string;
  version: string;
  scope: string;
  enabled: boolean;
  installed: boolean;
  description: string | null;
}

export interface PluginSearchPage {
  items: PluginSummary[];
  total: number;
}

export interface PluginInstallResult {
  status: "ok" | "needs_confirmation";
  message: string;
  command: string | null;
  kind: string | null;
  sha256: string | null;
}

export interface MarketplaceSkillSummary {
  source: string;
  skill_id: string;
  name: string;
  installs: number;
}

export interface MarketplaceSkillPreview {
  description: string;
  when_to_use: string;
  body: string;
  source_gone: boolean;
  security_audits: { name: string; verdict: string }[];
}

export type Category =
  | "advisor"
  | "steering"
  | "rules"
  | "hooks"
  | "agents"
  | "skills"
  | "outputStyles"
  | "permissions"
  | "mcpServers"
  | "plugins";

/** Every kind the advisor can classify a described need into — matches the
 * "advisor" schema's `candidates[].kind` enum in assistant.rs 1:1. */
export type AdvisorKind = "rule" | "hook" | "skill" | "outputStyle" | "permissions" | "mcpServer" | "plugin";

export interface AdvisorCandidate {
  kind: AdvisorKind;
  reason: string;
}
