import { useEffect, useState } from "react";
import { api } from "../api";
import type { AgentSummary, Scope, SkillSummary } from "../types";
import AiAssistBox from "./AiAssistBox";
import BoolSelect from "./BoolSelect";
import Button from "./Button";
import { useLanguage } from "./LanguageProvider";
import ListRow from "./ListRow";
import MarkdownEditor from "./MarkdownEditor";
import Select from "./Select";
import TagInput from "./TagInput";
import TextArea from "./TextArea";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";
import { BUILTIN_TOOLS } from "../toolSuggestions";

interface Props {
  scope: Scope;
  onListChanged: (list: AgentSummary[]) => void;
  refreshSignal: number;
  /** Set by Global Search to jump straight to an existing agent. */
  openId?: string;
  onOpenIdConsumed?: () => void;
}

const MODEL_OPTIONS = ["inherit", "sonnet", "opus", "haiku", "fable"];

const COLOR_HEX: Record<string, string> = {
  red: "#e5484d",
  blue: "#3b82f6",
  green: "#2fae5c",
  yellow: "#eab308",
  purple: "#a855f7",
  orange: "#f97316",
  pink: "#ec4899",
  cyan: "#06b6d4",
};

function parseList(raw: string | null, sep: string | RegExp = ","): string[] {
  if (!raw) return [];
  return raw
    .split(sep)
    .map((t) => t.trim())
    .filter(Boolean);
}

type TriBool = "" | "true" | "false";

const emptyForm = () => ({
  name: "",
  description: "",
  tools: [] as string[],
  model: "",
  disallowedTools: [] as string[],
  permissionMode: "",
  color: "",
  maxTurns: "" as string | number,
  skills: [] as string[],
  body: "",
  memory: "",
  background: "" as TriBool,
  omitClaudeMd: "" as TriBool,
  effort: "",
  isolation: "",
  initialPrompt: "",
  cacheTtl: "",
  mcpServersRaw: "",
  hooksRaw: "",
});

function triToBool(v: TriBool): boolean | null {
  return v === "" ? null : v === "true";
}

function boolToTri(v: boolean | null | undefined): TriBool {
  return v === true ? "true" : v === false ? "false" : "";
}

/** JSON.parse con mensaje de error legible — para los bloques "avanzado". */
function parseJsonField(raw: string, label: string): Record<string, unknown> | unknown[] | null {
  if (!raw.trim()) return null;
  try {
    return JSON.parse(raw);
  } catch (e) {
    throw new Error(`${label}: JSON inválido (${e instanceof Error ? e.message : e})`);
  }
}

export default function AgentsView({ scope, onListChanged, refreshSignal, openId, onOpenIdConsumed }: Props) {
  const [list, setList] = useState<AgentSummary[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [form, setForm] = useState(emptyForm());
  const [originalId, setOriginalId] = useState<string | null>(null);
  const toast = useToast();
  const { t } = useLanguage();
  const [permissionModes, setPermissionModes] = useState<string[]>([]);
  const [agentColors, setAgentColors] = useState<string[]>([]);
  const [availableSkills, setAvailableSkills] = useState<SkillSummary[]>([]);

  useEffect(() => {
    api.listPermissionModes().then(setPermissionModes);
    api.listAgentColors().then(setAgentColors);
  }, []);

  const refresh = async () => {
    const items = await api.listAgents(scope);
    setList(items);
    onListChanged(items);
    // Skills apply to an agent regardless of which scope the agent itself
    // lives in, so always offer both project and global ones to pick from.
    const [projectSkills, globalSkills] = await Promise.all([
      api.listSkills("project"),
      api.listSkills("global"),
    ]);
    setAvailableSkills([...projectSkills, ...globalSkills]);
  };

  useEffect(() => {
    refresh();
    setSelected(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  useEffect(() => {
    if (refreshSignal > 0) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshSignal]);

  const openNew = () => {
    setSelected("__new__");
    setOriginalId(null);
    setForm({ ...emptyForm(), body: "You are a...\n" });
  };

  const openExisting = async (id: string) => {
    const agent = await api.readAgent(scope, id);
    setSelected(id);
    setOriginalId(id);
    setForm({
      name: agent.name,
      description: agent.description,
      tools: parseList(agent.tools),
      model: agent.model ?? "",
      disallowedTools: parseList(agent.disallowed_tools),
      permissionMode: agent.permission_mode ?? "",
      color: agent.color ?? "",
      maxTurns: agent.max_turns ?? "",
      skills: agent.skills ?? [],
      body: agent.body,
      memory: agent.memory ?? "",
      background: boolToTri(agent.background),
      omitClaudeMd: boolToTri(agent.omit_claude_md),
      effort: agent.effort ?? "",
      isolation: agent.isolation ?? "",
      initialPrompt: agent.initial_prompt ?? "",
      cacheTtl: (agent.experimental?.cacheTtl as string) ?? "",
      mcpServersRaw: agent.mcp_servers ? JSON.stringify(agent.mcp_servers, null, 2) : "",
      hooksRaw: agent.hooks ? JSON.stringify(agent.hooks, null, 2) : "",
    });
  };

  useEffect(() => {
    if (!openId) return;
    openExisting(openId);
    onOpenIdConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [openId]);

  const save = async () => {
    try {
      const mcpServers = parseJsonField(form.mcpServersRaw, "MCP servers");
      const hooks = parseJsonField(form.hooksRaw, "Hooks del agente");
      await api.saveAgent({
        scope,
        originalId,
        name: form.name.trim(),
        description: form.description.trim(),
        tools: form.tools.length > 0 ? form.tools.join(", ") : null,
        model: form.model.trim() || null,
        disallowedTools: form.disallowedTools.length > 0 ? form.disallowedTools.join(", ") : null,
        permissionMode: form.permissionMode.trim() || null,
        color: form.color.trim() || null,
        maxTurns: form.maxTurns === "" ? null : Number(form.maxTurns),
        skills: form.skills.length > 0 ? form.skills : null,
        memory: form.memory.trim() || null,
        background: triToBool(form.background),
        omitClaudeMd: triToBool(form.omitClaudeMd),
        effort: form.effort.trim() || null,
        isolation: form.isolation.trim() || null,
        initialPrompt: form.initialPrompt.trim() || null,
        experimental: form.cacheTtl.trim() ? { cacheTtl: form.cacheTtl.trim() } : null,
        mcpServers: Array.isArray(mcpServers) ? mcpServers : mcpServers ? [mcpServers] : null,
        hooks: hooks as Record<string, unknown> | null,
        body: form.body,
      });
      toast.success(t.agents.savedToast);
      await refresh();
      setOriginalId(form.name.trim());
      setSelected(form.name.trim());
    } catch (e) {
      toast.error(String(e));
    }
  };

  const remove = async (id: string) => {
    if (!window.confirm(t.agents.deleteConfirm(id))) return;
    await api.deleteAgent(scope, id);
    setSelected(null);
    await refresh();
  };

  const moveToOtherScope = async () => {
    if (!originalId) return;
    const otherScope: Scope = scope === "project" ? "global" : "project";
    try {
      const mcpServers = parseJsonField(form.mcpServersRaw, "MCP servers");
      const hooks = parseJsonField(form.hooksRaw, "Hooks del agente");
      await api.saveAgent({
        scope: otherScope,
        originalId: null,
        name: form.name.trim(),
        description: form.description.trim(),
        tools: form.tools.length > 0 ? form.tools.join(", ") : null,
        model: form.model.trim() || null,
        disallowedTools: form.disallowedTools.length > 0 ? form.disallowedTools.join(", ") : null,
        permissionMode: form.permissionMode.trim() || null,
        color: form.color.trim() || null,
        maxTurns: form.maxTurns === "" ? null : Number(form.maxTurns),
        skills: form.skills.length > 0 ? form.skills : null,
        memory: form.memory.trim() || null,
        background: triToBool(form.background),
        omitClaudeMd: triToBool(form.omitClaudeMd),
        effort: form.effort.trim() || null,
        isolation: form.isolation.trim() || null,
        initialPrompt: form.initialPrompt.trim() || null,
        experimental: form.cacheTtl.trim() ? { cacheTtl: form.cacheTtl.trim() } : null,
        mcpServers: Array.isArray(mcpServers) ? mcpServers : mcpServers ? [mcpServers] : null,
        hooks: hooks as Record<string, unknown> | null,
        body: form.body,
      });
      await api.deleteAgent(scope, originalId);
      toast.success(
        t.common.movedToast(form.name || originalId, otherScope === "project" ? t.app.scopeProject : t.app.scopeGlobal),
      );
      setSelected(null);
      await refresh();
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="flex h-full">
      <div className="w-72 shrink-0 border-r overflow-y-auto" style={{ borderColor: "var(--border)" }}>
        <div className="p-3">
          <Button full onClick={openNew}>
            {t.agents.newAgent}
          </Button>
        </div>
        {list.map((a) => (
          <ListRow
            key={a.id}
            active={selected === a.id}
            onClick={() => openExisting(a.id)}
            title={
              <>
                {!a.valid && <span style={{ color: "var(--danger)" }}>{t.agents.invalid}</span>}
                {a.name || a.id}
              </>
            }
            subtitle={a.description || t.agents.noDescription}
          />
        ))}
        {list.length === 0 && (
          <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>
            {t.agents.emptyList}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-5">
        {!selected ? (
          <div className="max-w-2xl space-y-3" style={{ color: "var(--text-muted)" }}>
            <p>{t.agents.emptyMain}</p>
            <p>{t.agents.emptyMainPick}</p>
          </div>
        ) : (
          <div className="max-w-2xl space-y-4">
            <AiAssistBox
              kind="agent"
              currentFields={form}
              onResult={(data) =>
                setForm({
                  ...form,
                  name: typeof data.name === "string" ? data.name : form.name,
                  description: typeof data.description === "string" ? data.description : form.description,
                  tools: Array.isArray(data.tools) ? (data.tools as string[]) : form.tools,
                  model: typeof data.model === "string" ? data.model : form.model,
                  disallowedTools: Array.isArray(data.disallowedTools)
                    ? (data.disallowedTools as string[])
                    : form.disallowedTools,
                  permissionMode: typeof data.permissionMode === "string" ? data.permissionMode : form.permissionMode,
                  color: typeof data.color === "string" ? data.color : form.color,
                  // null es una respuesta válida del schema ("sin límite"), no solo number
                  maxTurns:
                    data.maxTurns === null ? "" : typeof data.maxTurns === "number" ? data.maxTurns : form.maxTurns,
                  skills: Array.isArray(data.skills) ? (data.skills as string[]) : form.skills,
                  body: typeof data.body === "string" ? data.body : form.body,
                  memory: typeof data.memory === "string" ? data.memory : form.memory,
                  background: typeof data.background === "boolean" ? boolToTri(data.background) : form.background,
                  omitClaudeMd:
                    typeof data.omitClaudeMd === "boolean" ? boolToTri(data.omitClaudeMd) : form.omitClaudeMd,
                  effort: typeof data.effort === "string" ? data.effort : form.effort,
                  isolation: typeof data.isolation === "string" ? data.isolation : form.isolation,
                  initialPrompt: typeof data.initialPrompt === "string" ? data.initialPrompt : form.initialPrompt,
                  cacheTtl: typeof data.cacheTtl === "string" ? data.cacheTtl : form.cacheTtl,
                  mcpServersRaw: typeof data.mcpServersRaw === "string" ? data.mcpServersRaw : form.mcpServersRaw,
                  hooksRaw: typeof data.hooksRaw === "string" ? data.hooksRaw : form.hooksRaw,
                })
              }
            />

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.agents.nameLabel} <span style={{ color: "var(--danger)" }}>*</span>
              </label>
              <TextInput
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder="backend-java"
                className="w-full mt-1"
              />
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.agents.nameHint(`${form.name || "…"}.md`)}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.agents.descriptionLabel} <span style={{ color: "var(--danger)" }}>*</span>
              </label>
              <TextArea
                value={form.description}
                onChange={(e) => setForm({ ...form, description: e.target.value })}
                rows={3}
                placeholder={t.agents.descriptionPlaceholder}
                className="w-full mt-1"
              />
            </div>

            <div className="flex gap-3">
              <div className="flex-1">
                <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                  {t.agents.toolsLabel}
                </label>
                <div className="mt-1">
                  <TagInput
                    tags={form.tools}
                    onChange={(tools) => setForm({ ...form, tools })}
                    suggestions={BUILTIN_TOOLS}
                    placeholder={t.common.pickOrTypeTool}
                  />
                </div>
                <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                  {t.agents.toolsHint("mcp__servidor__accion")}
                </div>
              </div>
              <div className="w-40">
                <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                  {t.agents.modelLabel}
                </label>
                <Select value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} className="w-full mt-1">
                  <option value="">{t.agents.default}</option>
                  {MODEL_OPTIONS.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </Select>
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.agents.disallowedToolsLabel}
              </label>
              <div className="mt-1">
                <TagInput
                  tags={form.disallowedTools}
                  onChange={(disallowedTools) => setForm({ ...form, disallowedTools })}
                  suggestions={BUILTIN_TOOLS}
                  placeholder={t.agents.disallowedToolsPlaceholder}
                />
              </div>
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.agents.disallowedToolsHint}
              </div>
            </div>

            <div className="flex gap-3">
              <div className="flex-1">
                <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                  {t.agents.permissionModeLabel}
                </label>
                <Select
                  value={form.permissionMode}
                  onChange={(e) => setForm({ ...form, permissionMode: e.target.value })}
                  className="w-full mt-1"
                >
                  <option value="">{t.agents.default}</option>
                  {permissionModes.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </Select>
              </div>
              <div className="w-28">
                <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                  {t.agents.maxTurnsLabel}
                </label>
                <TextInput
                  type="number"
                  min={1}
                  value={form.maxTurns}
                  onChange={(e) =>
                    setForm({ ...form, maxTurns: e.target.value === "" ? "" : Number(e.target.value) })
                  }
                  className="w-full mt-1"
                />
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.agents.colorLabel}
              </label>
              <div className="flex gap-1.5 mt-1.5">
                {agentColors.map((c) => (
                  <button
                    key={c}
                    onClick={() => setForm({ ...form, color: form.color === c ? "" : c })}
                    title={c}
                    className="w-6 h-6 rounded-full"
                    style={{
                      background: COLOR_HEX[c] ?? c,
                      outline: form.color === c ? "2px solid var(--text)" : "none",
                      outlineOffset: "2px",
                    }}
                  />
                ))}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.agents.skillsLabel}
              </label>
              {availableSkills.length === 0 ? (
                <div className="text-xs mt-1" style={{ color: "var(--text-muted)" }}>
                  {t.agents.skillsEmpty}
                </div>
              ) : (
                <div className="mt-1 space-y-2">
                  {(["project", "global"] as const).map((sc) => {
                    const items = availableSkills.filter((s) => s.scope === sc);
                    if (items.length === 0) return null;
                    return (
                      <div key={sc}>
                        <div className="text-[10px] uppercase tracking-wide mb-1" style={{ color: "var(--text-muted)" }}>
                          {sc === "project" ? t.app.scopeProject : t.app.scopeGlobal}
                        </div>
                        <div className="flex flex-wrap gap-1.5">
                          {items.map((s) => {
                            const active = form.skills.includes(s.id);
                            return (
                              <Button
                                key={`${sc}-${s.id}`}
                                type="button"
                                variant="chip"
                                size="sm"
                                active={active}
                                title={s.description || s.id}
                                onClick={() =>
                                  setForm({
                                    ...form,
                                    skills: active
                                      ? form.skills.filter((x) => x !== s.id)
                                      : [...form.skills, s.id],
                                  })
                                }
                              >
                                {s.id}
                              </Button>
                            );
                          })}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>

            <details className="rounded-lg border p-3" style={{ borderColor: "var(--border)" }}>
              <summary className="text-xs font-medium cursor-pointer" style={{ color: "var(--text-muted)" }}>
                {t.agents.advanced}
              </summary>
              <div className="mt-3 space-y-3">
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.agents.memoryLabel}
                    </label>
                    <Select value={form.memory} onChange={(e) => setForm({ ...form, memory: e.target.value })} className="w-full mt-1">
                      <option value="">{t.agents.unset}</option>
                      <option value="user">user</option>
                      <option value="project">project</option>
                      <option value="local">local</option>
                    </Select>
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.agents.effortLabel}
                    </label>
                    <Select value={form.effort} onChange={(e) => setForm({ ...form, effort: e.target.value })} className="w-full mt-1">
                      <option value="">{t.agents.unset}</option>
                      {["low", "medium", "high", "xhigh", "max"].map((v) => (
                        <option key={v} value={v}>
                          {v}
                        </option>
                      ))}
                    </Select>
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.agents.isolationLabel}
                    </label>
                    <Select value={form.isolation} onChange={(e) => setForm({ ...form, isolation: e.target.value })} className="w-full mt-1">
                      <option value="">{t.agents.unset}</option>
                      <option value="worktree">worktree</option>
                    </Select>
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.agents.cacheTtlLabel}
                    </label>
                    <Select value={form.cacheTtl} onChange={(e) => setForm({ ...form, cacheTtl: e.target.value })} className="w-full mt-1">
                      <option value="">{t.agents.unset}</option>
                      <option value="5m">5m</option>
                      <option value="1h">1h</option>
                    </Select>
                  </div>
                  <BoolSelect
                    label="background"
                    value={form.background}
                    onChange={(v) => setForm({ ...form, background: v })}
                    hint={t.agents.backgroundHint}
                  />
                  <BoolSelect
                    label="omitClaudeMd"
                    value={form.omitClaudeMd}
                    onChange={(v) => setForm({ ...form, omitClaudeMd: v })}
                    hint={t.agents.omitClaudeMdHint}
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.agents.initialPromptLabel}
                  </label>
                  <TextArea
                    value={form.initialPrompt}
                    onChange={(e) => setForm({ ...form, initialPrompt: e.target.value })}
                    rows={2}
                    placeholder={t.agents.initialPromptPlaceholder}
                    className="w-full mt-1"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.agents.mcpServersLabel}
                  </label>
                  <TextArea
                    value={form.mcpServersRaw}
                    onChange={(e) => setForm({ ...form, mcpServersRaw: e.target.value })}
                    rows={3}
                    placeholder='["github", {"playwright": {"type": "stdio", "command": "npx", "args": ["-y", "@playwright/mcp@latest"]}}]'
                    className="w-full mt-1"
                    small
                    mono
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.agents.hooksLabel}
                  </label>
                  <TextArea
                    value={form.hooksRaw}
                    onChange={(e) => setForm({ ...form, hooksRaw: e.target.value })}
                    rows={3}
                    placeholder='{"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "./check.sh"}]}]}'
                    className="w-full mt-1"
                    small
                    mono
                  />
                </div>
              </div>
            </details>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.agents.systemPromptLabel}
              </label>
              <div className="mt-1">
                <MarkdownEditor
                  value={form.body}
                  onChange={(v) => setForm({ ...form, body: v })}
                  minRows={14}
                />
              </div>
            </div>

            <div className="flex gap-2 pt-2">
              <Button onClick={save}>{t.common.save}</Button>
              {originalId && (
                <>
                  <Button variant="ghost" onClick={moveToOtherScope}>
                    {scope === "project" ? t.common.moveToGlobal : t.common.moveToProject}
                  </Button>
                  <Button variant="danger" onClick={() => remove(originalId)}>
                    {t.common.delete}
                  </Button>
                </>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
