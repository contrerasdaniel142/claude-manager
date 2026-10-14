import { invoke } from "@tauri-apps/api/core";
import type {
  AgentFile,
  AgentSummary,
  HookRule,
  HooksDocument,
  MarketplaceSummary,
  MarketplaceSkillPreview,
  MarketplaceSkillSummary,
  McpServerFile,
  McpServerSummary,
  OutputStyleFile,
  OutputStyleSummary,
  PermissionsSettings,
  PluginSearchPage,
  PluginSummary,
  RemoteMcpServer,
  RemoteSkill,
  SaveMcpServerInput,
  RuleFile,
  RuleSummary,
  SaveAgentInput,
  SaveSkillInput,
  Scope,
  SearchResult,
  SkillFile,
  SkillSummary,
  SteeringFile,
  SteeringSummary,
} from "./types";

export const api = {
  getRoots: () => invoke<{ project: string; global: string }>("get_roots"),
  setLanguage: (lang: "en" | "es") => invoke<void>("set_language", { lang }),
  setProjectRoot: (path: string) => invoke<{ project: string; global: string }>("set_project_root", { path }),

  listAgents: (scope: Scope) => invoke<AgentSummary[]>("list_agents", { scope }),
  readAgent: (scope: Scope, id: string) => invoke<AgentFile>("read_agent", { scope, id }),
  saveAgent: (input: SaveAgentInput) => invoke<AgentSummary>("save_agent", { input }),
  deleteAgent: (scope: Scope, id: string) => invoke<void>("delete_agent", { scope, id }),
  listPermissionModes: () => invoke<string[]>("list_permission_modes"),
  listAgentColors: () => invoke<string[]>("list_agent_colors"),

  listSkills: (scope: Scope) => invoke<SkillSummary[]>("list_skills", { scope }),
  readSkill: (scope: Scope, id: string) => invoke<SkillFile>("read_skill", { scope, id }),
  saveSkill: (input: SaveSkillInput) => invoke<SkillSummary>("save_skill", { input }),
  deleteSkill: (scope: Scope, id: string) => invoke<void>("delete_skill", { scope, id }),
  moveSkill: (fromScope: Scope, toScope: Scope, id: string) =>
    invoke<SkillSummary>("move_skill", { fromScope, toScope, id }),
  listRemoteSkills: (repoUrl: string) => invoke<RemoteSkill[]>("list_remote_skills", { repoUrl }),
  importSkill: (repoUrl: string, relPath: string, scope: Scope) =>
    invoke<string>("import_skill", { repoUrl, relPath, scope }),
  searchMarketplaceSkills: (query: string) =>
    invoke<MarketplaceSkillSummary[]>("search_marketplace_skills", { query }),
  previewMarketplaceSkill: (source: string, skillId: string) =>
    invoke<MarketplaceSkillPreview>("preview_marketplace_skill", { source, skillId }),
  installMarketplaceSkill: (source: string, skillId: string, scope: Scope) =>
    invoke<string>("install_marketplace_skill", { source, skillId, scope }),

  listHookEvents: () => invoke<string[]>("list_hook_events"),
  readHooks: (scope: Scope) => invoke<HooksDocument>("read_hooks", { scope }),
  saveHooks: (scope: Scope, rules: HookRule[]) =>
    invoke<HooksDocument>("save_hooks", { scope, rules }),

  listSteering: (scope: Scope) => invoke<SteeringSummary[]>("list_steering", { scope }),
  readSteering: (scope: Scope, relPath: string) =>
    invoke<SteeringFile>("read_steering", { scope, relPath }),
  saveSteering: (scope: Scope, relPath: string, content: string) =>
    invoke<SteeringFile>("save_steering", { scope, relPath, content }),
  deleteSteering: (scope: Scope, relPath: string) =>
    invoke<void>("delete_steering", { scope, relPath }),

  assistantAvailable: () => invoke<boolean>("assistant_available"),
  listAssistantModels: () => invoke<string[]>("list_assistant_models"),
  generateWithAI: (
    kind: "rule" | "hook" | "agent" | "skill" | "claudemd" | "outputStyle" | "permissions" | "mcpServer" | "advisor",
    prompt: string,
    currentFields: unknown,
    model: string,
  ) => invoke<Record<string, unknown>>("generate_with_ai", { kind, prompt, currentFields, model }),

  listRules: (scope: Scope) => invoke<RuleSummary[]>("list_rules", { scope }),
  searchEverything: (scope: Scope, query: string) =>
    invoke<SearchResult[]>("search_everything", { scope, query }),
  readRule: (scope: Scope, id: string) => invoke<RuleFile>("read_rule", { scope, id }),
  saveRule: (scope: Scope, originalId: string | null, id: string, paths: string[], body: string) =>
    invoke<RuleSummary>("save_rule", { scope, originalId, id, paths, body }),
  deleteRule: (scope: Scope, id: string) => invoke<void>("delete_rule", { scope, id }),

  listOutputStyles: (scope: Scope) => invoke<OutputStyleSummary[]>("list_output_styles", { scope }),
  readOutputStyle: (scope: Scope, id: string) =>
    invoke<OutputStyleFile>("read_output_style", { scope, id }),
  saveOutputStyle: (
    scope: Scope,
    originalId: string | null,
    id: string,
    name: string | null,
    description: string | null,
    keepCodingInstructions: boolean,
    body: string,
  ) =>
    invoke<OutputStyleSummary>("save_output_style", {
      scope,
      originalId,
      id,
      name,
      description,
      keepCodingInstructions,
      body,
    }),
  deleteOutputStyle: (scope: Scope, id: string) => invoke<void>("delete_output_style", { scope, id }),

  readPermissions: (scope: Scope) => invoke<PermissionsSettings>("read_permissions", { scope }),
  savePermissions: (scope: Scope, input: PermissionsSettings) =>
    invoke<PermissionsSettings>("save_permissions", { scope, input }),

  listMcpServers: (scope: Scope) => invoke<McpServerSummary[]>("list_mcp_servers", { scope }),
  readMcpServer: (scope: Scope, name: string) => invoke<McpServerFile>("read_mcp_server", { scope, name }),
  saveMcpServer: (input: SaveMcpServerInput) => invoke<McpServerSummary>("save_mcp_server", { input }),
  deleteMcpServer: (scope: Scope, name: string) => invoke<void>("delete_mcp_server", { scope, name }),
  listRemoteMcpServers: (repoUrl: string) =>
    invoke<RemoteMcpServer[]>("list_remote_mcp_servers", { repoUrl }),
  importMcpServer: (repoUrl: string, relPath: string, name: string, scope: Scope) =>
    invoke<McpServerSummary>("import_mcp_server", { repoUrl, relPath, name, scope }),

  listMarketplaces: () => invoke<MarketplaceSummary[]>("list_marketplaces"),
  addMarketplace: (source: string, scope: string) => invoke<string>("add_marketplace", { source, scope }),
  removeMarketplace: (name: string) => invoke<string>("remove_marketplace", { name }),
  listInstalledPlugins: () => invoke<PluginSummary[]>("list_installed_plugins"),
  searchAvailablePlugins: (marketplace: string, query: string, offset: number, limit: number) =>
    invoke<PluginSearchPage>("search_available_plugins", { marketplace, query, offset, limit }),
  installPlugin: (name: string, marketplace: string, scope: string) =>
    invoke<string>("install_plugin", { name, marketplace, scope }),
  uninstallPlugin: (name: string, marketplace: string, scope: string) =>
    invoke<string>("uninstall_plugin", { name, marketplace, scope }),
  enablePlugin: (name: string, marketplace: string, scope: string) =>
    invoke<string>("enable_plugin", { name, marketplace, scope }),
  disablePlugin: (name: string, marketplace: string, scope: string) =>
    invoke<string>("disable_plugin", { name, marketplace, scope }),
};
