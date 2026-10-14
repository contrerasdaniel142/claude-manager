import { useEffect, useState } from "react";
import { api } from "../api";
import type { Scope, SkillSummary } from "../types";
import AiAssistBox from "./AiAssistBox";
import BoolSelect from "./BoolSelect";
import Button from "./Button";
import ImportFromRepoBox from "./ImportFromRepoBox";
import { useLanguage } from "./LanguageProvider";
import ListRow from "./ListRow";
import MarkdownEditor from "./MarkdownEditor";
import Select from "./Select";
import SkillMarketplaceBox from "./SkillMarketplaceBox";
import TagInput from "./TagInput";
import TextArea from "./TextArea";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";
import { BUILTIN_TOOLS, FILE_GLOB_SUGGESTIONS } from "../toolSuggestions";

interface Props {
  scope: Scope;
  onListChanged: (list: SkillSummary[]) => void;
  refreshSignal: number;
  /** Set by the Advisor module when it sends the user here with fields it
   * already generated from a described need — applied as if this view's
   * own AiAssistBox had just returned that same result. */
  prefill?: Record<string, unknown>;
  onPrefillConsumed?: () => void;
  /** Set by Global Search to jump straight to an existing skill. */
  openId?: string;
  onOpenIdConsumed?: () => void;
}

type TriBool = "" | "true" | "false";

function triToBool(v: TriBool): boolean | null {
  return v === "" ? null : v === "true";
}

function boolToTri(v: boolean | null | undefined): TriBool {
  return v === true ? "true" : v === false ? "false" : "";
}

function parseJsonField(raw: string, label: string): Record<string, unknown> | null {
  if (!raw.trim()) return null;
  try {
    return JSON.parse(raw);
  } catch (e) {
    throw new Error(`${label}: JSON inválido (${e instanceof Error ? e.message : e})`);
  }
}

function parseTools(raw: string | null): string[] {
  if (!raw) return [];
  return raw.split(/\s+/).map((t) => t.trim()).filter(Boolean);
}

const emptySkillForm = () => ({
  name: "",
  description: "",
  whenToUse: "",
  allowedTools: [] as string[],
  body: "",
  paths: [] as string[],
  argumentHint: "",
  arguments: "",
  disableModelInvocation: "" as TriBool,
  userInvocable: "" as TriBool,
  disallowedTools: [] as string[],
  model: "",
  effort: "",
  context: "",
  agent: "",
  background: "" as TriBool,
  hooksRaw: "",
  shell: "",
  metadataRaw: "",
  license: "",
  compatibility: "",
});

type SkillForm = ReturnType<typeof emptySkillForm>;

function mergeAiResult(base: SkillForm, data: Record<string, unknown>): SkillForm {
  return {
    ...base,
    name: typeof data.name === "string" ? data.name : base.name,
    description: typeof data.description === "string" ? data.description : base.description,
    whenToUse: typeof data.whenToUse === "string" ? data.whenToUse : base.whenToUse,
    allowedTools: Array.isArray(data.allowedTools) ? (data.allowedTools as string[]) : base.allowedTools,
    body: typeof data.body === "string" ? data.body : base.body,
    paths: Array.isArray(data.paths) ? (data.paths as string[]) : base.paths,
    argumentHint: typeof data.argumentHint === "string" ? data.argumentHint : base.argumentHint,
    arguments: typeof data.arguments === "string" ? data.arguments : base.arguments,
    disableModelInvocation:
      typeof data.disableModelInvocation === "boolean"
        ? boolToTri(data.disableModelInvocation)
        : base.disableModelInvocation,
    userInvocable: typeof data.userInvocable === "boolean" ? boolToTri(data.userInvocable) : base.userInvocable,
    disallowedTools: Array.isArray(data.disallowedTools) ? (data.disallowedTools as string[]) : base.disallowedTools,
    model: typeof data.model === "string" ? data.model : base.model,
    effort: typeof data.effort === "string" ? data.effort : base.effort,
    context: typeof data.context === "string" ? data.context : base.context,
    agent: typeof data.agent === "string" ? data.agent : base.agent,
    background: typeof data.background === "boolean" ? boolToTri(data.background) : base.background,
    hooksRaw: typeof data.hooksRaw === "string" ? data.hooksRaw : base.hooksRaw,
    shell: typeof data.shell === "string" ? data.shell : base.shell,
    metadataRaw: typeof data.metadataRaw === "string" ? data.metadataRaw : base.metadataRaw,
    license: typeof data.license === "string" ? data.license : base.license,
    compatibility: typeof data.compatibility === "string" ? data.compatibility : base.compatibility,
  };
}

export default function SkillsView({
  scope,
  onListChanged,
  refreshSignal,
  prefill,
  onPrefillConsumed,
  openId,
  onOpenIdConsumed,
}: Props) {
  const [list, setList] = useState<SkillSummary[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [originalId, setOriginalId] = useState<string | null>(null);
  const [extraFiles, setExtraFiles] = useState<string[]>([]);
  const [form, setForm] = useState(emptySkillForm());
  const toast = useToast();
  const { t } = useLanguage();

  const refresh = async () => {
    const items = await api.listSkills(scope);
    setList(items);
    onListChanged(items);
  };

  useEffect(() => {
    refresh();
    setSelected(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  // Must run after the `[scope]` effect above — see RulesView for why.
  useEffect(() => {
    if (!prefill) return;
    setSelected("__new__");
    setOriginalId(null);
    setExtraFiles([]);
    setForm(mergeAiResult(emptySkillForm(), prefill));
    onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [prefill]);

  useEffect(() => {
    if (refreshSignal > 0) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshSignal]);

  const openNew = () => {
    setSelected("__new__");
    setOriginalId(null);
    setExtraFiles([]);
    setForm(emptySkillForm());
  };

  const openImport = () => {
    setSelected("__import__");
  };

  const openMarketplace = () => {
    setSelected("__marketplace__");
  };

  const openExisting = async (id: string) => {
    const skill = await api.readSkill(scope, id);
    setSelected(id);
    setOriginalId(id);
    setExtraFiles(skill.extra_files);
    setForm({
      name: skill.name,
      description: skill.description,
      whenToUse: skill.when_to_use ?? "",
      allowedTools: parseTools(skill.allowed_tools),
      body: skill.body,
      paths: skill.paths,
      argumentHint: skill.argument_hint ?? "",
      arguments: skill.arguments ?? "",
      disableModelInvocation: boolToTri(skill.disable_model_invocation),
      userInvocable: boolToTri(skill.user_invocable),
      disallowedTools: parseTools(skill.disallowed_tools),
      model: skill.model ?? "",
      effort: skill.effort ?? "",
      context: skill.context ?? "",
      agent: skill.agent ?? "",
      background: boolToTri(skill.background),
      hooksRaw: skill.hooks ? JSON.stringify(skill.hooks, null, 2) : "",
      shell: skill.shell ?? "",
      metadataRaw: skill.metadata ? JSON.stringify(skill.metadata, null, 2) : "",
      license: skill.license ?? "",
      compatibility: skill.compatibility ?? "",
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
      const hooks = parseJsonField(form.hooksRaw, "hooks");
      const metadata = parseJsonField(form.metadataRaw, "metadata");
      await api.saveSkill({
        scope,
        originalId,
        name: form.name.trim(),
        description: form.description.trim(),
        whenToUse: form.whenToUse.trim() || null,
        allowedTools: form.allowedTools.length > 0 ? form.allowedTools.join(" ") : null,
        paths: form.paths,
        argumentHint: form.argumentHint.trim() || null,
        arguments: form.arguments.trim() || null,
        disableModelInvocation: triToBool(form.disableModelInvocation),
        userInvocable: triToBool(form.userInvocable),
        disallowedTools: form.disallowedTools.length > 0 ? form.disallowedTools.join(" ") : null,
        model: form.model.trim() || null,
        effort: form.effort.trim() || null,
        context: form.context.trim() || null,
        agent: form.agent.trim() || null,
        background: triToBool(form.background),
        hooks,
        shell: form.shell.trim() || null,
        metadata,
        license: form.license.trim() || null,
        compatibility: form.compatibility.trim() || null,
        body: form.body,
      });
      toast.success(t.skills.savedToast);
      await refresh();
      const newId = form.name.trim() || originalId || "";
      setOriginalId(newId);
      setSelected(newId);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const remove = async (id: string) => {
    if (!window.confirm(t.skills.deleteConfirm(id))) return;
    await api.deleteSkill(scope, id);
    setSelected(null);
    await refresh();
  };

  const moveToOtherScope = async () => {
    if (!originalId) return;
    const otherScope: Scope = scope === "project" ? "global" : "project";
    try {
      await api.moveSkill(scope, otherScope, originalId);
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
        <div className="p-3 space-y-2">
          <Button full onClick={openNew}>
            {t.skills.newSkill}
          </Button>
          <Button full onClick={openImport}>
            {t.skills.installFromRepo}
          </Button>
          <Button full onClick={openMarketplace}>
            {t.skills.searchMarketplace}
          </Button>
        </div>
        {list.map((s) => (
          <ListRow
            key={s.id}
            active={selected === s.id}
            onClick={() => openExisting(s.id)}
            title={
              <>
                {!s.valid && <span style={{ color: "var(--danger)" }}>{t.skills.invalid}</span>}
                {s.name || s.id}
              </>
            }
            subtitle={s.description || t.skills.noDescription}
          />
        ))}
        {list.length === 0 && (
          <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>
            {t.skills.emptyList}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-5">
        {selected === "__import__" ? (
          <ImportFromRepoBox
            strings={t.importSkill}
            localNames={list.map((s) => s.name)}
            search={(repoUrl) => api.listRemoteSkills(repoUrl)}
            install={(repoUrl, item) => api.importSkill(repoUrl, item.rel_path, scope)}
            getKey={(item) => item.rel_path}
            getName={(item) => item.name}
            getTitle={(item) => item.name}
            getSubtitle={(item) => item.description || item.rel_path}
            onImported={refresh}
          />
        ) : selected === "__marketplace__" ? (
          <SkillMarketplaceBox scope={scope} localNames={list.map((s) => s.name)} onInstalled={refresh} />
        ) : !selected ? (
          <div className="max-w-2xl space-y-3" style={{ color: "var(--text-muted)" }}>
            <p>{t.skills.emptyMain}</p>
            <p>{t.skills.emptyMainPick}</p>
          </div>
        ) : (
          <div className="max-w-2xl space-y-4">

            <AiAssistBox
              kind="skill"
              currentFields={form}
              onResult={(data) => setForm((prev) => mergeAiResult(prev, data))}
            />

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.skills.nameLabel}
              </label>
              <TextInput
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder="api-design"
                className="w-full mt-1"
              />
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.skills.nameHint(`skills/${form.name || originalId || "…"}/SKILL.md`)}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.skills.descriptionLabel}
              </label>
              <TextArea
                value={form.description}
                onChange={(e) => setForm({ ...form, description: e.target.value })}
                rows={2}
                className="w-full mt-1"
              />
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.skills.whenToUseLabel}
              </label>
              <TextArea
                value={form.whenToUse}
                onChange={(e) => setForm({ ...form, whenToUse: e.target.value })}
                rows={2}
                placeholder={t.skills.whenToUsePlaceholder}
                className="w-full mt-1"
              />
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.skills.allowedToolsLabel}
              </label>
              <div className="mt-1">
                <TagInput
                  tags={form.allowedTools}
                  onChange={(allowedTools) => setForm({ ...form, allowedTools })}
                  suggestions={BUILTIN_TOOLS}
                  placeholder={t.common.pickOrTypeTool}
                />
              </div>
            </div>

            <details className="rounded-lg border p-3" style={{ borderColor: "var(--border)" }}>
              <summary className="text-xs font-medium cursor-pointer" style={{ color: "var(--text-muted)" }}>
                {t.skills.advanced}
              </summary>
              <div className="mt-3 space-y-3">
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.skills.pathsLabel}
                  </label>
                  <div className="mt-1">
                    <TagInput
                      tags={form.paths}
                      onChange={(paths) => setForm({ ...form, paths })}
                      suggestions={FILE_GLOB_SUGGESTIONS}
                      placeholder={`${t.common.eg}: src/**/*.ts`}
                    />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.skills.argumentHintLabel}
                    </label>
                    <TextInput
                      value={form.argumentHint}
                      onChange={(e) => setForm({ ...form, argumentHint: e.target.value })}
                      placeholder="[issue-number]"
                      className="w-full mt-1"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.skills.argumentsLabel}
                    </label>
                    <TextInput
                      value={form.arguments}
                      onChange={(e) => setForm({ ...form, arguments: e.target.value })}
                      placeholder={`${t.common.eg}: issue-number`}
                      className="w-full mt-1"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      model
                    </label>
                    <Select value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} className="w-full mt-1">
                      <option value="">{t.skills.unset}</option>
                      {["inherit", "sonnet", "opus", "haiku", "fable"].map((v) => (
                        <option key={v} value={v}>
                          {v}
                        </option>
                      ))}
                    </Select>
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      effort
                    </label>
                    <Select value={form.effort} onChange={(e) => setForm({ ...form, effort: e.target.value })} className="w-full mt-1">
                      <option value="">{t.skills.unset}</option>
                      {["low", "medium", "high", "xhigh", "max"].map((v) => (
                        <option key={v} value={v}>
                          {v}
                        </option>
                      ))}
                    </Select>
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      shell
                    </label>
                    <Select value={form.shell} onChange={(e) => setForm({ ...form, shell: e.target.value })} className="w-full mt-1">
                      <option value="">{t.skills.shellUnset}</option>
                      <option value="bash">bash</option>
                      <option value="powershell">powershell</option>
                    </Select>
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.skills.contextLabel}
                    </label>
                    <Select value={form.context} onChange={(e) => setForm({ ...form, context: e.target.value })} className="w-full mt-1">
                      <option value="">{t.skills.unset}</option>
                      <option value="fork">{t.skills.contextFork}</option>
                    </Select>
                  </div>
                  {form.context === "fork" && (
                    <>
                      <div>
                        <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                          {t.skills.agentLabel}
                        </label>
                        <TextInput
                          value={form.agent}
                          onChange={(e) => setForm({ ...form, agent: e.target.value })}
                          placeholder={t.skills.agentPlaceholder}
                          className="w-full mt-1"
                        />
                      </div>
                      <BoolSelect
                        label="background"
                        value={form.background}
                        onChange={(v) => setForm({ ...form, background: v })}
                        hint={t.skills.backgroundHint}
                      />
                    </>
                  )}
                  <BoolSelect
                    label="disable-model-invocation"
                    value={form.disableModelInvocation}
                    onChange={(v) => setForm({ ...form, disableModelInvocation: v })}
                    hint={t.skills.disableModelInvocationHint}
                  />
                  <BoolSelect
                    label="user-invocable"
                    value={form.userInvocable}
                    onChange={(v) => setForm({ ...form, userInvocable: v })}
                    hint={t.skills.userInvocableHint}
                  />
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.skills.licenseLabel}
                    </label>
                    <TextInput
                      value={form.license}
                      onChange={(e) => setForm({ ...form, license: e.target.value })}
                      placeholder={`${t.common.eg}: MIT`}
                      className="w-full mt-1"
                    />
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.skills.disallowedToolsLabel}
                  </label>
                  <div className="mt-1">
                    <TagInput
                      tags={form.disallowedTools}
                      onChange={(disallowedTools) => setForm({ ...form, disallowedTools })}
                      suggestions={BUILTIN_TOOLS}
                      placeholder={t.common.pickOrTypeTool}
                    />
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.skills.compatibilityLabel}
                  </label>
                  <TextArea
                    value={form.compatibility}
                    onChange={(e) => setForm({ ...form, compatibility: e.target.value })}
                    rows={2}
                    placeholder={t.skills.compatibilityPlaceholder}
                    className="w-full mt-1"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.skills.hooksLabel}
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
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.skills.metadataLabel}
                  </label>
                  <TextArea
                    value={form.metadataRaw}
                    onChange={(e) => setForm({ ...form, metadataRaw: e.target.value })}
                    rows={2}
                    placeholder='{"team": "platform"}'
                    className="w-full mt-1"
                    small
                    mono
                  />
                </div>
              </div>
            </details>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.skills.contentLabel}
              </label>
              <div className="mt-1">
                <MarkdownEditor
                  value={form.body}
                  onChange={(v) => setForm({ ...form, body: v })}
                  minRows={14}
                />
              </div>
            </div>

            {extraFiles.length > 0 && (
              <div className="text-xs" style={{ color: "var(--text-muted)" }}>
                {t.skills.extraFiles(extraFiles.join(", "))}
              </div>
            )}

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
