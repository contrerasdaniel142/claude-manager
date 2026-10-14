import { useEffect, useState } from "react";
import { api } from "../api";
import type { HookAction, HookRule, HookType, Scope } from "../types";
import AiAssistBox from "./AiAssistBox";
import Banner from "./Banner";
import Button from "./Button";
import Checkbox from "./Checkbox";
import { useLanguage } from "./LanguageProvider";
import ListRow from "./ListRow";
import Select from "./Select";
import TagInput from "./TagInput";
import TextArea from "./TextArea";
import TextInput from "./TextInput";
import { useToast } from "./ToastProvider";
import {
  EVENT_GROUPS,
  FILE_GLOB_SUGGESTIONS,
  LITERAL_FILE_SUGGESTIONS,
  matcherIsClosedSet,
  matcherKind,
  matcherSuggestions,
  matcherTriggersRegexMode,
} from "../toolSuggestions";

interface Props {
  scope: Scope;
  onCountChanged: (count: number) => void;
  onLoadErrorsChanged?: (errors: string[]) => void;
  refreshSignal: number;
  /** Set by the Advisor module when it sends the user here with fields it
   * already generated from a described need — applied as if this view's
   * own AiAssistBox had just returned that same result. */
  prefill?: Record<string, unknown>;
  onPrefillConsumed?: () => void;
}

function splitPipe(s: string): string[] {
  return s.split("|").map((t) => t.trim()).filter(Boolean);
}

type TriBool = "" | "true" | "false";

function triToBool(v: TriBool): boolean | null {
  return v === "" ? null : v === "true";
}

function boolToTri(v: boolean | null | undefined): TriBool {
  return v === true ? "true" : v === false ? "false" : "";
}

const emptyForm = (defaultEvent: string) => ({
  title: "",
  description: "",
  event: defaultEvent,
  matcher: [] as string[],
  hookType: "command" as HookType,
  action: "command" as HookAction,
  command: "",
  message: "",
  timeout: "" as string | number,
  isAsync: false,
  fileGlob: [] as string[],
  ifFilter: "",
  statusMessage: "",
  args: [] as string[],
  asyncRewake: "" as TriBool,
  shell: "",
  onFailure: "",
  url: "",
  headers: "",
  allowedEnvVars: [] as string[],
  server: "",
  tool: "",
  input: "",
  prompt: "",
  promptModel: "",
});


type HookForm = ReturnType<typeof emptyForm>;

function mergeAiResult(base: HookForm, data: Record<string, unknown>): HookForm {
  return {
    ...base,
    title: typeof data.title === "string" ? data.title : base.title,
    description: typeof data.description === "string" ? data.description : base.description,
    event: typeof data.event === "string" && data.event ? data.event : base.event,
    matcher: Array.isArray(data.matcher) ? (data.matcher as string[]) : base.matcher,
    hookType: typeof data.hookType === "string" ? (data.hookType as HookType) : base.hookType,
    action: data.action === "command" || data.action === "message" ? data.action : base.action,
    command: typeof data.command === "string" ? data.command : base.command,
    message: typeof data.message === "string" ? data.message : base.message,
    timeout: data.timeout === null ? "" : typeof data.timeout === "number" ? data.timeout : base.timeout,
    isAsync: typeof data.isAsync === "boolean" ? data.isAsync : base.isAsync,
    fileGlob: Array.isArray(data.fileGlob) ? (data.fileGlob as string[]) : base.fileGlob,
    ifFilter: typeof data.ifFilter === "string" ? data.ifFilter : base.ifFilter,
    statusMessage: typeof data.statusMessage === "string" ? data.statusMessage : base.statusMessage,
    args: Array.isArray(data.args) ? (data.args as string[]) : base.args,
    asyncRewake: typeof data.asyncRewake === "boolean" ? boolToTri(data.asyncRewake) : base.asyncRewake,
    shell: typeof data.shell === "string" ? data.shell : base.shell,
    onFailure: typeof data.onFailure === "string" ? data.onFailure : base.onFailure,
    url: typeof data.url === "string" ? data.url : base.url,
    headers: typeof data.headers === "string" ? data.headers : base.headers,
    allowedEnvVars: Array.isArray(data.allowedEnvVars) ? (data.allowedEnvVars as string[]) : base.allowedEnvVars,
    server: typeof data.server === "string" ? data.server : base.server,
    tool: typeof data.tool === "string" ? data.tool : base.tool,
    input: typeof data.input === "string" ? data.input : base.input,
    prompt: typeof data.prompt === "string" ? data.prompt : base.prompt,
    promptModel: typeof data.promptModel === "string" ? data.promptModel : base.promptModel,
  };
}

export default function HooksView({
  scope,
  onCountChanged,
  onLoadErrorsChanged,
  refreshSignal,
  prefill,
  onPrefillConsumed,
}: Props) {
  const [rules, setRules] = useState<HookRule[]>([]);
  const [loadErrors, setLoadErrors] = useState<string[]>([]);
  const [editIndex, setEditIndex] = useState<number | "new" | null>(null);
  const [form, setForm] = useState(emptyForm("PostToolUse"));
  const toast = useToast();
  const { t } = useLanguage();

  const refresh = async () => {
    const doc = await api.readHooks(scope);
    setRules(doc.rules);
    setLoadErrors(doc.errors);
    onCountChanged(doc.rules.length);
    onLoadErrorsChanged?.(doc.errors);
  };

  useEffect(() => {
    refresh();
    setEditIndex(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  // Must run after the `[scope]` effect above — both fire on mount, in
  // declaration order, and that one unconditionally resets `editIndex`,
  // which would wipe this one's "new" if it ran first.
  useEffect(() => {
    if (!prefill) return;
    setEditIndex("new");
    setForm(mergeAiResult(emptyForm("PostToolUse"), prefill));
    onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [prefill]);

  useEffect(() => {
    if (refreshSignal > 0) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshSignal]);

  const openNew = () => {
    setEditIndex("new");
    setForm(emptyForm("PostToolUse"));
  };

  const openExisting = (idx: number) => {
    const r = rules[idx];
    setEditIndex(idx);
    setForm({
      title: r.title,
      description: r.description,
      event: r.event,
      matcher: splitPipe(r.matcher),
      hookType: r.hook_type,
      action: r.action,
      command: r.command,
      message: r.message,
      timeout: r.timeout ?? "",
      isAsync: r.is_async,
      fileGlob: splitPipe(r.file_glob),
      ifFilter: r.if_filter ?? "",
      statusMessage: r.status_message ?? "",
      args: r.args ?? [],
      asyncRewake: boolToTri(r.async_rewake),
      shell: r.shell ?? "",
      onFailure: r.on_failure ?? "",
      url: r.url ?? "",
      headers: r.headers ?? "",
      allowedEnvVars: r.allowed_env_vars ?? [],
      server: r.server ?? "",
      tool: r.tool ?? "",
      input: r.input ?? "",
      prompt: r.prompt ?? "",
      promptModel: r.prompt_model ?? "",
    });
  };

  const persist = async (next: HookRule[]) => {
    const doc = await api.saveHooks(scope, next);
    setRules(doc.rules);
    onCountChanged(doc.rules.length);
    return doc;
  };

  const buildRule = (): HookRule => ({
    title: form.title.trim(),
    description: form.description.trim(),
    event: form.event,
    matcher: form.matcher.join("|"),
    hook_type: form.hookType,
    action: form.action,
    command: form.command.trim(),
    message: form.message.trim(),
    timeout: form.timeout === "" ? null : Number(form.timeout),
    is_async: form.isAsync,
    file_glob: form.fileGlob.join("|"),
    if_filter: form.ifFilter.trim() || null,
    status_message: form.statusMessage.trim() || null,
    args: form.args.length > 0 ? form.args : null,
    async_rewake: triToBool(form.asyncRewake),
    shell: form.shell.trim() || null,
    on_failure: form.onFailure.trim() || null,
    url: form.url.trim() || null,
    headers: form.headers.trim() || null,
    allowed_env_vars: form.allowedEnvVars.length > 0 ? form.allowedEnvVars : null,
    server: form.server.trim() || null,
    tool: form.tool.trim() || null,
    input: form.input.trim() || null,
    prompt: form.prompt.trim() || null,
    prompt_model: form.promptModel.trim() || null,
  });

  const save = async () => {
    const rule = buildRule();
    const next = [...rules];
    if (editIndex === "new") {
      next.push(rule);
    } else if (typeof editIndex === "number") {
      next[editIndex] = rule;
    }
    try {
      await persist(next);
      toast.success(t.hooks.savedToast);
      setEditIndex(null);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const remove = async (idx: number) => {
    if (!window.confirm(t.hooks.deleteConfirm)) return;
    const next = rules.filter((_, i) => i !== idx);
    try {
      await persist(next);
      setEditIndex(null);
    } catch (e) {
      toast.error(String(e));
    }
  };

  // Hooks aren't individually addressable — the whole list is saved per
  // scope at once — so moving means: append this rule to the other scope's
  // current list, then save this scope's list without it.
  const moveToOtherScope = async () => {
    if (typeof editIndex !== "number") return;
    const otherScope: Scope = scope === "project" ? "global" : "project";
    try {
      const rule = buildRule();
      const otherDoc = await api.readHooks(otherScope);
      await api.saveHooks(otherScope, [...otherDoc.rules, rule]);
      await persist(rules.filter((_, i) => i !== editIndex));
      toast.success(
        t.common.movedToast(
          rule.title || rule.event,
          otherScope === "project" ? t.app.scopeProject : t.app.scopeGlobal,
        ),
      );
      setEditIndex(null);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const mKind = matcherKind(form.event);

  return (
    <div className="flex h-full">
      <div className="w-80 shrink-0 border-r overflow-y-auto" style={{ borderColor: "var(--border)" }}>
        <div className="p-3">
          <Button full onClick={openNew}>
            {t.hooks.newHook}
          </Button>
        </div>
        {rules.map((r, idx) => (
          <ListRow
            key={idx}
            active={editIndex === idx}
            onClick={() => openExisting(idx)}
            title={r.title || `${r.event}${r.matcher ? " · " + r.matcher : ""}`}
            subtitle={
              r.description ||
              (r.action === "message" ? r.message : r.command) ||
              `${r.event}${r.file_glob ? ` [${r.file_glob}]` : ""}`
            }
          />
        ))}
        {rules.length === 0 && (
          <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>
            {t.hooks.emptyList}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-5">
        {loadErrors.length > 0 && (
          <Banner kind="error">
            {t.hooks.settingsJsonErrors}
            {"\n"}
            {loadErrors.join("\n")}
          </Banner>
        )}

        {editIndex === null ? (
          <div className="max-w-2xl space-y-3" style={{ color: "var(--text-muted)" }}>
            <p>{t.hooks.emptyMain}</p>
            <p>{t.hooks.emptyMainPick}</p>
          </div>
        ) : (
          <div className="max-w-xl space-y-4">
            <AiAssistBox
              kind="hook"
              currentFields={form}
              onResult={(data) => setForm((prev) => mergeAiResult(prev, data))}
            />

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.hooks.titleLabel}
              </label>
              <TextInput
                value={form.title}
                onChange={(e) => setForm({ ...form, title: e.target.value })}
                placeholder={t.hooks.titlePlaceholder}
                className="w-full mt-1"
              />
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.hooks.descriptionLabel}
              </label>
              <TextInput
                value={form.description}
                onChange={(e) => setForm({ ...form, description: e.target.value })}
                className="w-full mt-1"
              />
            </div>

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.hooks.triggerLabel} <span style={{ color: "var(--danger)" }}>*</span>
              </label>
              <Select
                value={form.event}
                onChange={(e) => setForm({ ...form, event: e.target.value, matcher: [] })}
                className="w-full mt-1"
              >
                {EVENT_GROUPS.map((g) => (
                  <optgroup key={g.key} label={t.hooks.eventGroupLabels[g.key]}>
                    {g.events.map((ev) => (
                      <option key={ev} value={ev}>
                        {ev}
                      </option>
                    ))}
                  </optgroup>
                ))}
              </Select>
              <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                {t.hooks.eventDescriptions[form.event] ?? ""}
              </div>
            </div>

            {mKind !== "none" && (
              <div>
                <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                  {t.hooks.matcherLabel}
                </label>
                {matcherIsClosedSet(form.event) ? (
                  <>
                    <div className="flex flex-wrap gap-1.5 mt-1">
                      {matcherSuggestions(form.event).map((v) => {
                        const active = form.matcher.includes(v);
                        return (
                          <Button
                            key={v}
                            type="button"
                            variant="chip"
                            size="sm"
                            active={active}
                            onClick={() =>
                              setForm({
                                ...form,
                                matcher: active
                                  ? form.matcher.filter((x) => x !== v)
                                  : [...form.matcher, v],
                              })
                            }
                          >
                            {v}
                          </Button>
                        );
                      })}
                    </div>
                    <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                      {t.hooks.matcherClosedSetHint}
                    </div>
                  </>
                ) : (
                  <>
                    <div className="mt-1">
                      <TagInput
                        tags={form.matcher}
                        onChange={(matcher) => setForm({ ...form, matcher })}
                        suggestions={mKind === "literal-file" ? LITERAL_FILE_SUGGESTIONS : matcherSuggestions(form.event)}
                        placeholder={
                          mKind === "tool"
                            ? t.hooks.matcherToolPlaceholder
                            : mKind === "literal-file"
                              ? t.hooks.matcherLiteralFilePlaceholder
                              : t.hooks.matcherKeywordPlaceholder
                        }
                      />
                    </div>
                    <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                      {mKind === "tool" && t.hooks.matcherToolHint}
                      {mKind === "literal-file" && t.hooks.matcherLiteralFileHint}
                      {mKind === "keyword" && t.hooks.matcherKeywordHint}
                    </div>
                  </>
                )}
                {matcherTriggersRegexMode(form.matcher, form.event) && (
                  <div
                    className="text-[11px] mt-1.5 px-2 py-1.5 rounded"
                    style={{ background: "var(--accent-soft)", color: "var(--accent)" }}
                  >
                    {t.hooks.regexWarning("^Edit$")}
                  </div>
                )}
              </div>
            )}

            {mKind === "tool" && (
              <div>
                <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                  {t.hooks.fileGlobLabel}
                </label>
                <div className="mt-1">
                  <TagInput
                    tags={form.fileGlob}
                    onChange={(fileGlob) => setForm({ ...form, fileGlob })}
                    suggestions={FILE_GLOB_SUGGESTIONS}
                    placeholder={`${t.common.eg}: *.java, *.tsx`}
                  />
                </div>
                <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                  {t.hooks.fileGlobHint(".claude/hooks/_clman/")}
                </div>
              </div>
            )}

            <div>
              <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                {t.hooks.hookTypeLabel}
              </label>
              <Select
                value={form.hookType}
                onChange={(e) => setForm({ ...form, hookType: e.target.value as HookType })}
                className="w-full mt-1"
              >
                <option value="command">{t.hooks.hookTypeCommand}</option>
                <option value="http">{t.hooks.hookTypeHttp}</option>
                <option value="mcp_tool">{t.hooks.hookTypeMcpTool}</option>
                <option value="prompt">{t.hooks.hookTypePrompt}</option>
                <option value="agent">{t.hooks.hookTypeAgent}</option>
              </Select>
            </div>

            {form.hookType === "command" && (
              <>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.actionLabel}
                  </label>
                  <div className="flex gap-1 mt-1">
                    <Button
                      variant="chip"
                      active={form.action === "message"}
                      onClick={() => setForm({ ...form, action: "message", timeout: "", isAsync: false })}
                    >
                      {t.hooks.actionMessage}
                    </Button>
                    <Button
                      variant="chip"
                      active={form.action === "command"}
                      onClick={() => setForm({ ...form, action: "command" })}
                    >
                      {t.hooks.actionCommand}
                    </Button>
                  </div>
                </div>

                {form.action === "message" ? (
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.hooks.instructionsLabel} <span style={{ color: "var(--danger)" }}>*</span>
                    </label>
                    <TextArea
                      value={form.message}
                      onChange={(e) => setForm({ ...form, message: e.target.value })}
                      rows={3}
                      placeholder={t.hooks.instructionsPlaceholder}
                      className="w-full mt-1"
                    />
                    <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                      {t.hooks.instructionsHint}
                    </div>
                  </div>
                ) : (
                  <>
                    <div>
                      <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                        {t.hooks.commandLabel} <span style={{ color: "var(--danger)" }}>*</span>
                      </label>
                      <TextArea
                        value={form.command}
                        onChange={(e) => setForm({ ...form, command: e.target.value })}
                        rows={3}
                        placeholder="python3 .claude/hooks/mi_hook.py"
                        className="w-full mt-1"
                        mono
                      />
                      <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                        {t.hooks.commandHint}
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                          {t.hooks.shellLabel}
                        </label>
                        <Select value={form.shell} onChange={(e) => setForm({ ...form, shell: e.target.value })} className="w-full mt-1">
                          <option value="">{t.hooks.defaultShort}</option>
                          <option value="bash">bash</option>
                          <option value="powershell">powershell</option>
                        </Select>
                      </div>
                      <div>
                        <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                          {t.hooks.onFailureLabel}
                        </label>
                        <Select value={form.onFailure} onChange={(e) => setForm({ ...form, onFailure: e.target.value })} className="w-full mt-1">
                          <option value="">{t.hooks.onFailureDefault}</option>
                          <option value="continue">continue</option>
                          <option value="block">block</option>
                        </Select>
                      </div>
                    </div>
                    <div>
                      <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                        {t.hooks.argsLabel}
                      </label>
                      <div className="mt-1">
                        <TagInput
                          tags={form.args}
                          onChange={(args) => setForm({ ...form, args })}
                          suggestions={[]}
                          placeholder={t.hooks.argsPlaceholder}
                        />
                      </div>
                    </div>
                  </>
                )}

                <div className="flex gap-4 items-end">
                  <div className="w-32">
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.hooks.timeoutLabel}
                    </label>
                    <TextInput
                      type="number"
                      value={form.timeout}
                      onChange={(e) =>
                        setForm({ ...form, timeout: e.target.value === "" ? "" : Number(e.target.value) })
                      }
                      className="w-full mt-1"
                    />
                  </div>
                  <Checkbox
                    label={t.hooks.asyncLabel}
                    checked={form.isAsync}
                    onChange={(e) => setForm({ ...form, isAsync: e.target.checked })}
                  />
                  {form.isAsync && (
                    <Checkbox
                      label={t.hooks.asyncRewakeLabel}
                      checked={form.asyncRewake === "true"}
                      onChange={(e) => setForm({ ...form, asyncRewake: e.target.checked ? "true" : "" })}
                    />
                  )}
                </div>
              </>
            )}

            {form.hookType === "http" && (
              <>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.urlLabel} <span style={{ color: "var(--danger)" }}>*</span>
                  </label>
                  <TextInput
                    value={form.url}
                    onChange={(e) => setForm({ ...form, url: e.target.value })}
                    placeholder="https://example.com/webhook"
                    className="w-full mt-1"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.headersLabel}
                  </label>
                  <TextArea
                    value={form.headers}
                    onChange={(e) => setForm({ ...form, headers: e.target.value })}
                    rows={2}
                    placeholder={"Authorization: Bearer $TOKEN"}
                    className="w-full mt-1"
                    small
                    mono
                  />
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.allowedEnvVarsLabel}
                  </label>
                  <div className="mt-1">
                    <TagInput
                      tags={form.allowedEnvVars}
                      onChange={(allowedEnvVars) => setForm({ ...form, allowedEnvVars })}
                      suggestions={[]}
                      placeholder={`${t.common.eg}: TOKEN`}
                    />
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.onFailureLabel}
                  </label>
                  <Select value={form.onFailure} onChange={(e) => setForm({ ...form, onFailure: e.target.value })} className="w-full mt-1">
                    <option value="">{t.hooks.onFailureDefault}</option>
                    <option value="continue">continue</option>
                    <option value="block">block</option>
                  </Select>
                </div>
              </>
            )}

            {form.hookType === "mcp_tool" && (
              <>
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.hooks.serverLabel} <span style={{ color: "var(--danger)" }}>*</span>
                    </label>
                    <TextInput
                      value={form.server}
                      onChange={(e) => setForm({ ...form, server: e.target.value })}
                      placeholder="github"
                      className="w-full mt-1"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                      {t.hooks.toolLabel} <span style={{ color: "var(--danger)" }}>*</span>
                    </label>
                    <TextInput
                      value={form.tool}
                      onChange={(e) => setForm({ ...form, tool: e.target.value })}
                      placeholder="create_issue"
                      className="w-full mt-1"
                    />
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.inputLabel}
                  </label>
                  <TextArea
                    value={form.input}
                    onChange={(e) => setForm({ ...form, input: e.target.value })}
                    rows={2}
                    placeholder='{"title": "..."}'
                    className="w-full mt-1"
                    small
                    mono
                  />
                </div>
              </>
            )}

            {(form.hookType === "prompt" || form.hookType === "agent") && (
              <>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.promptLabel} <span style={{ color: "var(--danger)" }}>*</span>
                  </label>
                  <TextArea
                    value={form.prompt}
                    onChange={(e) => setForm({ ...form, prompt: e.target.value })}
                    rows={3}
                    placeholder={t.hooks.promptPlaceholder}
                    className="w-full mt-1"
                  />
                  <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.promptHint}
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.promptModelLabel}
                  </label>
                  <TextInput
                    value={form.promptModel}
                    onChange={(e) => setForm({ ...form, promptModel: e.target.value })}
                    placeholder={t.hooks.promptModelPlaceholder}
                    className="w-full mt-1"
                  />
                </div>
              </>
            )}

            <details className="rounded-lg border p-3" style={{ borderColor: "var(--border)" }}>
              <summary className="text-xs font-medium cursor-pointer" style={{ color: "var(--text-muted)" }}>
                {t.hooks.advanced}
              </summary>
              <div className="mt-3 space-y-3">
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.ifLabel}
                  </label>
                  <TextInput
                    value={form.ifFilter}
                    onChange={(e) => setForm({ ...form, ifFilter: e.target.value })}
                    placeholder="Edit(*.ts)"
                    className="w-full mt-1"
                    mono
                  />
                  <div className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.ifHint}
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
                    {t.hooks.statusMessageLabel}
                  </label>
                  <TextInput
                    value={form.statusMessage}
                    onChange={(e) => setForm({ ...form, statusMessage: e.target.value })}
                    className="w-full mt-1"
                  />
                </div>
              </div>
            </details>

            <div className="flex gap-2 pt-2">
              <Button onClick={save}>{t.common.save}</Button>
              {typeof editIndex === "number" && (
                <>
                  <Button variant="ghost" onClick={moveToOtherScope}>
                    {scope === "project" ? t.common.moveToGlobal : t.common.moveToProject}
                  </Button>
                  <Button variant="danger" onClick={() => remove(editIndex)}>
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
